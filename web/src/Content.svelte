<script>
    import Search from "./Search.svelte";
</script>

<div class="content">
    <Search />
    <br>
    <h2 class="">Song Library</h2>
    <br>
    <div class="w-full px-3">
        <table class="w-full text-left">
            <thead class="items-head">
                <tr>
                    <!-- play icon -->
                    <th></th>
                    <!-- album start -->
                    <th style="min-width: 25px">Title</th>
                    <!-- song name -->
                    <th style="min-width: 300px"></th>
                    <th style="min-width: 30px">Artist</th>
                    <th style="min-width: 50px">Album</th>
                    <th>Date added</th>
                </tr>
            </thead>
            <tbody class="items">
                <tr>
                    <td><span class="iconamoon--player-play-fill"></span></td>
                    <td>[img]</td>
                    <td>Someby That I Used To Know</td>
                    <td>Goty</td>
                    <td></td>
                    <td>2026-02-01</td>
                </tr>
                <tr>
                    <td><span class="iconamoon--player-play-fill"></span></td>
                    <td>[img]</td>
                    <td>Someby That I Used To Know</td>
                    <td>Pierce The Veil</td>
                    <td>Collide with the sky</td>
                    <td>2026-02-01</td>
                </tr>
            </tbody>
        </table>
    </div>
</div>

<style>
    .items-head tr {
        height: 2.2rem;
    }
    .items tr {
        border-top: 1px solid var(--border-low);
        height: 2.5rem;
    }
    .items tr:hover {
        background-color: var(--bg-high);
    }
    .items tr:hover td:first-child {
        border-top-left-radius: 10px;
        border-bottom-left-radius: 10px;
    }
    .items tr:hover td:last-child {
        border-top-right-radius: 14px;
        border-bottom-right-radius: 14px;
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
