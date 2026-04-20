import type { SongMeta } from "./types";
import { PlayerState } from "./types";

export type PlayerOptions = {
    baseUrl?: string;
};

const LOG_PREFIX = "[Player]";

/** How many upcoming chunks to decode ahead of the schedule tail (not counting the current one). */
const PREFETCH_CHUNK_COUNT = 4;

function isAbortError(e: unknown): boolean {
    return e instanceof Error && e.message === "aborted";
}

/**
 * Chunked playback over `/api/songs/md` and `/api/songs/chunks?c=…` using Web Audio
 * (`decodeAudioData` + scheduled `AudioBufferSourceNode`s) so chunk boundaries are
 * sample-accurate, with aggressive decode-ahead for smooth continuation.
 */
export class Player {
    private readonly baseUrl: string;
    private _state: PlayerState = PlayerState.PAUSED;
    private songMeta: SongMeta | null = null;
    /** Used to abort any previous ongoing actions */
    private actionCounter = 0;
    /** Song played through to the end; `getCurrentTimeMs` reports `duration_ms` until seek/play. */
    private playbackComplete = false;

    private audioCtx: AudioContext | null = null;
    private gainNode: GainNode | null = null;
    /** Output level 0–100 (linear), applied to `gainNode` when the context exists. */
    private _gainPercent = 100;

    private decoded = new Map<number, AudioBuffer>();
    private decoding = new Map<number, Promise<AudioBuffer>>();

    private activeSources: AudioBufferSourceNode[] = [];

    /** Timeline head when paused / before first `play` (ms). */
    private pausedAtMs = 0;

    /** When `isAudioRunning`, maps `audioCtx.currentTime` to the global timeline (ms). */
    private playStartTimelineMs = 0;
    private playStartCtxTime = 0;
    private isAudioRunning = false;

    /** Next chunk index to append after `scheduleTailCtxTime` (exclusive end of chain so far). */
    private scheduleNextIdx = 0;
    /** AudioContext time where the next decoded chunk should begin. */
    private scheduleTailCtxTime = 0;

