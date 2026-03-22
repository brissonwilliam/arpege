<script>

</script>

<div class="footer">
    <div class="player">
        <div class="w-2/10 text-xs">
            <p>Now playing (nice icon + song name)</p>
        </div>
        <div class="w-8/12">
            <div class="flex justify-center lg:gap-7 gap-3">
                <div class="player-btn start-btn"></div>
                <div class="player-btn play-btn">
                    <div class="play-btn"></div>
                    <div class="pause-btn"></div>
                </div>
                <div class="player-btn end-btn"></div>
            </div>
            <div class="mt-3 text-center text-xs">
                |----------PROGRESS BAR-------|
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
