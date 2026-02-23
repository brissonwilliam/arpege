<script lang="ts">
    import "./app.css";
    import Sidebar from "./Sidebar.svelte";
    import Footer from "./Footer.svelte";
    import Content from "./Content.svelte";
</script>

<div class="layout">
    <Sidebar />
    <div class="content">
        <p class="read-the-docs">
            Click on the Vite and Svelte logos to learn more
        </p>
    </div>
</div>
<Footer />
