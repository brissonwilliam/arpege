<script>
</script>

<div class="content content-dim">
    <div class="w-full px-3 table content-dim">
        <div class="item-row" style="min-height: var(--content-overflow-top)"></div>
        {#each Array(50) as _, i}
            <div class="item-row">
                <p><span class="iconamoon--player-play-fill"></span></p>
                <p>[img]</p>
                <p>Someby That I Used To Know</p>
                <p>Goty</p>
                <p></p>
                <p>2026-02-01</p>
            </div>
            <div class="item-row">
                <p><span class="iconamoon--player-play-fill"></span></p>
                <p>[img]</p>
                <p>Someby That I Used To Know</p>
                <p>Pierce The Veil</p>
                <p>Collide with the sky</p>
                <p>2026-02-01</p>
            </div>
        {/each}
        <div class="item-row" style="min-height: var(--content-overflow-bottom)"></div>
    </div>
</div>

<style>
    .table {
        display: block;
        overflow-y: scroll;
    }

    .item-row {
        display: flex;
        border-bottom: 1px solid var(--border-low);
        height: var(--song-item-height);
        vertical-align: middle;
        align-items: center;
        gap: 20px;
        margin: 0;
    }

    .item-row:hover {
        background-color: var(--bg-high);
        border-radius: 15px;
    }

    table tr td:first-child {
        padding-left: 12px;
        min-width: 30px;
        max-width: 30px;
    }
    table tr td:last-child {
        padding-right: 12px;
    }
    .items tr:hover .iconamoon--player-play-fill {
        display: flex;
    }
    .iconamoon--player-play-fill {
        align-items: middle;
        display: none;
        width: 24px;
        height: 24px;
        --svg: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath fill='%23000' fill-rule='evenodd' d='M19.5 14.598c2-1.155 2-4.041 0-5.196l-9-5.196C8.5 3.05 6 4.494 6 6.804v10.392c0 2.31 2.5 3.753 4.5 2.598z' clip-rule='evenodd'/%3E%3C/svg%3E");
        background-color: currentColor;
        -webkit-mask-image: var(--svg);
        mask-image: var(--svg);
        -webkit-mask-repeat: no-repeat;
        mask-repeat: no-repeat;
        -webkit-mask-size: 100% 100%;
        mask-size: 100% 100%;
    }
</style>
