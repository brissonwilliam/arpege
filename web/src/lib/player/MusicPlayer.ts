import type { SongMeta, ChunkMeta } from "./types";

const PREFETCH_AHEAD = 3; // how many chunks to keep fetching ahead of current

/**
 * MusicPlayer wraps the MediaSource Extensions (MSE) API.
 *
 * Flow:
 *   1. A MediaSource is created and attached to an <audio> element via a blob URL.
 *   2. Once the MediaSource is open, a SourceBuffer is added for the song's mime type.
 *   3. Chunks are fetched sequentially and appended to the SourceBuffer.
 *   4. The audio element plays from the SourceBuffer as data arrives.
 *
 * Chunk alignment contract (enforced by the server, trusted here):
 *   - MP3:       Each chunk starts on an MPEG frame boundary (~417–522 bytes aligned).
 *               MP3 frames are self-contained so no init segment is needed.
 *   - AAC/fMP4: Chunk 0 contains the fMP4 init segment (moov + ftyp boxes).
 *               Subsequent chunks are fMP4 media segments (moof + mdat).
 *               mime_type must be "audio/mp4; codecs=\"mp4a.40.2\"" (or similar).
 *   - OGG/Opus: Each chunk starts on an Ogg page boundary.
 *               The first page of the first chunk contains the Opus header.
 *
 * Seeking:
 *   We find the chunk whose range covers the target ms, abort in-flight fetches,
 *   call SourceBuffer.remove() to flush buffered data past the seek point,
 *   then re-fetch from that chunk onward.
 */
export class MusicPlayer {
    private audio: HTMLAudioElement;
    private mediaSource: MediaSource | null = null;
    private sourceBuffer: SourceBuffer | null = null;
    private meta: SongMeta | null = null;

    // Chunk fetch state
    private nextChunkIndex = 0;
    private fetchController: AbortController | null = null;
    private appendQueue: ArrayBuffer[] = [];
    private isAppending = false;

    // Callbacks for the store layer
    onStateChange?: (state: "playing" | "paused" | "loading" | "error") => void;
    onTimeUpdate?: (currentMs: number) => void;
    onError?: (msg: string) => void;

    constructor(audio: HTMLAudioElement) {
        this.audio = audio;
        this.audio.addEventListener("timeupdate", () => {
            this.onTimeUpdate?.(this.audio.currentTime * 1000);
            this.maybePrefetch();
        });
        this.audio.addEventListener("playing", () => this.onStateChange?.("playing"));
        this.audio.addEventListener("pause", () => this.onStateChange?.("paused"));
        this.audio.addEventListener("waiting", () => this.onStateChange?.("loading"));
    }

    // ── Public API ──────────────────────────────────────────────────────────────

    async load(meta: SongMeta): Promise<void> {
        this.meta = meta;
        this.teardown();

        if (!MediaSource.isTypeSupported(meta.mime_type)) {
            this.emitError(`Unsupported mime type: ${meta.mime_type}`);
            return;
        }

        this.mediaSource = new MediaSource();
        // The blob URL is the glue between MSE and the audio element.
        // The MediaSource lives as long as this URL is referenced.
        this.audio.src = URL.createObjectURL(this.mediaSource);

        await this.waitForSourceOpen();

        this.sourceBuffer = this.mediaSource.addSourceBuffer(meta.mime_type);
        // updateend fires after each SourceBuffer.appendBuffer() completes.
        // We use it as a drain signal for the append queue.
        this.sourceBuffer.addEventListener("updateend", () => this.drainQueue());
        this.sourceBuffer.addEventListener("error", (e) => {
            this.emitError(`SourceBuffer error: ${(e as ErrorEvent).message ?? "unknown"}`);
        });

        this.nextChunkIndex = 0;
        this.fetchNextChunks();
    }

    play(): void {
        this.audio.play().catch((e) => this.emitError(e.message));
    }

    pause(): void {
        this.audio.pause();
    }

    /**
     * Seek to a position in milliseconds.
     * We find the chunk covering seekMs, flush the SourceBuffer from that point,
     * and re-fetch from that chunk index onward.
     */
    seek(seekMs: number): void {
        if (!this.meta || !this.sourceBuffer) return;

        const chunkIndex = this.findChunkIndex(seekMs);
        if (chunkIndex === -1) return;

        // Cancel in-flight fetches
        this.fetchController?.abort();
        this.fetchController = null;
        this.appendQueue = [];

        const seekSeconds = seekMs / 1000;
        this.audio.currentTime = seekSeconds;
        this.nextChunkIndex = chunkIndex;

        // Flush everything after seekMs from the buffer so MSE doesn't get confused.
        // We wait for the remove() to complete (updateend) before re-fetching.
        if (!this.sourceBuffer.updating) {
            this.removeAndRefetch(seekSeconds);
        } else {
            this.sourceBuffer.addEventListener(
                "updateend",
                () => this.removeAndRefetch(seekSeconds),
                { once: true }
            );
        }
    }

    destroy(): void {
        this.teardown();
        this.audio.src = "";
    }

