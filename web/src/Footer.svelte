<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { Player } from "./lib/player/Player";
    import { PlayerState as pstate } from "./lib/player/types";

    const p = new Player();

    const POLL_MS = 250;

    let isPlaying = false;
    let currentMs = 0;
    let durationMs = 0;
    let pollTimer: ReturnType<typeof setInterval> | null = null;

    function formatMmSs(totalSeconds: number): string {
        const s = Math.floor(totalSeconds);
        const m = Math.floor(s / 60);
        const r = s % 60;
        return `${m}:${r.toString().padStart(2, "0")}`;
    }

    function refreshFromPlayer() {
        currentMs = p.getCurrentTimeMs();
        durationMs = p.meta?.duration_ms ?? 0;
        isPlaying = p.isPlaying;
    }

    function onProgressClick(e: MouseEvent & { currentTarget: HTMLElement }) {
        const rect = e.currentTarget.getBoundingClientRect();
        const w = rect.width;
        const dm = p.meta?.duration_ms ?? 0;
        if (w <= 0 || dm <= 0) {
            return;
        }
        const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / w));
        const durationSec = Math.max(0, Math.round(dm / 1000));
        const targetSec = Math.round(ratio * durationSec);
        void p.seek(targetSec);
    }

    onMount(async () => {
        await p.load();
        refreshFromPlayer();
        pollTimer = setInterval(refreshFromPlayer, POLL_MS);
    });

    onDestroy(() => {
        if (pollTimer !== null) {
            clearInterval(pollTimer);
            pollTimer = null;
        }
    });

    async function togglePlayPause() {
        switch (p.state) {
            case pstate.PLAYING:
                p.pause();
                isPlaying = false;
                break;
            case pstate.PAUSED:
                await p.play();
                isPlaying = true;
                break;
            case pstate.ERROR:
                await p.load();
                await p.play();
                isPlaying = true;
                break;
        }
        refreshFromPlayer();
    }
</script>

<div class="footer">
    <div class="player">
        <div class="w-2/10 text-xs">
            <p>Now playing (nice icon + song name)</p>
        </div>
        <div class="w-8/12">
            <div class="flex justify-center lg:gap-7 gap-3">
                <div class="player-btn start-btn"></div>
                <button
                    on:click={togglePlayPause}
                    aria-label="play-pause"
                    class={!isPlaying
                        ? "player-btn play-btn"
                        : "player-btn pause-btn"}
                >
                </button>
                <div class="player-btn end-btn"></div>
            </div>
            <div class="mt-3 flex items-center gap-2 text-xs">
                <span
                    class="shrink-0 tabular-nums text-neutral-500 dark:text-neutral-400"
                    >{formatMmSs(currentMs / 1000)}</span
                >
                <div
                    class="progress-hit relative flex min-h-[36px] flex-1 cursor-pointer items-center py-3"
                    on:click={onProgressClick}
                    role="presentation"
                >
                    <div
                        class="progress-track h-2 w-full rounded-full bg-neutral-200 dark:bg-neutral-700"
                    >
                        <div
                            class="progress-fill h-full rounded-full bg-neutral-600 dark:bg-neutral-300"
                            style="width: {durationMs > 0
                                ? Math.min(100, (currentMs / durationMs) * 100)
                                : 0}%"
                        ></div>
                    </div>
                </div>
                <span
                    class="shrink-0 tabular-nums text-neutral-500 dark:text-neutral-400"
                    >{formatMmSs(durationMs / 1000)}</span
                >
            </div>
        </div>
        <div class="w-2/10 text-xs text-right">
            <p>Volume and shite</p>
        </div>
    </div>
</div>