    /** Creates a player; optional `baseUrl` defaults to `"/api"` (Vite proxy). */
    constructor(options?: PlayerOptions) {
        this.baseUrl = options?.baseUrl ?? "/api";
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

    get isPlaying(): boolean {
        return this.isAudioRunning;
    }

    /** Playback volume 0–100 (linear, maps to `GainNode.gain`). */
    get gainPercent(): number {
        return this._gainPercent;
    }

    /** Sets playback volume 0–100 (linear). Safe to call before the first `play()`. */
    setGainPercent(percent: number): void {
        const p = Math.max(0, Math.min(100, Math.round(percent)));
        this._gainPercent = p;
        this.applyGainToNode();
    }

    private applyGainToNode(): void {
        if (this.gainNode) {
            this.gainNode.gain.value = this._gainPercent / 100;
        }
    }

    /** Updates internal state (used by async paths and audio events). */
    private setState(s: PlayerState) {
        this._state = s;
    }

    private stopAllSources() {
        for (const s of this.activeSources) {
            try {
                s.stop(0);
            } catch {
                /* already stopped / not started */
            }
            try {
                s.disconnect();
            } catch {
                /* ignore */
            }
        }
        this.activeSources = [];
    }

    /** Creates `AudioContext` / `GainNode` if needed; does **not** resume (decode works while suspended). */
    private ensureContextExists(): AudioContext {
        if (!this.audioCtx) {
            const Ctor =
                window.AudioContext ??
                (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
            if (!Ctor) {
                throw new Error("Web Audio not supported");
            }
            this.audioCtx = new Ctor();
            this.gainNode = this.audioCtx.createGain();
            this.applyGainToNode();
            this.gainNode.connect(this.audioCtx.destination);
        }
        return this.audioCtx;
    }

    private async resumeContext(): Promise<AudioContext> {
        const ctx = this.ensureContextExists();
        if (ctx.state === "suspended") {
            await ctx.resume();
        }
        return ctx;
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

    private clearDecodeCaches() {
        this.decoded.clear();
        this.decoding.clear();
    }

    /**
     * Decodes chunk `i` to an `AudioBuffer` (cached). Throws `{ message: 'aborted' }` if superseded
     * by `load`, `seek`, or `destroy` while fetching/decoding.
     */
    private async ensureDecoded(i: number): Promise<AudioBuffer> {
        if (!this.songMeta) {
            throw new Error("no song metadata");
        }
        const hit = this.decoded.get(i);
        if (hit) {
            return hit;
        }
        const inflight = this.decoding.get(i);
        if (inflight) {
            return inflight;
        }

        const callGen = this.actionCounter;

        const task = (async () => {
            const chunk = this.songMeta!.chunks[i];
            console.log(LOG_PREFIX, "ensureDecoded", { index: i, chunkId: chunk.id });
            const buf = await this.fetchChunkBuffer(chunk.id);
            if (callGen !== this.actionCounter) {
                throw new Error("aborted");
            }
            const ctx = this.ensureContextExists();
            const copy = buf.slice(0);
            const audioBuf = await ctx.decodeAudioData(copy);
            if (callGen !== this.actionCounter) {
                throw new Error("aborted");
            }
            this.decoded.set(i, audioBuf);
            this.decoding.delete(i);
            if (this.isAudioRunning) {
                this.appendScheduledChain();
            }
            return audioBuf;
        })();

        this.decoding.set(i, task);
        return task;
    }

    /** Decode chunk `i` and, if present, `i+1` in parallel so the second can be scheduled immediately after the first. */
    private async ensureDecodedPairForStart(i: number): Promise<void> {
        if (!this.songMeta) {
            return;
        }
        const n = this.songMeta.chunks.length;
        const tasks: Promise<AudioBuffer>[] = [this.ensureDecoded(i)];
        if (i + 1 < n) {
            tasks.push(this.ensureDecoded(i + 1));
        }
        await Promise.all(tasks);
    }

    /** Starts background decode for `count` chunks starting at `fromIndex` (inclusive). */
    private prefetchDecodeAhead(fromIndex: number, count: number) {
        if (!this.songMeta) {
            return;
        }
        const n = this.songMeta.chunks.length;
        const to = Math.min(fromIndex + count, n);
        for (let j = fromIndex; j < to; j++) {
            void this.ensureDecoded(j).catch((e) => {
                if (!isAbortError(e)) {
                    console.warn(LOG_PREFIX, "prefetch decode failed", { index: j, e });
                }
            });
        }
    }

    /** Appends every already-decoded chunk that fits the tail of the scheduled chain. */
    private appendScheduledChain() {
        if (!this.songMeta || !this.audioCtx || !this.gainNode) {
            return;
        }
        const gen = this.actionCounter;
        const { chunks } = this.songMeta;

        while (this.scheduleNextIdx < chunks.length) {
            const buf = this.decoded.get(this.scheduleNextIdx);
            if (!buf) {
                this.prefetchDecodeAhead(this.scheduleNextIdx, PREFETCH_CHUNK_COUNT);
                return;
            }

            const src = this.audioCtx.createBufferSource();
            src.buffer = buf;
            src.connect(this.gainNode);

            const idx = this.scheduleNextIdx;
            const isLast = idx === chunks.length - 1;
            if (isLast) {
                src.onended = () => {
                    if (gen !== this.actionCounter) {
                        return;
                    }
                    this.handlePlaybackComplete();
                };
            }

            src.start(this.scheduleTailCtxTime, 0, buf.duration);
            this.activeSources.push(src);
            this.scheduleTailCtxTime += buf.duration;
            this.scheduleNextIdx += 1;
        }
    }

    private handlePlaybackComplete() {
        if (!this.songMeta) {
            return;
        }
        console.log(LOG_PREFIX, "playback complete");
        this.playbackComplete = true;
        this.isAudioRunning = false;
        this.pausedAtMs = this.songMeta.duration_ms;
        this.activeSources = [];
        this.setState(PlayerState.PAUSED);
    }

    /**
     * Stops any current chain and schedules gapless playback from global timeline position `timelineMs`.
     * Requires chunk `findChunkIndex(timelineMs)` to already be decoded (caller awaits `ensureDecoded`).
     */
    private async beginPlaybackFrom(timelineMs: number): Promise<void> {
        if (!this.songMeta || !this.gainNode) {
            throw new Error("no song");
        }
        const gen = this.actionCounter;
        this.stopAllSources();
        const ctx = await this.resumeContext();

        const chunks = this.songMeta.chunks;
        const i = this.findChunkIndex(timelineMs);
        const buf = this.decoded.get(i);
        if (!buf) {
            throw new Error("chunk not decoded");
        }

        const offSec = (timelineMs - chunks[i].start_ms) / 1000;
        let safeOff = Math.max(0, offSec);
        if (buf.duration > 0 && safeOff >= buf.duration) {
            safeOff = Math.max(0, buf.duration - 1e-3);
        }
        let partDur = buf.duration - safeOff;
        if (partDur <= 0) {
            partDur = buf.duration;
            safeOff = 0;
        }

        const when = ctx.currentTime;
        const lead = ctx.createBufferSource();
        lead.buffer = buf;
        lead.connect(this.gainNode);
        if (i === chunks.length - 1) {
            lead.onended = () => {
                if (gen !== this.actionCounter) {
                    return;
                }
                this.handlePlaybackComplete();
            };
        }
        lead.start(when, safeOff, partDur);
        this.activeSources.push(lead);

        this.playStartTimelineMs = timelineMs;
        this.playStartCtxTime = when;
        this.isAudioRunning = true;
        this.playbackComplete = false;

        this.scheduleTailCtxTime = when + partDur;
        this.scheduleNextIdx = i + 1;

        this.appendScheduledChain();
        this.prefetchDecodeAhead(this.scheduleNextIdx, PREFETCH_CHUNK_COUNT);
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

        for (let j = 0; j < chunks.length; j++) {
            const c = chunks[j];
            const inRange = positionMs >= c.start_ms && positionMs < c.end_ms;
            if (inRange) {
                return j;
            }
        }

        console.log(
            LOG_PREFIX,
            "findChunkIndex: no [start,end) match (gap or overlap edge case); using fallback",
            { positionMs },
        );

        for (let j = chunks.length - 1; j >= 0; j--) {
            const c = chunks[j];
            if (positionMs >= c.start_ms) {
                return j;
            }
        }

        console.log(LOG_PREFIX, "findChunkIndex: no chunk start <= position; using 0", {
            positionMs,
        });
        return 0;
    }

    /**
     * Loads song metadata from `GET ${baseUrl}/songs/md` and resets playback state for a new track.
     */
    async load(): Promise<void> {
        this.actionCounter += 1;
        const ac = this.actionCounter;

        this.stopAllSources();
        this.isAudioRunning = false;
        this.clearDecodeCaches();

        if (this.audioCtx) {
            try {
                await this.audioCtx.close();
            } catch {
                /* ignore */
            }
            this.audioCtx = null;
            this.gainNode = null;
        }

        this.playbackComplete = false;
        this.pausedAtMs = 0;
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
     * Current playback position on the song timeline in milliseconds (UI / polling).
     * Uses the Web Audio clock while playing and `pausedAtMs` while paused.
     */
    getCurrentTimeMs(): number {
        if (!this.songMeta) {
            return 0;
        }
        if (this.playbackComplete) {
            return this.songMeta.duration_ms;
        }
        if (this.isAudioRunning && this.audioCtx && this.audioCtx.state === "running") {
            const t =
                this.playStartTimelineMs + (this.audioCtx.currentTime - this.playStartCtxTime) * 1000;
            return this.clampTimelineMs(t);
        }
        return this.clampTimelineMs(this.pausedAtMs);
    }

    /**
     * Seeks to `second` (rounded to whole seconds) on the song timeline. Decodes the target chunk
     * (and kicks off decode-ahead) so `play()` can start immediately afterward. Preserves play/pause.
     */
    async seek(second: number): Promise<void> {
        if (!this.songMeta) {
            console.log(LOG_PREFIX, "seek without md");
            return;
        }
        this.actionCounter += 1;
        const gen = this.actionCounter;
        const wasPlaying = this.isAudioRunning;
        const durationSec = Math.max(0, Math.round(this.songMeta.duration_ms / 1000));
        const secClamped = Math.max(0, Math.min(Math.round(second), durationSec));
        const ms = this.clampTimelineMs(secClamped * 1000);
        this.playbackComplete = false;
        const i = this.findChunkIndex(ms);

        console.log(LOG_PREFIX, "seek", {
            requestedSec: second,
            clampedSec: secClamped,
            ms,
            chunkIndex: i,
        });

        this.stopAllSources();
        this.isAudioRunning = false;
        if (this.audioCtx && this.audioCtx.state === "running") {
            try {
                await this.audioCtx.suspend();
            } catch {
                /* ignore */
            }
        }

        this.pausedAtMs = ms;

        try {
            await this.ensureDecodedPairForStart(i);
            if (gen !== this.actionCounter) {
                return;
            }
            this.prefetchDecodeAhead(i + 2, PREFETCH_CHUNK_COUNT);

            if (wasPlaying) {
                await this.beginPlaybackFrom(ms);
                this.setState(PlayerState.PLAYING);
            } else {
                this.setState(PlayerState.PAUSED);
            }
        } catch (e) {
            if (gen !== this.actionCounter || isAbortError(e)) {
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
     * Starts or resumes playback. Decodes the chunk at the current timeline (if needed),
     * then schedules gapless audio from that position. Decode-ahead starts as soon as the
     * first chunk is ready.
     */
    async play(): Promise<void> {
        if (!this.isLoaded()) {
            console.log(LOG_PREFIX, "cannot play: not loaded");
            return;
        }
        if (this._state === PlayerState.ERROR) {
            console.log(LOG_PREFIX, "cannot play: ERR");
            return;
        }
        if (this.isAudioRunning && this._state === PlayerState.PLAYING) {
            return;
        }
        const gen = this.actionCounter;
        this.playbackComplete = false;

        const timelineMs = this.getCurrentTimeMs();

        try {
            const i = this.findChunkIndex(timelineMs);
            await this.ensureDecodedPairForStart(i);
            if (gen !== this.actionCounter) {
                return;
            }
            this.prefetchDecodeAhead(i + 2, PREFETCH_CHUNK_COUNT);

            await this.beginPlaybackFrom(timelineMs);
            if (gen !== this.actionCounter) {
                return;
            }
            this.setState(PlayerState.PLAYING);
            console.log(LOG_PREFIX, "play: playing");
        } catch (e) {
            if (gen !== this.actionCounter || isAbortError(e)) {
                return;
            }
            this.setState(PlayerState.ERROR);
            console.warn(LOG_PREFIX, "play: error", e);
        }
    }

    /** Pauses playback and updates state when we were `playing`. */
    pause(): void {
        if (!this.audioCtx) {
            return;
        }
        if (this.isAudioRunning) {
            const t = this.playStartTimelineMs + (this.audioCtx.currentTime - this.playStartCtxTime) * 1000;
            this.pausedAtMs = this.clampTimelineMs(t);
        }
        this.stopAllSources();
        this.isAudioRunning = false;
        void this.audioCtx.suspend().catch(() => { });
        if (this._state === PlayerState.PLAYING) {
            this.setState(PlayerState.PAUSED);
        }
        console.log(LOG_PREFIX, "pause");
    }

    /**
     * Tears down listeners and revokes blob URLs. Do not use this instance after calling.
     */
    destroy(): void {
        this.actionCounter += 1;
        this.stopAllSources();
        this.isAudioRunning = false;
        this.clearDecodeCaches();
        void this.audioCtx?.close().catch(() => { });
        this.audioCtx = null;
        this.gainNode = null;
        this.songMeta = null;
        this.playbackComplete = false;
        this.pausedAtMs = 0;
        console.log(LOG_PREFIX, "destroy");
    }
}
