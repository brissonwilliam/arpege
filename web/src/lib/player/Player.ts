import type { SongMeta } from "./types";
import { PlayerState } from "./types";

export type PlayerOptions = {
    baseUrl?: string;
};

const LOG_PREFIX = "[Player]";

/**
 * Chunked playback over `/api/songs/md` and `/api/songs/chunks?c=…` using
 * HTMLAudioElement and blob URLs.
 */
export class Player {
    private readonly baseUrl: string;
    private readonly audio: HTMLAudioElement;
    private _state: PlayerState = PlayerState.PAUSED;
    private songMeta: SongMeta | null = null;
    /** Index of the chunk currently loaded into `audio`. */
    private chunkIndex = 0;
    private objectUrl: string | null = null;
    /** Used to abort any previous ongoing actions */
    private actionCounter = 0;
    /** Song played through to the end; `getCurrentTimeMs` reports `duration_ms` until seek/play. */
    private playbackComplete = false;

    private readonly onAudioEnded = () => {
        void this.handleEnded();
    };

    private readonly onAudioError = () => {
        this.handleAudioError();
    };

    /** Creates a player; optional `baseUrl` defaults to `"/api"` (Vite proxy). */
    constructor(options?: PlayerOptions) {
        this.baseUrl = options?.baseUrl ?? "/api";
        this.audio = new Audio();
        this.audio.addEventListener("ended", this.onAudioEnded);
        this.audio.addEventListener("error", this.onAudioError);
        console.log(LOG_PREFIX, "constructed", { baseUrl: this.baseUrl });
    }

    /** Current high-level playback lifecycle state. */
    get state(): PlayerState {
        return this._state;
    }

    /** Last loaded song metadata, or `null` before a successful `load()`. */
    get meta(): SongMeta | null {
        return this.songMeta;
    }

    /** Updates internal state (used by async paths and audio events). */
    private setState(s: PlayerState) {
        this._state = s;
    }

    /** Dispatched when the media element hits a decode/network error. */
    private handleAudioError() {
        console.warn(LOG_PREFIX, "audio error event");
        this.setState(PlayerState.ERROR);
    }

    /** Revokes the current blob object URL so the blob can be GC’d. */
    private revokeUrl() {
        if (this.objectUrl) {
            URL.revokeObjectURL(this.objectUrl);
            this.objectUrl = null;
        }
    }

    /** Builds the chunk GET URL with a properly encoded `c` query param. */
    private chunkRequestUrl(chunkId: string): string {
        const url = `${this.baseUrl}/songs/chunks?c=${encodeURIComponent(chunkId)}`;
        return url;
    }

    /** Fetches one chunk’s bytes from the API. */
    private async fetchChunkBuffer(id: string): Promise<ArrayBuffer> {
        const res = await fetch(this.chunkRequestUrl(id));
        if (!res.ok) {
            throw new Error(`chunk HTTP ${res.status}`);
        }
        const buf = await res.arrayBuffer();
        return buf;
    }

    /**
     * Waits until `HTMLMediaElement` has enough data to know `duration` / seek safely,
     * or rejects if the element errors while loading.
     */
    private waitLoadedMetadata(): Promise<void> {
        return new Promise((resolve, reject) => {
            if (this.audio.readyState >= HTMLMediaElement.HAVE_METADATA) {
                resolve();
                return;
            }
            const ok = () => {
                cleanup();
                resolve();
            };
            const bad = () => {
                cleanup();
                reject(new Error("audio metadata"));
            };
            const cleanup = () => {
                this.audio.removeEventListener("loadedmetadata", ok);
                this.audio.removeEventListener("error", bad);
            };
            this.audio.addEventListener("loadedmetadata", ok, { once: true });
            this.audio.addEventListener("error", bad, { once: true });
        });
    }

    /**
     * Downloads chunk `i`, wraps it in a Blob with `mime_type`, assigns an object URL to `audio`,
     * and waits for metadata so `duration` / `currentTime` are usable.
     */
    private async prepareChunk(i: number): Promise<void> {
        if (!this.songMeta) {
            throw new Error("no song metadata");
        }
        const chunk = this.songMeta.chunks[i];
        console.log(LOG_PREFIX, "prepareChunk", { index: i, chunkId: chunk.id });
        const buf = await this.fetchChunkBuffer(chunk.id);
        this.revokeUrl();
        const blob = new Blob([buf], { type: this.songMeta.mime_type });
        this.objectUrl = URL.createObjectURL(blob);
        this.chunkIndex = i;
        this.audio.src = this.objectUrl;
        await this.waitLoadedMetadata();
    }

    /** Clamps a timeline position to `[0, song.duration_ms]`. */
    private clampTimelineMs(ms: number): number {
        if (!this.songMeta) {
            return 0;
        }
        const lo = 0;
        const hi = this.songMeta.duration_ms;
        const clamped = Math.max(lo, Math.min(ms, hi));
        return clamped;
    }

