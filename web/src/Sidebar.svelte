<script lang="ts">
    import "./app.css";
    import IconBurger from "@iconify-svelte/iconamoon/menu-burger-horizontal";
    import IconFolderMusic from "@iconify-svelte/iconamoon/folder-music-fill";
    import IconSong from "@iconify-svelte/iconamoon/music-1-light";
    import IconAlbum from "@iconify-svelte/iconamoon/music-album-light";
    import IconArtist from "@iconify-svelte/iconamoon/music-artist-duotone";
</script>

<div class="sidebar" >
    <div class="flex flex-row items-center">
        <h2>Arpege</h2>
    </div>
    <div class="mt-5">
        <div class="navrow hover:border-1">
            <IconSong height="1.5em" />
            <p class="">Songs</p>
        </div>
        <div class="navrow hover:border-1">
            <IconAlbum height="1.5em" />
            <p class="">Albums</p>
        </div>
        <div class="navrow hover:border-1">
            <IconArtist height="1.5em" />
            <p class="">Artists</p>
        </div>
        <div class="navrow hover:border-1">
            <IconFolderMusic height="1.5em" />
            <p class="">Playlists</p>
        </div>
    </div>
</div>

<style>
    .navrow {
        display: flex;
        align-items: center;
        gap: 16px;
        border-radius: 12px;
        padding: 8px;
    }
</style>
