<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { MusicPlayerStore } from "../lib/player/MusicPlayerStore.svelte";

    const player = new MusicPlayerStore();
    let audioEl = $state<HTMLAudioElement | null>(null);

    onMount(async () => {
        if (audioEl) await player.init(audioEl);
    });

    onDestroy(() => player.destroy());

    function formatTime(ms: number): string {
        const total = Math.floor(ms / 1000);
        const m = Math.floor(total / 60);
        const s = total % 60;
        return `${m}:${s.toString().padStart(2, "0")}`;
    }

    function handleSeek(e: MouseEvent) {
        const bar = e.currentTarget as HTMLElement;
        const rect = bar.getBoundingClientRect();
        const fraction = Math.max(
            0,
            Math.min(1, (e.clientX - rect.left) / rect.width),
        );
        player.seek(fraction * player.durationMs);
    }
</script>

<!-- Hidden native audio element; MSE drives it entirely -->
<audio bind:this={audioEl} style="display:none"></audio>
<main>
    <div class="player" class:loading={player.state === "loading"}>
        <div class="meta">
            <div class="disc" class:spinning={player.state === "playing"}>
                <div class="disc-inner"></div>
            </div>
            <div class="text">
                {#if player.meta}
                    <p class="title">{player.meta.title}</p>
                    <p class="artist">{player.meta.artist}</p>
                {:else if player.state === "error"}
                    <p class="title error">{player.error}</p>
                {:else}
                    <p class="title muted">Loading…</p>
                {/if}
            </div>
        </div>

        <!-- Progress bar -->
        <div class="progress-wrap">
            <span class="time">{formatTime(player.currentMs)}</span>
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="progress-bar" onclick={handleSeek}>
                <div
                    class="progress-fill"
                    style="width: {player.progressFraction * 100}%"
                ></div>
                <div
                    class="progress-thumb"
                    style="left: {player.progressFraction * 100}%"
                ></div>
            </div>
            <span class="time">{formatTime(player.durationMs)}</span>
        </div>

        <!-- Controls -->
        <div class="controls">
            <button
                class="play-btn"
                onclick={() => player.togglePlayPause()}
                disabled={player.state === "idle" ||
                    player.state === "loading" ||
                    player.state === "error"}
                aria-label={player.state === "playing" ? "Pause" : "Play"}
            >
                {#if player.state === "loading"}
                    <span class="spinner"></span>
                {:else if player.state === "playing"}
                    <!-- Pause icon -->
                    <svg
                        viewBox="0 0 24 24"
                        fill="currentColor"
                        width="28"
                        height="28"
                    >
                        <rect x="5" y="3" width="4" height="18" rx="1" />
                        <rect x="15" y="3" width="4" height="18" rx="1" />
                    </svg>
                {:else}
                    <!-- Play icon -->
                    <svg
                        viewBox="0 0 24 24"
                        fill="currentColor"
                        width="28"
                        height="28"
                    >
                        <path
                            d="M6 3.765C6 2.477 7.388 1.688 8.5 2.353l11.113 6.735c1.087.659 1.087 2.205 0 2.864L8.5 18.647C7.388 19.312 6 18.523 6 17.235V3.765z"
                        />
                    </svg>
                {/if}
            </button>
        </div>
    </div>
</main>

<style>

    main {
        width: 100%;
        min-height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        position: relative;
        overflow: hidden;
    }

    /* Grain overlay for texture */
    .player {
        position: relative;
        z-index: 1;
        width: min(420px, 92vw);
        background: #111;
        border: 1px solid #222;
        border-radius: 2px;
        padding: 2.5rem 2rem 2rem;
        display: flex;
        flex-direction: column;
        gap: 2rem;
        transition: opacity 0.3s;
    }

    .player.loading {
        opacity: 0.6;
    }

    /* Meta row */
    .meta {
        display: flex;
        align-items: center;
        gap: 1.25rem;
    }

    /* Vinyl disc */
    .disc {
        flex-shrink: 0;
        width: 72px;
        height: 72px;
        border-radius: 50%;
        background: conic-gradient(
            from 0deg,
            #1c1c1c 0deg 10deg,
            #141414 10deg 20deg,
            #1c1c1c 20deg 30deg,
            #141414 30deg 40deg,
            #1c1c1c 40deg 50deg,
            #141414 50deg 60deg,
            #1c1c1c 60deg 70deg,
            #141414 70deg 80deg,
            #1c1c1c 80deg 90deg,
            #141414 90deg 100deg,
            #1c1c1c 100deg 110deg,
            #141414 110deg 120deg,
            #1c1c1c 120deg 130deg,
            #141414 130deg 140deg,
            #1c1c1c 140deg 150deg,
            #141414 150deg 160deg,
            #1c1c1c 160deg 170deg,
            #141414 170deg 180deg,
            #1c1c1c 180deg 190deg,
            #141414 190deg 200deg,
            #1c1c1c 200deg 210deg,
            #141414 210deg 220deg,
            #1c1c1c 220deg 230deg,
            #141414 230deg 240deg,
            #1c1c1c 240deg 250deg,
            #141414 250deg 260deg,
            #1c1c1c 260deg 270deg,
            #141414 270deg 280deg,
            #1c1c1c 280deg 290deg,
            #141414 290deg 300deg,
            #1c1c1c 300deg 310deg,
            #141414 310deg 320deg,
            #1c1c1c 320deg 330deg,
            #141414 330deg 340deg,
            #1c1c1c 340deg 350deg,
            #141414 350deg 360deg
        );
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
        transition: transform 0.4s ease;
    }

    .disc.spinning {
        animation: spin 4s linear infinite;
    }

    .disc-inner {
        width: 22px;
        height: 22px;
        border-radius: 50%;
        background: #0a0a0a;
        border: 1px solid #2a2a2a;
        box-shadow: inset 0 0 6px rgba(255, 255, 255, 0.04);
    }

    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    .text {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        overflow: hidden;
    }

    .title {
        font-family: "DM Serif Display", serif;
        font-size: 1.35rem;
        color: #f0ede8;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        line-height: 1.2;
    }

    .title.muted {
        color: #444;
    }
    .title.error {
        color: #c0392b;
        font-size: 0.85rem;
    }

    .artist {
        font-size: 0.7rem;
        color: #555;
        letter-spacing: 0.12em;
        text-transform: uppercase;
    }

    /* Progress */
    .progress-wrap {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .time {
        font-size: 0.65rem;
        color: #444;
        letter-spacing: 0.05em;
        min-width: 2.8ch;
        white-space: nowrap;
    }

    .progress-bar {
        position: relative;
        flex: 1;
        height: 3px;
        background: #222;
        border-radius: 2px;
        cursor: pointer;
    }

    .progress-bar:hover .progress-thumb {
        opacity: 1;
        transform: translateX(-50%) scale(1);
    }

    .progress-fill {
        position: absolute;
        left: 0;
        top: 0;
        height: 100%;
        background: #f0ede8;
        border-radius: 2px;
        transition: width 0.1s linear;
        pointer-events: none;
    }

    .progress-thumb {
        position: absolute;
        top: 50%;
        width: 10px;
        height: 10px;
        background: #f0ede8;
        border-radius: 50%;
        transform: translateX(-50%) translateY(-50%) scale(0.6);
        opacity: 0;
        transition:
            opacity 0.2s,
            transform 0.2s;
        pointer-events: none;
    }

    /* Controls */
    .controls {
        display: flex;
        justify-content: center;
    }

    .play-btn {
        width: 56px;
        height: 56px;
        border-radius: 50%;
        border: 1px solid #2c2c2c;
        background: #161616;
        color: #f0ede8;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition:
            background 0.15s,
            border-color 0.15s,
            transform 0.1s;
    }

    .play-btn:hover:not(:disabled) {
        background: #1e1e1e;
        border-color: #3a3a3a;
        transform: scale(1.05);
    }

    .play-btn:active:not(:disabled) {
        transform: scale(0.97);
    }
    .play-btn:disabled {
        opacity: 0.3;
        cursor: not-allowed;
    }

    /* Loading spinner */
    .spinner {
        width: 20px;
        height: 20px;
        border: 2px solid #333;
        border-top-color: #f0ede8;
        border-radius: 50%;
        animation: rotate 0.8s linear infinite;
        display: inline-block;
    }

    @keyframes rotate {
        to {
            transform: rotate(360deg);
        }
    }
</style>