    /**
     * Maps a global timeline position (ms) to a chunk index.
     *
     * **Primary rule:** half-open intervals `[start_ms, end_ms)` so boundaries are unambiguous
     * when chunks are contiguous and non-overlapping.
     *
     * **Fallback (second loop):** If metadata has **gaps** (e.g. `end_ms` of chunk *i* is less
     * than `start_ms` of chunk *i+1*) then `positionMs` can fall in no chunk’s half-open range,
     * so the first loop finds nothing. The backward scan picks the **last** chunk whose
     * `start_ms <= positionMs`, i.e. the chunk we’ve already “entered” by wall-clock time—
     * a reasonable approximation for scrubbing in a gap. If the API ever guarantees perfect
     * tiling with no gaps, the first loop always matches and the fallback never runs.
     *
     * **Last resort:** index `0` if nothing matches (e.g. negative `positionMs` with positive
     * chunk starts).
     */
    private findChunkIndex(positionMs: number): number {
        const chunks = this.songMeta!.chunks;

        for (let i = 0; i < chunks.length; i++) {
            const c = chunks[i];
            const inRange = positionMs >= c.start_ms && positionMs < c.end_ms;
            if (inRange) {
                return i;
            }
        }

        console.log(
            LOG_PREFIX,
            "findChunkIndex: no [start,end) match (gap or overlap edge case); using fallback",
            { positionMs },
        );

        for (let i = chunks.length - 1; i >= 0; i--) {
            const c = chunks[i];
            if (positionMs >= c.start_ms) {
                return i;
            }
        }

        console.log(LOG_PREFIX, "findChunkIndex: no chunk start <= position; using 0", {
            positionMs,
        });
        return 0;
    }

    /**
     * Seeks within the **already loaded** chunk: sets `audio.currentTime` to the offset that
     * corresponds to `timelineMs` on the global timeline.
     */
    private applyOffsetInCurrentChunk(timelineMs: number) {
        const chunk = this.songMeta!.chunks[this.chunkIndex];
        const offsetSecRaw = (timelineMs - chunk.start_ms) / 1000;
        let sec = Math.max(0, offsetSecRaw);

        // avoiding aligning to the very last ms, which could trigger
        // what browsers consider a "end of stream"
        const d = this.audio.duration;
        if (d > 0 && sec > 0 && sec >= d) {
            sec -= 0.001
        }

        this.audio.currentTime = sec;
        console.log(LOG_PREFIX, "applyOffsetInCurrentChunk", {
            timelineMs,
            chunkIndex: this.chunkIndex,
            currentTimeSec: sec,
            reportedDurationSec: d,
        });
    }

    /**
     * Loads song metadata from `GET ${baseUrl}/songs/md` and resets playback state for a new track.
     */
    async load(): Promise<void> {
        this.actionCounter += 1;
        const ac = this.actionCounter;
        this.audio.pause();
        this.revokeUrl();
        this.audio.removeAttribute("src");
        this.chunkIndex = 0;
        this.playbackComplete = false;
        this.songMeta = null;

        console.log(LOG_PREFIX, "load: fetching metadata");

        try {
            const res = await fetch(`${this.baseUrl}/songs/md`);
            if (!res.ok) {
                throw new Error(`md HTTP ${res.status}`);
            }
            const data = (await res.json()) as SongMeta;
            if (ac !== this.actionCounter) {
                return;
            }
            if (!data.chunks?.length) {
                throw new Error("no chunks");
            }
            this.songMeta = data;
            console.log(LOG_PREFIX, "load: ok", {
                id: data.id,
                chunks: data.chunks.length,
                duration_ms: data.duration_ms,
            });
        } catch (e) {
            if (ac !== this.actionCounter) {
                return;
            }
            this.songMeta = null;
            this.setState(PlayerState.ERROR);
            console.warn(LOG_PREFIX, "load: failed", e);
        }
    }

    /**
     * Returns the current global timeline position in ms for UI (e.g. progress bar),
     * using chunk `start_ms` plus the element’s `currentTime` within the loaded blob.
     */
    getCurrentTimeMs(): number {
        if (!this.songMeta) {
            return 0;
        }
        if (this.playbackComplete) {
            return this.songMeta.duration_ms;
        }
        const chunks = this.songMeta.chunks;
        if (this.chunkIndex < 0 || this.chunkIndex >= chunks.length) {
            return 0;
        }
        const base = chunks[this.chunkIndex].start_ms;
        const t = this.audio.currentTime;
        if (!Number.isFinite(t)) {
            return this.clampTimelineMs(base);
        }
        const combined = base + t * 1000;
        return this.clampTimelineMs(combined);
    }

