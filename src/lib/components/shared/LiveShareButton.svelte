<script lang="ts">
    import { menuTooltip } from "$lib/utils/tooltip";
    import { liveConnectionListeningStore, startHosting, stopHosting } from "$lib/utils/live";

    let working = $state(false);

    async function beginLiveSharing() {
        if (working) return;

        working = true;
        const id = await startHosting();
        navigator.clipboard.writeText(`https://live.lostark.bible/${id}`);
        working = false;
    }

    async function stopLiveSharing() {
        if (working) return;

        working = true;
        await stopHosting();
        working = false;
    }
</script>

{#if $liveConnectionListeningStore}
    <button use:menuTooltip={{ content: "Stop Live Sharing" }} onclick={stopLiveSharing} aria-label="Stop Live Sharing">
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="size-4.5 text-gray-400 transition hover:text-gray-50"
            class:opacity-40={working}
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="3"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><path d="M13 3H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-3" /><path d="M8 21h8" /><path
                d="M12 17v4" /><path d="m22 3-5 5" /><path d="m17 3 5 5" /></svg>
    </button>
{:else}
    <button
        use:menuTooltip={{ content: "Enable Live Sharing" }}
        onclick={beginLiveSharing}
        aria-label="Enable Live Sharing">
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="size-4.5 text-gray-400 transition hover:text-gray-50"
            class:opacity-40={working}
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="3"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><path d="M13 3H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-3" /><path d="M8 21h8" /><path
                d="M12 17v4" /><path d="m17 8 5-5" /><path d="M17 3h5v5" /></svg>
    </button>
{/if}
