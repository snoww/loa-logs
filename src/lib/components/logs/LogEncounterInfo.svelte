<script lang="ts">
    import { takingScreenshot } from "$lib/utils/stores";
    import { getVersion } from "@tauri-apps/api/app";

    export let bossName: string;
    export let encounterDuration: string;
    export let totalDamageDealt: number;
    export let dps: number;
</script>

{#if $takingScreenshot}
<div class="flex justify-between items-center px-1">
    <div class="font-bold">
        {bossName}
    </div>
    {#await getVersion() then version}
    <div class="">
        LOA Logs v{version}
    </div>
    {/await}
</div>
{/if}
<div class="px-1 text-sm" class:pb-2={$takingScreenshot} id="header">
    <div class="flex justify-between items-center">
        <div class="flex space-x-2">
            <div>
                {encounterDuration}
            </div>
            <div class="flex space-x-1 tracking-tighter text-gray-300">
                <div>
                    Total DMG:
                </div>
                <div class="text-white">
                    {totalDamageDealt.toLocaleString()}
                </div>
            </div>
            <div class="flex space-x-1 tracking-tighter text-gray-300">
                <div>
                    Total DPS:
                </div>
                <div class="text-white">
                    {dps.toLocaleString("en", { minimumFractionDigits: 0, maximumFractionDigits: 0 })}
                </div>
            </div>
        </div>
        {#if $takingScreenshot}
        <div class="font-mono text-xs">
            {"github.com/snoww/loa-logs"}
        </div>
        {/if}
    </div>
</div>