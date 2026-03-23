import { MusicPlayer } from "./MusicPlayer";
import type { SongMeta, PlayerState } from "./types";

const HARDCODED_SONG_ID = "1";

/**
 * Svelte 5 runes-based store.
 * Uses $state for reactive primitives; the class instance itself is the store.
 * Instantiate once, pass around, bind to components.
 */
export class MusicPlayerStore {
    // Reactive state (Svelte 5 runes)
    state = $state<PlayerState>("idle");
    currentMs = $state(0);
    meta = $state<SongMeta | null>(null);
    error = $state<string | null>(null);

    private player: MusicPlayer | null = null;

    // ── Derived ─────────────────────────────────────────────────────────────────

    get durationMs(): number {
        return this.meta?.duration_ms ?? 0;
    }

    get progressFraction(): number {
        return this.durationMs > 0 ? this.currentMs / this.durationMs : 0;
    }

    // ── Lifecycle ───────────────────────────────────────────────────────────────

    /**
     * Must be called inside onMount with the <audio> element reference.
     * Fetches metadata and kicks off chunk loading.
     */
    async init(audio: HTMLAudioElement): Promise<void> {
        this.state = "loading";
        this.player = new MusicPlayer(audio);

        this.player.onStateChange = (s) => { this.state = s; };
        this.player.onTimeUpdate = (ms) => { this.currentMs = ms; };
        this.player.onError = (msg) => handleErr(this, msg);

        try {
            const res = await fetch(`/api/songs/md?s=${HARDCODED_SONG_ID}`);
            if (!res.ok) {
                throw new Error(`Metadata fetch failed: HTTP ${res.status}`);
            }
            const songMeta: SongMeta = await res.json();
            this.meta = songMeta;
            await this.player.load(songMeta);
            this.state = "paused";
        } catch (e: unknown) {
            handleErr(this, e)
        }
    }

    destroy(): void {
        this.player?.destroy();
        this.player = null;
    }

    // ── Controls ─────────────────────────────────────────────────────────────────

    play(): void {
        this.player?.play();
    }

    pause(): void {
        this.player?.pause();
    }

    togglePlayPause(): void {
        if (this.state === "playing") {
            this.pause();
            return
        }
        this.play();
    }

    seek(ms: number): void {
        this.currentMs = ms;
        this.player?.seek(ms);
    }
}

function handleErr(p: MusicPlayerStore, err: any) {
    p.state = "error";
    console.error("[MusicPlayerStore]:", err);
}