    /**
     * Seeks to `positionMs` on the song timeline: may fetch a different chunk and sets offset
     * within that chunk. Preserves play/pause (was playing → still playing after seek).
     */
    async seek(positionMs: number): Promise<void> {
        if (!this.songMeta) {
            console.log(LOG_PREFIX, "seek without md");
            return;
        }
        this.actionCounter += 1;
        const gen = this.actionCounter;
        const wasPlaying = !this.audio.paused;
        const ms = this.clampTimelineMs(positionMs);
        this.playbackComplete = false;
        const i = this.findChunkIndex(ms);

        console.log(LOG_PREFIX, "seek", { requested: positionMs, clamped: ms, chunkIndex: i });

        try {
            const sameChunkLoaded =
                this.chunkIndex === i && Boolean(this.objectUrl && this.audio.src);

            if (sameChunkLoaded) {
                this.applyOffsetInCurrentChunk(ms);
                if (gen !== this.actionCounter) {
                    return;
                }
                if (wasPlaying) {
                    await this.audio.play();
                    if (gen !== this.actionCounter) {
                        return;
                    }
                    this.setState(PlayerState.PLAYING);
                } else {
                    this.setState(PlayerState.PAUSED);
                }
                return;
            }

            await this.prepareChunk(i);
            if (gen !== this.actionCounter) {
                return;
            }
            this.applyOffsetInCurrentChunk(ms);
            if (wasPlaying) {
                await this.audio.play();
                if (gen !== this.actionCounter) {
                    return;
                }
                this.setState(PlayerState.PLAYING);
            } else {
                this.setState(PlayerState.PAUSED);
            }
        } catch (e) {
            if (gen !== this.actionCounter) {
                return;
            }
            this.setState(PlayerState.ERROR);
            console.warn(LOG_PREFIX, "seek: error", e);
        }
    }

    isLoaded(): boolean {
        return this.songMeta != null;
    }

    /**
     * Starts or resumes playback; loads the current chunk’s blob first if `audio` has no source.
     */
    async play(): Promise<void> {
        if (!this.isLoaded()) {
            console.log(LOG_PREFIX, "cannot play: not loaded")
            return
        }
        if (this._state == PlayerState.ERROR) {
            console.log(LOG_PREFIX, "cannot play: ERR")
            return
        }
        const gen = this.actionCounter;
        this.playbackComplete = false;

        try {
            if (!this.audio.src || !this.objectUrl) {
                await this.prepareChunk(this.chunkIndex);
                if (gen !== this.actionCounter) {
                    return;
                }
            }
            await this.audio.play();
            if (gen !== this.actionCounter) {
                return;
            }
            this.setState(PlayerState.PLAYING);
            console.log(LOG_PREFIX, "play: playing");
        } catch (e) {
            if (gen !== this.actionCounter) {
                return;
            }
            this.setState(PlayerState.ERROR);
            console.warn(LOG_PREFIX, "play: error", e);
        }
    }

    /** Pauses playback and updates state when we were `playing`. */
    pause(): void {
        this.audio.pause();
        if (this._state === PlayerState.PLAYING) {
            this.setState(PlayerState.PAUSED);
        }
        console.log(LOG_PREFIX, "pause");
    }

    /**
     * When a chunk finishes naturally, loads the next chunk and continues playing; on the last
     * chunk, clears the source and marks playback complete for timeline reporting.
     */
    private async handleEnded() {
        if (!this.songMeta) {
            return;
        }
        const ac = this.actionCounter;
        const next = this.chunkIndex + 1;
        if (next >= this.songMeta.chunks.length) {
            console.log(LOG_PREFIX, "handleEnded: song complete");
            this.playbackComplete = true;
            this.chunkIndex = 0;
            this.revokeUrl();
            this.audio.removeAttribute("src");
            if (ac !== this.actionCounter) {
                return;
            }
            this.setState(PlayerState.PAUSED);
            return;
        }

        try {
            console.log(LOG_PREFIX, "handleEnded: advancing chunk", { from: this.chunkIndex, to: next });
            await this.prepareChunk(next);
            if (ac !== this.actionCounter) {
                return;
            }
            await this.audio.play();
            if (ac !== this.actionCounter) {
                return;
            }
            this.setState(PlayerState.PLAYING);
        } catch (e) {
            if (ac !== this.actionCounter) {
                return;
            }
            this.setState(PlayerState.ERROR);
            console.warn(LOG_PREFIX, "handleEnded: error", e);
        }
    }

    /**
     * Tears down listeners and revokes blob URLs. Do not use this instance after calling.
     */
    destroy(): void {
        this.actionCounter += 1;
        this.audio.removeEventListener("ended", this.onAudioEnded);
        this.audio.removeEventListener("error", this.onAudioError);
        this.audio.pause();
        this.audio.removeAttribute("src");
        this.revokeUrl();
        this.songMeta = null;
        this.playbackComplete = false;
        this.chunkIndex = 0;
        this.loaded = false;
        console.log(LOG_PREFIX, "destroy");
    }
}
