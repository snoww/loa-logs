<script lang="ts">
    import { getVersion } from "@tauri-apps/api/app";
</script>

<div class="text-bold bg-zinc-900/[.6] px-2 py-px">
    <div class="flex items-end">
        <div>LOA Logs</div>
        <div class="ml-1 text-sm text-gray-500">
            {#await getVersion()}
                ver
            {:then version}
                ver{version}
            {/await}
        </div>
    </div>
</div>

<div data-tauri-drag-region class="fixed top-0 h-8 w-screen"></div>
