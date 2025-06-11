<script lang="ts">
  import { raidGates } from "$lib/constants/encounters";
  import { screenshot, settings } from "$lib/stores.svelte.js";
  import type { Encounter } from "$lib/types";
  import { getVersion } from "@tauri-apps/api/app";
  import BossOnlyDamage from "$lib/components/BossOnlyDamage.svelte";
  import { abbreviateNumber, formatTimestampDate, formatTimestampTime, getBossHpBars, timestampToMinutesAndSeconds } from "$lib/utils";
  import { middot } from "$lib/components/Snippets.svelte";

  let { encounter }: { encounter: Encounter } = $props();

  let raidGate = $derived(raidGates[encounter.currentBossName]);

  let bossHpBars = $derived.by(() => {
    let boss = encounter.entities[encounter.currentBossName];
    if (boss) {
      return Math.ceil((boss.currentHp / boss.maxHp) * getBossHpBars(boss.name, boss.maxHp));
    }
    return undefined;
  });
</script>

<div class="flex flex-col gap-1 px-4 tracking-tight" class:hidden={!screenshot.state}>
  <div class="flex items-center justify-between gap-1">
    <div class="flex items-center gap-1 truncate">
      {#if encounter.cleared}
        <p class="text-lime-400">[Cleared]</p>
      {/if}
      {#if !encounter.cleared && bossHpBars}
        <p class="text-neutral-400">[Wipe - {bossHpBars}x]</p>
      {/if}
      {#if encounter.bossOnlyDamage}
        <BossOnlyDamage />
      {/if}
      {#if encounter.difficulty}
        <p
          class:text-yellow-300={encounter.difficulty === "Hard"}
          class:text-amber-600={encounter.difficulty === "Inferno" ||
            encounter.difficulty === "Challenge" ||
            encounter.difficulty === "Trial"}
          class:text-cyan-400={encounter.difficulty === "Solo"}
          class:text-purple-500={encounter.difficulty === "Extreme" || encounter.difficulty === "The First"}
        >
          [{encounter.difficulty}]
        </p>
      {/if}
      {#if !settings.app.general.showGate && raidGate}
        <p class="text-sky-200">
          [{raidGate}]
        </p>
      {/if}
      <p class="font-semibold truncate">
        {encounter.currentBossName || "No Boss"}
      </p>
      {@render middot()}
      <p class="text-neutral-300">
        {formatTimestampDate(encounter.fightStart)}
      </p>
    </div>
    <div class="flex items-center gap-1 font-mono text-xs">
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
  <div class="flex items-center">
    <div class="flex gap-1 text-neutral-300">
      <div>Duration:</div>
      <div class="text-white">
        {timestampToMinutesAndSeconds(encounter.duration)}
      </div>
    </div>

    {@render middot()}

    <div class="flex gap-1 text-neutral-300">
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

    <div class="flex gap-1 text-neutral-300">
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
