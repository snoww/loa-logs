<script lang="ts">
  import { raidGates } from "$lib/constants/encounters";
  import { screenshot, settings } from "$lib/stores.svelte.js";
  import type { Encounter } from "$lib/types";
  import { getVersion } from "@tauri-apps/api/app";
  import BossOnlyDamage from "$lib/components/BossOnlyDamage.svelte";
  import {
    abbreviateNumber,
    formatTimestampDate,
    formatTimestampTime,
    getBossHpBars,
    timestampToMinutesAndSeconds
  } from "$lib/utils";
  import { difficultyColor, middot } from "$lib/components/Snippets.svelte";

  let { encounter }: { encounter: Encounter } = $props();

  let raidGate = $derived(raidGates[encounter.currentBossName]);

  let bossHpBars = $derived.by(() => {
    let boss = encounter.entities[encounter.currentBossName];
    if (boss) {
      return Math.ceil((boss.currentHp / boss.maxHp) * getBossHpBars(boss));
    }
    return undefined;
  });

  let intermissionDuration = $derived(
    (encounter.encounterDamageStats.misc?.intermissionEnd ?? 0) -
      (encounter.encounterDamageStats.misc?.intermissionStart ?? 0)
  );
</script>

<div class="flex flex-col gap-1 px-4 tracking-tight" class:hidden={!screenshot.state}>
  <div class="flex items-center justify-between gap-1">
    <div class="flex min-w-0 items-center gap-1">
      {#if encounter.cleared}
        <p class="shrink-0 text-lime-400">[Cleared]</p>
      {/if}
      {#if !encounter.cleared && bossHpBars}
        <p class="shrink-0 text-neutral-400">[Wipe - {bossHpBars}x]</p>
      {/if}
      {#if encounter.bossOnlyDamage}
        <BossOnlyDamage />
      {/if}
      {#if encounter.difficulty}
        <p class="shrink-0">[{@render difficultyColor(encounter.difficulty)}]</p>
      {/if}
      {#if !settings.app.general.showGate && raidGate}
        <p class="shrink-0 text-sky-200">
          [{raidGate}]
        </p>
      {/if}
      <p class="min-w-0 truncate font-semibold" title={encounter.currentBossName || "No Boss"}>
        {encounter.currentBossName || "No Boss"}
      </p>
      {@render middot()}
      <p class="shrink-0 text-neutral-300">
        {formatTimestampDate(encounter.fightStart)}
      </p>
    </div>
    <div class="flex w-fit shrink-0 items-center gap-1 font-mono text-xs whitespace-nowrap">
      {#if !settings.app.general.hideLogo}
        <p>LOA Logs</p>
      {/if}
      {#await getVersion() then version}
        <p>
          v{version}
        </p>
      {/await}
    </div>
  </div>
  <div class="flex items-center whitespace-nowrap">
    <div class="flex shrink-0 items-baseline gap-1 text-neutral-300">
      <div>Duration:</div>
      <div class="text-white">
        {timestampToMinutesAndSeconds(encounter.duration)}
      </div>
      {#if intermissionDuration}
        <div class="-ml-0.5 text-xs">
          +{timestampToMinutesAndSeconds(intermissionDuration)}
        </div>
      {/if}
    </div>

    {@render middot()}

    <div class="flex shrink-0 gap-1 text-neutral-300">
      <div>Total DMG:</div>
      {#if settings.app.logs.abbreviateHeader}
        <div class="text-white">
          {abbreviateNumber(encounter.encounterDamageStats.totalDamageDealt)}
        </div>
      {:else}
        <div class="text-white">
          {encounter.encounterDamageStats.totalDamageDealt.toLocaleString()}
        </div>
      {/if}
    </div>

    {@render middot()}

    <div class="flex shrink-0 gap-1 text-neutral-300">
      <div>Total DPS:</div>
      {#if settings.app.logs.abbreviateHeader}
        <div class="text-white">
          {abbreviateNumber(encounter.encounterDamageStats.dps)}
        </div>
      {:else}
        <div class="text-white">
          {encounter.encounterDamageStats.dps.toLocaleString()}
        </div>
      {/if}
    </div>
  </div>
</div>
