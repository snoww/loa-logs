<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { AP, type APTypes } from "$lib/constants/classes";
  import { EFTable_ArkPassive } from "$lib/constants/EFTable_ArkPassive";
  import type { EntityState } from "$lib/entity.svelte";
  import type { ArkPassiveNode } from "$lib/types";
  import { isSupportSpec, normalizeIlvl } from "$lib/utils";

  const { state }: { state: EntityState } = $props();

  let arkPassiveSpec = $derived.by(() => {
    if (!state.entity.arkPassiveData || !state.entity.arkPassiveData.enlightenment) return "";
    for (const node of state.entity.arkPassiveData.enlightenment) {
      const specName = getSpecFromArkPassive(node);
      if (specName !== "Unknown") {
        return specName;
      }
    }
  });

  function getSpecFromArkPassive(node: ArkPassiveNode): string {
    switch (node.id) {
      case 2160000:
        return "Berserker Technique";
      case 2160010:
        return "Mayhem";
      case 2170000:
        return "Lone Knight";
      case 2170010:
        return "Combat Readiness";
      case 2180000:
        return "Rage Hammer";
      case 2180010:
        return "Gravity Training";
      case 2360000:
        return "Judgement";
      case 2360010:
        return "Blessed Aura";
      case 2450000:
        return "Punisher";
      case 2450010:
        return "Predator";
      case 2230000:
        return "Ultimate Skill: Taijutsu";
      case 2230100:
        return "Shock Training";
      case 2220000:
        return "First Intention";
      case 2220100:
        return "Esoteric Skill Enhancement";
      case 2240000:
        return "Energy Overflow";
      case 2240100:
        return "Robust Spirit";
      case 2340000:
        return "Control";
      case 2340100:
        return "Pinnacle";
      case 2470000:
        return "Brawl King Storm";
      case 2470100:
        return "Asura's Path";
      case 2390000:
        return "Esoteric Flurry";
      case 2390010:
        return "Deathblow";
      case 2300000:
        return "Barrage Enhancement";
      case 2300100:
        return "Firepower Enhancement";
      case 2290000:
        return "Enhanced Weapon";
      case 2290100:
        return "Pistoleer";
      case 2280000:
        return "Death Strike";
      case 2280100:
        return "Loyal Companion";
      case 2350000:
        return "Evolutionary Legacy";
      case 2350100:
        return "Arthetinean Skill";
      case 2380000:
        return "Peacemaker";
      case 2380100:
        return "Time to Hunt";
      case 2370000:
        return "Igniter";
      case 2370100:
        return "Reflux";
      case 2190000:
        return "Grace of the Empress";
      case 2190100:
        return "Order of the Emperor";
      case 2200000:
        return "Communication Overflow";
      case 2200100:
        return "Master Summoner";
      case 2210000:
        return "Desperate Salvation";
      case 2210100:
        return "True Courage";
      case 2270000:
        return "Demonic Impulse";
      case 2270600:
        return "Perfect Suppression";
      case 2250000:
        return "Surge";
      case 2250600:
        return "Remaining Energy";
      case 2260000:
        return "Lunar Voice";
      case 2260600:
        return "Hunger";
      case 2460000:
        return "Full Moon Harvester";
      case 2460600:
        return "Night's Edge";
      case 2320000:
        return "Wind Fury";
      case 2320600:
        return "Drizzle";
      case 2310000:
        return "Full Bloom";
      case 2310600:
        return "Recurrence";
      default:
        return "Unknown";
    }
  }
</script>

{#snippet renderTree(name: string, tree?: ArkPassiveNode[])}
  {@const [text, color] = AP[name as APTypes]}
  {#if tree && tree.length}
    <div class="text-purple-400">[{text}]</div>
    <div class="flex flex-col">
      {#each tree as node}
        {@const data = EFTable_ArkPassive[node.id]}
        {#if data}
          <div class="flex items-center gap-1">
            <div class={color}>T{data[3] + 1} {data[0]}</div>
            <span class="text-white">Lv. {node.lv}</span>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
{/snippet}

{#snippet tooltip()}
  <div class="flex flex-col">
    <p>{state.name}</p>
    {#if state.entity.combatPower}
      <p class="text-xs {isSupportSpec(state.entity.spec) ? 'text-green-400' : 'text-red-400'}">
        {normalizeIlvl(state.entity.combatPower)} Combat Power
      </p>
    {/if}
    <div class="text-xs">
      {#if state.entity.arkPassiveData && state.entity.spec}
        {#if arkPassiveSpec == state.entity.spec}
          {@render renderTree("evolution", state.entity.arkPassiveData.evolution)}
          {@render renderTree("enlightenment", state.entity.arkPassiveData.enlightenment)}
          {@render renderTree("leap", state.entity.arkPassiveData.leap)}
        {:else}
          <div class="text-violet-400">Mismatched Ark Passive Data</div>
        {/if}
      {/if}
    </div>
  </div>
{/snippet}

<QuickTooltip {tooltip} delay={500} class="truncate">
  {state.name}
</QuickTooltip>
