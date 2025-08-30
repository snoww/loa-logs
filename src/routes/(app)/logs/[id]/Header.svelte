<script lang="ts">
  import { page } from "$app/state";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import BossOnlyDamage from "$lib/components/BossOnlyDamage.svelte";
  import { raidGates } from "$lib/constants/encounters";
  import { IconArrowLeft, IconStar } from "$lib/icons";
  import type { Encounter } from "$lib/types";
  import { formatTimestamp, getBossHpBars } from "$lib/utils";
  import { invoke } from "@tauri-apps/api/core";

  let { encounter }: { encounter: Encounter } = $props();
  let raidGate = $derived(raidGates[encounter.currentBossName]);

  let bossHpBars = $derived.by(() => {
    let boss = encounter.entities[encounter.currentBossName];
    if (boss) {
      return Math.ceil((boss.currentHp / boss.maxHp) * getBossHpBars(boss.name, boss.maxHp));
    }
    return undefined;
  });

  let fav = $state(encounter.favorite);

  async function toggleFavorite() {
    await invoke("toggle_encounter_favorite", { id: Number(page.params.id) });
    fav = !fav;
  }
</script>

{#snippet badge(text: string)}
  <p class="rounded-sm bg-neutral-700/80 px-2 py-0.5">{text}</p>
{/snippet}

<div class="sticky top-0 z-20 bg-neutral-900/70 px-6 shadow-md drop-shadow-lg backdrop-blur-lg">
  <div class="h-18 mx-auto flex max-w-[180rem] items-center">
    <div class="flex flex-col px-1 py-4">
      <div class="flex gap-2 overflow-y-auto text-nowrap py-1 text-xs">
        <a
          href="/logs"
          class="bg-accent-500/70 hover:bg-accent-500/80 flex items-center gap-1 rounded-sm py-0.5 pl-1 pr-2"
        >
          <IconArrowLeft class="shrink-0" />
          Back
        </a>
        {#if raidGate}
          {@render badge(raidGate)}
        {/if}
        {#if encounter.difficulty}
          <p
            class="rounded-sm bg-neutral-700/80 px-2 py-0.5"
            class:text-yellow-300={encounter.difficulty === "Hard"}
            class:text-amber-600={encounter.difficulty === "Inferno" ||
              encounter.difficulty === "Challenge" ||
              encounter.difficulty === "Trial"}
            class:text-cyan-400={encounter.difficulty === "Solo"}
            class:text-purple-500={encounter.difficulty === "Extreme" || encounter.difficulty === "The First"}
          >
            {encounter.difficulty}
          </p>
        {/if}

        {#if !encounter.cleared && bossHpBars}
          {@render badge(`Wipe - ${bossHpBars}x`)}
        {/if}
        {@render badge(formatTimestamp(encounter.fightStart))}
      </div>

      <div class="mt-1">
        <h1 class="flex items-center gap-1 text-xl font-semibold tracking-tight max-md:pr-2">
          {#if encounter.bossOnlyDamage}
            <BossOnlyDamage />
          {/if}
          <button class="group" onclick={toggleFavorite}>
            <QuickTooltip tooltip={fav ? "Favorite Encounter" : "Add to favorites"}>
              <IconStar class="size-5 shrink-0 group-hover:text-yellow-400 {fav ? 'text-yellow-400' : ''}" />
            </QuickTooltip>
          </button>
          <span class:text-lime-400={encounter.cleared}>#{page.params.id.toLocaleString()}: </span>
          {encounter.currentBossName || "No Boss"}
        </h1>
      </div>
    </div>
  </div>
</div>
