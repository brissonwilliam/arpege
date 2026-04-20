<script lang="ts">
    import { onMount } from "svelte";
    import Search from "./Search.svelte";

    let minui = false;

    onMount(async () => {
        minui = localStorage.getItem("minui") === "1";
        apply();
    });

    function toggleMinui() {
        minui = !minui;
        localStorage.setItem("minui", minui ? "1" : "0");
        apply();
    }

    function apply() {
        document.documentElement.classList.toggle("minui", minui);
    }
</script>

<div class="header">
    <h2 class="w-3/12"></h2>
    <div class="w-6/12">
        <Search />
    </div>
    <div class="w-3/12">
        <div class="ml-5 w-2/6">
            <button class="window-pill theme-toggle" on:click={toggleMinui}> Minui </button>
        </div>
    </div>
</div>

<style>
</style>
