<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { Player } from "./lib/player/Player";
    import { PlayerState as pstate } from "./lib/player/types";

    const p = new Player();

    const POLL_MS = 500;

    let isPlaying = false;
    let currentMs = 0;
    let durationMs = 0;
    let songTitle = "";
    let songArtist = "";
    let gainPercent = 100;
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
        songTitle = p.meta?.title ?? "";
        songArtist = p.meta?.artist ?? "";
    }

    function onGainInput() {
        p.setGainPercent(gainPercent);
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
        gainPercent = p.gainPercent;
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
        <div class="flex w-2/10 min-w-0 items-center gap-2 text-xs">
            <!-- NOW PLAYING -->
            <div class="min-w-0 text-lg flex flex-col gap-0.5">
                <span
                    class="truncate font-medium text-neutral-900 dark:text-neutral-100"
                    title={songTitle || undefined}>{songTitle || "—"}</span
                >
                {#if songArtist}
                    <div>
                        <span class="text-neutral-300" title={songArtist}
                            >{songArtist}</span
                        >
                    </div>
                {/if}
            </div>
        </div>
        <div class="w-8/12">
            <!-- CONTROL BUTTONS -->
            <div class="flex justify-center mt-2 lg:gap-7 gap-3">
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
            <!-- PROGRESS BAR -->
            <div class="mt-1 flex items-center gap-2 text-xs">
                <span class="shrink-0 tabular-nums text-neutral-500"
                    >{formatMmSs(currentMs / 1000)}</span
                >
                <div
                    class="progress-hit relative flex min-h-[36px] flex-1 cursor-pointer items-center py-3"
                    on:click={onProgressClick}
                    role="presentation"
                >
                    <div
                        class="progress-track h-2 w-full rounded-full bg-neutral-700"
                    >
                        <div
                            class="progress-fill h-full rounded-full bg-neutral-300"
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
        <div
            class="mt-4 flex w-2/10 min-w-0 flex-col items-end justify-center gap-1 text-xs"
        >
            <label class="flex w-full max-w-[11rem] items-center gap-2">
                <span
                    class="shrink-0 text-neutral-500 dark:text-neutral-400"
                    aria-hidden="true"
                >
                    {#if gainPercent > 60}
                        <!-- prettier-ignore -->
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" d="M20.803 8a40.5 40.5 0 0 1 0 8"/><path stroke-linejoin="round" d="M13 12c0-1.884-.163-3.73-.475-5.525c-.123-.704-.937-1.019-1.52-.605L8.52 7.632A2 2 0 0 1 7.363 8H5a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h2.363a2 2 0 0 1 1.157.368l2.485 1.762c.583.414 1.397.1 1.52-.605A32 32 0 0 0 13 12Z"/><path stroke-linecap="round" d="M16.877 9a36.5 36.5 0 0 1 0 6"/></g></svg>
                    {:else if gainPercent > 0}
                        <!-- prettier-ignore -->
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-width="2"><path stroke-linejoin="round" d="M13 12c0-1.884-.163-3.73-.475-5.525c-.123-.704-.937-1.019-1.52-.605L8.52 7.632A2 2 0 0 1 7.363 8H5a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h2.363a2 2 0 0 1 1.157.368l2.485 1.762c.583.414 1.397.1 1.52-.605A32 32 0 0 0 13 12Z"/><path stroke-linecap="round" d="M16.877 9a36.5 36.5 0 0 1 0 6"/></g></svg>
                    {:else}
                        <!-- prettier-ignore -->
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linejoin="round" stroke-width="2"><path d="M13 12c0-1.884-.163-3.73-.475-5.525c-.123-.704-.937-1.019-1.52-.605L8.52 7.632A2 2 0 0 1 7.363 8H5a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h2.363a2 2 0 0 1 1.157.368l2.485 1.762c.583.414 1.397.1 1.52-.605A32 32 0 0 0 13 12Z"/><path stroke-linecap="round" d="m17 10l4 4m-4 0l4-4"/></g></svg>
                    {/if}
                </span>
                <input
                    type="range"
                    min="0"
                    max="100"
                    bind:value={gainPercent}
                    on:input={onGainInput}
                    class="h-1.5 w-full min-w-0 flex-1 cursor-pointer appearance-none rounded-full bg-neutral-200 dark:bg-neutral-700 accent-neutral-600 dark:accent-neutral-300"
                    aria-valuemin="0"
                    aria-valuemax="100"
                    aria-valuenow={gainPercent}
                    aria-label="Volume"
                />
            </label>
            <span
                class="tabular-nums text-neutral-500 dark:text-neutral-400"
                aria-hidden="true">{gainPercent}%</span
            >
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
        width: 2.5rem;
        height: 2.5rem;
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
        width: 2.5rem;
        height: 2.5rem;
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
