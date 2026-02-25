<script>
    import IconPlay from "@iconify-svelte/iconamoon/play-circle-fill";
    import IconStart from "@iconify-svelte/iconamoon/player-start-light";
    import IconEnd from "@iconify-svelte/iconamoon/player-end-light";
</script>

<div class="footer">
    <div class="player flex justify-between items-center">
        <div class="w-2/10 text-xs">
            <p>Now playing (nice icon + song name)</p>
        </div>
        <div class="w-8/12">
            <div class="flex justify-center gap-8">
                <IconStart class="pt-1" height="2.2rem" />
                <IconPlay height="2.4rem" />
                <IconEnd class="pt-1" height="2.2rem" />
            </div>
            <div class="mt-3 text-center text-xs">|----------PROGRESS BAR-------|</div>
        </div>
        <div class="w-2/10 text-xs text-right">
            <p>Volume and shite</p>
        </div>
    </div>
</div>
