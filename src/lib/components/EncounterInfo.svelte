<script lang="ts">
    import { appWindow } from "@tauri-apps/api/window";

    export let encounterDuration: string;
    export let totalDamageDealt: number;
    export let dps: number;

</script>

<div class="px-2 py-1 bg-zinc-900/[.6] text-sm fixed top-0 left-0 h-7 w-full" id="header">
    <div data-tauri-drag-region class="flex justify-between">
        <div data-tauri-drag-region class="flex space-x-2">
            <div data-tauri-drag-region>
                {encounterDuration}
            </div>
            <div data-tauri-drag-region class="flex space-x-1 tracking-tighter text-gray-400">
                <div data-tauri-drag-region>
                    Total DMG
                </div>
                <div data-tauri-drag-region>
                    {totalDamageDealt.toLocaleString()}
                </div>
            </div>
            <div class="flex space-x-1 tracking-tighter text-gray-400">
                <div data-tauri-drag-region>
                    Total DPS
                </div>
                <div data-tauri-drag-region >
                    {dps.toLocaleString("en", { minimumFractionDigits: 0, maximumFractionDigits: 0 })}
                </div>
            </div>
        </div>
        <div data-tauri-drag-region class="flex space-x-1 items-center">
            <button on:click={() => appWindow.hide()}>
                <svg class="fill-gray-300 hover:fill-gray-50" xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
                    <path d="M20 14H4v-4h16"/>
                </svg>
            </button>
        </div>
    </div>
</div>