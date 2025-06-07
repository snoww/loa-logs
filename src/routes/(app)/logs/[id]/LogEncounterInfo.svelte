<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { screenshot, settings } from "$lib/stores.svelte.js";
  import { onMount } from "svelte";
  import { abbreviateNumber, timestampToMinutesAndSeconds } from "$lib/utils";
  import { middot } from "$lib/components/Snippets.svelte";

  let { enc }: { enc: EncounterState } = $props();

  let locale: string | undefined = $state();

  onMount(() => {
    locale = window.navigator.language;
  });
</script>

<div class="bg-black/10 px-3 py-2 text-sm" class:hidden={screenshot.state} id="header">
  <div class="flex flex-row gap-1">
    <div class="flex gap-1 text-neutral-300">
      <div>Duration:</div>
      <div class="text-white">
        {timestampToMinutesAndSeconds(enc.duration)}
      </div>
    </div>

    {@render middot()}

    <div class="flex gap-1 text-neutral-300">
      <div>Total DMG:</div>
      {#if settings.app.logs.abbreviateHeader}
        <div class="text-white">
          {abbreviateNumber(enc.totalDamageDealt)}
        </div>
      {:else}
        <div class="text-white">
          {enc.totalDamageDealt.toLocaleString()}
        </div>
      {/if}
    </div>

    {@render middot()}

    <div class="flex gap-1 text-neutral-300">
      <div>Total DPS:</div>
      {#if settings.app.logs.abbreviateHeader}
        <div class="text-white">
          {abbreviateNumber(enc.encounter!.encounterDamageStats.dps)}
        </div>
      {:else}
        <div class="text-white">
          {enc.encounter!.encounterDamageStats.dps.toLocaleString()}
        </div>
      {/if}
    </div>
  </div>
</div>