<style>
    .player-btn {
        width: var(--icon-size);
        height: var(--icon-size);
        display: inline-block;
        -webkit-mask-repeat: no-repeat;
        mask-repeat: no-repeat;
        -webkit-mask-size: 100% 100%;
        mask-size: 100% 100%;
        width: 2.2rem;
        height: 2.2rem;
        background-color: currentColor;
        padding: 5px 35px;
    }
    .player-btn {
        cursor: pointer;
    }

    .start-btn {
        --svg: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cg fill='none' stroke='%23000' stroke-linejoin='round' stroke-width='2'%3E%3Cpath d='M7 13.732c-1.333-.77-1.333-2.694 0-3.464l9-5.196c1.333-.77 3 .192 3 1.732v10.392c0 1.54-1.667 2.502-3 1.732z'/%3E%3Cpath stroke-linecap='round' d='M4 19V5'/%3E%3C/g%3E%3C/svg%3E");
        -webkit-mask-image: var(--svg);
        mask-image: var(--svg);
    }
    .start-btn:hover {
        --svg: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath fill='%23000' fill-rule='evenodd' d='m15.5 19.794l-9-5.196c-2-1.155-2-4.041 0-5.196l9-5.196c2-1.155 4.5.288 4.5 2.598v10.392c0 2.31-2.5 3.753-4.5 2.598M3 19a1 1 0 1 0 2 0V5a1 1 0 1 0-2 0z' clip-rule='evenodd'/%3E%3C/svg%3E");
        -webkit-mask-image: var(--svg);
        mask-image: var(--svg);
    }

    .play-btn {
        width: 2.8rem;
        height: 2.8rem;
        --svg: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath fill='none' stroke='%23000' stroke-linejoin='round' stroke-width='2.5' d='M19 10.268c1.333.77 1.333 2.694 0 3.464l-9 5.196c-1.333.77-3-.192-3-1.732V6.804c0-1.54 1.667-2.502 3-1.732z'/%3E%3C/svg%3E");
        -webkit-mask-image: var(--svg);
        mask-image: var(--svg);
    }
    .play-btn:hover {
        --svg: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath fill='%23000' fill-rule='evenodd' d='M19.5 14.598c2-1.155 2-4.041 0-5.196l-9-5.196C8.5 3.05 6 4.494 6 6.804v10.392c0 2.31 2.5 3.753 4.5 2.598z' clip-rule='evenodd'/%3E%3C/svg%3E");
        -webkit-mask-image: var(--svg);
        mask-image: var(--svg);
    }

    .pause-btn {
        width: 2.8rem;
        height: 2.8rem;
        --svg: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath fill='none' stroke='%23000' stroke-linejoin='round' stroke-width='2' d='M5 7a2 2 0 0 1 2-2h1a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2zm9 0a2 2 0 0 1 2-2h1a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2z'/%3E%3C/svg%3E");
        -webkit-mask-image: var(--svg);
        mask-image: var(--svg);
    }
    .pause-btn:hover {
        --svg: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath fill='%23000' fill-rule='evenodd' d='M4 7a3 3 0 0 1 3-3h1a3 3 0 0 1 3 3v10a3 3 0 0 1-3 3H7a3 3 0 0 1-3-3zm12-3a3 3 0 0 0-3 3v10a3 3 0 0 0 3 3h1a3 3 0 0 0 3-3V7a3 3 0 0 0-3-3z' clip-rule='evenodd'/%3E%3C/svg%3E");
        -webkit-mask-image: var(--svg);
        mask-image: var(--svg);
    }

    .end-btn {
        --svg: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cg fill='none' stroke='%23000' stroke-linejoin='round' stroke-width='2'%3E%3Cpath d='M17 10.268c1.333.77 1.333 2.694 0 3.464l-9 5.196c-1.333.77-3-.192-3-1.732V6.804c0-1.54 1.667-2.502 3-1.732z'/%3E%3Cpath stroke-linecap='round' d='M20 5v14'/%3E%3C/g%3E%3C/svg%3E");
        -webkit-mask-image: var(--svg);
        mask-image: var(--svg);
    }
    .end-btn:hover {
        --svg: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath fill='%23000' fill-rule='evenodd' d='M17.5 9.402c2 1.155 2 4.041 0 5.196l-9 5.196c-2 1.155-4.5-.288-4.5-2.598V6.804c0-2.31 2.5-3.753 4.5-2.598zM21 5a1 1 0 1 0-2 0v14a1 1 0 1 0 2 0z' clip-rule='evenodd'/%3E%3C/svg%3E");
        -webkit-mask-image: var(--svg);
        mask-image: var(--svg);
    }
</style>
