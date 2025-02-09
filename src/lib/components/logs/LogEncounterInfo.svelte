<script lang="ts">
    import { abbreviateNumber, getBossHpBars } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import { getVersion } from "@tauri-apps/api/app";
    import DifficultyLabel from "../shared/DifficultyLabel.svelte";
    import BossOnlyDamage from "../shared/BossOnlyDamage.svelte";
    import type { Entity } from "$lib/types";

    interface Props {
        difficulty: string | undefined;
        date: string;
        encounterDuration: string;
        totalDamageDealt: number;
        dps: number;
        cleared: boolean;
        bossOnlyDamage: boolean;
        raidGate: string | undefined;
        boss: Entity;
    }

    let { difficulty, date, encounterDuration, totalDamageDealt, dps, cleared, bossOnlyDamage, raidGate, boss }: Props =
        $props();

    let bossHpBars: number | undefined = $state();

    if (boss) {
        let bossMaxHpBars = getBossHpBars(boss.name, boss.maxHp);
        bossHpBars = Math.ceil((boss.currentHp / boss.maxHp) * bossMaxHpBars);
    }
</script>

{#if $takingScreenshot}
    <div class="flex items-center justify-between px-1 tracking-tight">
        <div>
            {#if cleared}
                <span class="text-lime-400">[Cleared]</span>
            {:else if !cleared && bossHpBars}
                <span class="text-gray-400">[Wipe - {bossHpBars}x]</span>
            {/if}
            {#if bossOnlyDamage}
                <BossOnlyDamage />
            {/if}
            <span class="font-medium">
                {#if $settings.general.showDifficulty && difficulty}
                    <DifficultyLabel {difficulty} />
                    {#if $settings.general.showGate && raidGate}
                        <span class="text-sky-200">[{raidGate}]</span>
                    {/if}
                    {boss.name}
                {:else}
                    {#if $settings.general.showGate && raidGate}
                        <span class="text-sky-200">[{raidGate}]</span>
                    {/if}
                    {boss.name}
                {/if}
            </span><span class="ml-2 font-mono text-xs">{date}</span>
        </div>
        {#await getVersion() then version}
            {#if !$settings.general.hideLogo}
                <div class="">
                    LOA Logs v{version}
                </div>
            {:else}
                <div class="font-mono text-xs">
                    v{version}
                </div>
            {/if}
        {/await}
    </div>
{/if}
<div class="px-1 text-sm" class:pb-2={$takingScreenshot} id="header">
    <div class="flex items-center justify-between">
        <div class="flex space-x-2">
            <div>
                {encounterDuration}
            </div>
            <div class="flex space-x-1 tracking-tighter text-gray-300">
                <div>Total DMG:</div>
                {#if $settings.logs.abbreviateHeader}
                    <div class="text-white">
                        {abbreviateNumber(totalDamageDealt)}
                    </div>
                {:else}
                    <div class="text-white">
                        {totalDamageDealt.toLocaleString()}
                    </div>
                {/if}
            </div>
            <div class="flex space-x-1 tracking-tighter text-gray-300">
                <div>Total DPS:</div>
                {#if $settings.logs.abbreviateHeader}
                    <div class="text-white">
                        {abbreviateNumber(dps)}
                    </div>
                {:else}
                    <div class="text-white">
                        {dps.toLocaleString(undefined, {
                            minimumFractionDigits: 0,
                            maximumFractionDigits: 0
                        })}
                    </div>
                {/if}
            </div>
        </div>
        {#if $takingScreenshot && !$settings.general.hideLogo}
            <div class="font-mono text-xs">
                {"github.com/snoww/loa-logs"}
            </div>
        {/if}
    </div>
</div>
