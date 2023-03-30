<script lang="ts">
    import { getVersion } from '@tauri-apps/api/app';

    export let encounterDuration: string;
    export let totalDamageDealt: number;
    export let dps: number;
</script>

<div class="px-2 py-1 bg-zinc-900/[.6] text-sm">
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
        <div data-tauri-drag-region>
            <div class="flex items-end">
                <div data-tauri-drag-region>
                    LOA Logs
                </div>
                <div data-tauri-drag-region class="ml-1 text-xs text-gray-500">
                    {#await getVersion()}
                    v
                    {:then version}
                    v{version}
                    {/await}
                </div>
            </div>
        </div>
    </div>
</div>