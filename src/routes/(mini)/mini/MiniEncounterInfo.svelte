<script lang="ts">
  import { percentValue } from "$lib/components/Snippets.svelte";
  import { bossHpMap } from "$lib/constants/encounters";
  import { EncounterState } from "$lib/encounter.svelte";
  import { settings } from "$lib/stores.svelte";
  import { EntityType } from "$lib/types";
  import { getBossHpBars, timestampToMinutesAndSeconds } from "$lib/utils";

  let { enc }: { enc: EncounterState } = $props();

  let durationPretty = $derived.by(() => {
    if (enc.duration <= 0) {
      return timestampToMinutesAndSeconds(0, false, false, true);
    } else {
      return timestampToMinutesAndSeconds(enc.duration, false, false, true);
    }
  });

  let boss = $derived(enc.encounter?.currentBoss);

  let bossHp = $derived(!boss || boss.currentHp < 0 ? 0 : boss.currentHp);
  let bossShield = $derived(boss?.currentShield ?? 0);

  let bossTotalBars = $derived.by(() => {
    if (boss && Object.hasOwn(bossHpMap, boss.name) && settings.app.meter.bossHpBar) {
      return getBossHpBars(boss.name, boss.maxHp);
    } else {
      return 1;
    }
  });

  let bossCurrentBars = $derived.by(() => {
    if (boss && bossTotalBars && !boss.isDead) {
      if (bossHp === boss.maxHp) {
        return bossTotalBars;
      } else {
        if (bossShield > 0) {
          return Math.round(((bossHp + bossShield) / boss.maxHp) * bossTotalBars);
        } else {
          return Math.ceil((bossHp / boss.maxHp) * bossTotalBars);
        }
      }
    } else {
      return 0;
    }
  });

  let bossCurrentPercentage = $derived.by(() => {
    if (!boss || boss.isDead || bossHp < 0) {
      return 0;
    } else {
      return (bossHp / boss.maxHp) * 100;
    }
  });
</script>

<div class="w-full text-xs tracking-tight">
  <div data-tauri-drag-region class="mx-auto flex w-80 items-center justify-between gap-2 bg-neutral-900/45 px-2 py-1">
    <div class="flex items-center gap-1 truncate">
      <div data-tauri-drag-region class="w-9">
        {durationPretty}
      </div>
      <div data-tauri-drag-region class="truncate">
        {enc.encounter?.currentBoss ? enc.encounter.currentBoss.name : "No Boss"}
      </div>
      <div data-tauri-drag-region class="text-neutral-300">
        {#if boss && settings.app.mini.bossHpBar}
          {#if bossTotalBars > 1}
            {bossCurrentBars}x
          {:else}
            {@render percentValue(bossCurrentPercentage.toFixed(0))}
          {/if}
        {/if}
      </div>
    </div>
    <div data-tauri-drag-region class="text-nowrap">
      {#if enc.timeToKill}
        TTK {enc.timeToKill}
      {:else}
        LOA Logs
      {/if}
    </div>
  </div>
</div>