    get currentTimeMs(): number {
        return this.audio.currentTime * 1000;
    }

    get paused(): boolean {
        return this.audio.paused;
    }

    // ── Private ─────────────────────────────────────────────────────────────────

    private waitForSourceOpen(): Promise<void> {
        return new Promise((resolve) => {
            if (this.mediaSource!.readyState === "open") {
                return resolve()
            };
            this.mediaSource!.addEventListener("sourceopen", () => resolve(), { once: true });
        });
    }

    private async fetchNextChunks(): Promise<void> {
        if (!this.meta) return;
        const { chunks } = this.meta;

        const limit = Math.min(this.nextChunkIndex + PREFETCH_AHEAD, chunks.length);

        for (let i = this.nextChunkIndex; i < limit; i++) {
            await this.fetchChunk(chunks[i]);
            if (this.nextChunkIndex !== i + 1) {
                return; // seek happened mid-fetch
            }
        }
    }

    private async fetchChunk(chunk: ChunkMeta): Promise<void> {
        if (!this.meta) return;

        this.fetchController = new AbortController();
        try {
            const res = await fetch(
                `/api/song/${this.meta.id}/chunks?c=${chunk.uuid}`,
                { signal: this.fetchController.signal }
            );
            if (!res.ok) {
                throw new Error(`HTTP ${res.status}`)
            };

            const buffer = await res.arrayBuffer();
            this.enqueueAppend(buffer);
            this.nextChunkIndex++;
        } catch (e: unknown) {
            if (e instanceof DOMException && e.name === "AbortError") {
                return; // expected on seek
            }
            this.emitError(`Chunk fetch failed: ${(e as Error).message}`);
        }
    }

    /**
     * We queue append operations because SourceBuffer can only handle one
     * appendBuffer() at a time. Calling it while updating=true throws.
     */
    private enqueueAppend(buffer: ArrayBuffer): void {
        this.appendQueue.push(buffer);
        this.drainQueue();
    }

    private drainQueue(): void {
        if (this.isAppending) return;
        if (!this.sourceBuffer || this.sourceBuffer.updating) {
            return
        }
        const next = this.appendQueue.shift();
        if (!next) {
            return
        };

        this.isAppending = true;
        try {
            this.sourceBuffer.appendBuffer(next);
        } catch (e) {
            this.emitError(`appendBuffer failed: ${(e as Error).message}`);
        } finally {
            this.isAppending = false;
        }
    }

    /**
     * Triggered by timeupdate. Fetch more chunks when we're running low on
     * buffered-ahead data.
     */
    private maybePrefetch(): void {
        if (!this.meta) return;
        const bufferedAheadMs = this.getBufferedAheadMs();
        const chunkDurationMs = this.getAverageChunkDurationMs();

        // If we have less than PREFETCH_AHEAD chunks worth of buffer, fetch more
        if (bufferedAheadMs < chunkDurationMs * PREFETCH_AHEAD) {
            this.fetchNextChunks();
        }
    }

    private getBufferedAheadMs(): number {
        const buf = this.audio.buffered;
        const now = this.audio.currentTime;
        for (let i = 0; i < buf.length; i++) {
            if (buf.start(i) <= now && buf.end(i) >= now) {
                return (buf.end(i) - now) * 1000;
            }
        }
        return 0;
    }

    private getAverageChunkDurationMs(): number {
        if (!this.meta || this.meta.chunks.length === 0) {
            return 5000;
        }
        const total = this.meta.chunks.reduce((s, c) => s + (c.end_ms - c.start_ms), 0);
        return total / this.meta.chunks.length;
    }

    private findChunkIndex(seekMs: number): number {
        if (!this.meta) {
            return -1;
        }
        return this.meta.chunks.findIndex(
            (c) => c.start_ms <= seekMs && seekMs < c.end_ms
        );
    }

    private removeAndRefetch(fromSeconds: number): void {
        if (!this.sourceBuffer || !this.meta) return;
        const duration = this.meta.duration_ms / 1000;
        try {
            // Remove all buffered data from seek point to end so MSE accepts new data
            this.sourceBuffer.remove(fromSeconds, duration + 1);
            this.sourceBuffer.addEventListener("updateend", () => this.fetchNextChunks(), {
                once: true,
            });
        } catch {
            // If remove() fails (e.g. nothing buffered there), just fetch
            this.fetchNextChunks();
        }
    }

    private teardown(): void {
        this.fetchController?.abort();
        this.fetchController = null;
        this.appendQueue = [];
        this.isAppending = false;
        this.nextChunkIndex = 0;

        if (this.audio.src.startsWith("blob:")) {
            URL.revokeObjectURL(this.audio.src);
        }
        if (this.mediaSource && this.mediaSource.readyState === "open") {
            try { this.mediaSource.endOfStream(); } catch { /* ignore */ }
        }
        this.sourceBuffer = null;
        this.mediaSource = null;
    }

    private emitError(msg: string): void {
        this.onStateChange?.("error");
        this.onError?.(msg);
        console.error("[MusicPlayer]", msg);
    }
}
