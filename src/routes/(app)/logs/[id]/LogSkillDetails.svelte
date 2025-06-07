<script lang="ts">
  import { IconChevronLeft, IconChevronRight } from "$lib/icons";
  import { focusedCast } from "$lib/stores.svelte.js";
  import type { EncounterDamageStats, Entity } from "$lib/types";
  import { getSkillCastBuffs, getSkillCastSupportBuffs } from "$lib/utils/buffs";
  import { onDestroy } from "svelte";
  import Card from "$lib/components/Card.svelte";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import BuffTooltip from "$lib/components/tooltips/BuffTooltip.svelte";
  import { abbreviateNumber, customRound, getSkillIcon, timestampToMinutesAndSeconds } from "$lib/utils";
  import { middot } from "$lib/components/Snippets.svelte";

  let {
    player,
    encounterDamageStats
  }: {
    player: Entity;
    encounterDamageStats: EncounterDamageStats;
  } = $props();

  type buffTypes = "Party" | "Self" | "Misc";
  let buffType: buffTypes = $state("Party");
  let skill = $derived(focusedCast.skillId ? player.skills[focusedCast.skillId] : undefined);
  let cast = $derived(skill ? skill.skillCastLog[focusedCast.cast] : undefined);
  let totalCastDamage = $derived(cast ? cast.hits.map((hit) => hit.damage).reduce((a, b) => a + b, 0) : 0);

  // focused cast info
  let crits = $derived(cast ? cast.hits.filter((hit) => hit.crit).length : 0);
  let critDamage = $derived(
    cast
      ? cast.hits
          .filter((hit) => hit.crit)
          .map((hit) => hit.damage)
          .reduce((a, b) => a + b, 0)
      : 0
  );
  let bas = $derived(cast ? cast.hits.filter((hit) => hit.backAttack).length : 0);
  let fas = $derived(cast ? cast.hits.filter((hit) => hit.frontAttack).length : 0);
  let supportBuffs = $derived.by(() => {
    if (!cast) return;
    return cast.hits.reduce(
      (acc, hit) => {
        const buffs = getSkillCastSupportBuffs(hit, encounterDamageStats);
        acc.buff += buffs.buff;
        acc.brand += buffs.brand;
        acc.identity += buffs.identity;
        return acc;
      },
      { buff: 0, brand: 0, identity: 0 }
    );
  });

  let allGroupedBuffs = $derived.by(() => {
    if (!cast) return;
    return cast.hits.map((hit) => getSkillCastBuffs(hit, encounterDamageStats, player, buffType));
  });

  onDestroy(() => {
    focusedCast.skillId = 0;
    focusedCast.cast = 0;
  });

  function getHighestDamageCastIndex() {
    if (!skill) return 0;
    let highestDamage = 0;
    let highestIndex = 0;
    for (const [i, cast] of skill.skillCastLog.entries()) {
      const totalDamage = cast.hits.map((hit) => hit.damage).reduce((a, b) => a + b, 0);
      if (totalDamage > highestDamage) {
        highestDamage = totalDamage;
        highestIndex = i;
      }
    }

    focusedCast.cast = highestIndex;
  }
</script>

{#snippet stats(stat: string, rate: number)}
  <p>
    {stat}: <span class="font-semibold">{customRound(rate * 100)}%</span>
  </p>
{/snippet}
{#snippet changeBuffType(type: buffTypes)}
  <button
    onclick={() => (buffType = type)}
    class="hover:text-accent-500/80 {type === buffType ? 'text-accent-500/70' : ''}"
  >
    {type}
  </button>
{/snippet}

<Card class="mb-80 mt-4">
  <div class="bg-black/10 px-3 py-2 font-medium">Skill Cast Details</div>
  <div class="overflow-x-scroll px-3 py-2">
    {#if focusedCast.skillId === 0}
      <div>Click on a skill icon in the chart above to show details.</div>
    {:else if cast && skill}
      <div class="flex flex-col gap-2">
        <!-- cast controls -->
        <div class="flex items-center gap-2">
          <button
            class="bg-accent-500/70 hover:bg-accent-500/80 rounded-sm px-1 py-0.5"
            onclick={getHighestDamageCastIndex}
          >
            <QuickTooltip tooltip="Show Highest Damage Cast" placement="top">Max Cast</QuickTooltip>
          </button>
          <button
            class="hover:text-accent-500/80 px-2"
            onclick={() => {
              focusedCast.cast = Math.max(0, focusedCast.cast - 1);
            }}
          >
            <QuickTooltip tooltip="Previous Cast" placement="top">
              <IconChevronLeft />
            </QuickTooltip>
          </button>
          <button
            class="hover:text-accent-500/80 px-2"
            onclick={() => {
              focusedCast.cast = Math.min(skill!.skillCastLog.length - 1, focusedCast.cast + 1);
            }}
          >
            <QuickTooltip tooltip="Next Cast" placement="top">
              <IconChevronRight />
            </QuickTooltip>
          </button>
        </div>

        <!-- skill name -->
        <div class="flex items-center gap-2">
          <img class="size-7 rounded-sm" src={getSkillIcon(skill.icon)} alt={skill.name} />
          <div class="font-semibold">
            {skill.name} #{focusedCast.cast + 1}
          </div>
        </div>

        <!-- cast time -->
        <div class="select-text font-mono text-xs">
          {#if cast.last - cast.timestamp === 0}
            <!-- instant cast / instant hit / no hit -->
            {timestampToMinutesAndSeconds(cast.timestamp, false, true)}
          {:else}
            {timestampToMinutesAndSeconds(cast.timestamp, false, true)} - {timestampToMinutesAndSeconds(
              cast.last,
              false,
              true
            )} ({customRound((cast.last - cast.timestamp) / 1000)}s)
          {/if}
        </div>

        <!-- cast summary -->
        <div>
          Total Damage: <span class="font-semibold">{abbreviateNumber(totalCastDamage)}</span>
        </div>
        {#if !cast.hits.length}
          <div>No Hits</div>
        {:else}
          <div class="flex flex-col">
            <div class="flex items-center gap-2">
              {@render stats("Crit", crits / cast.hits.length)}
              {@render middot()}
              {@render stats("CDMG", totalCastDamage > 0 ? critDamage / totalCastDamage : 0)}
              {#if bas > 0}
                {@render middot()}
                {@render stats("BA", bas / cast.hits.length)}
              {/if}
              {#if fas > 0}
                {@render middot()}
                {@render stats("FA", fas / cast.hits.length)}
              {/if}
            </div>
            {#if supportBuffs}
              <div class="flex items-center gap-2">
                {@render stats("Buff", totalCastDamage > 0 ? supportBuffs.buff / totalCastDamage : 0)}
                {@render middot()}
                {@render stats("Brand", totalCastDamage > 0 ? supportBuffs.brand / totalCastDamage : 0)}
                {@render middot()}
                {@render stats("Identity", totalCastDamage > 0 ? supportBuffs.identity / totalCastDamage : 0)}
              </div>
            {/if}
          </div>

          <!-- cast details -->
          <table class="table-fixed">
            <thead>
              <tr>
                <td class="w-12 font-semibold">
                  <QuickTooltip tooltip="Each damage tick" class="w-fit">Hits</QuickTooltip>
                </td>
                <td class="w-16 font-semibold">
                  <QuickTooltip tooltip="Time since previous damage tick" class="w-fit">Ticks</QuickTooltip>
                </td>
                <td class="w-12 font-semibold">
                  <QuickTooltip tooltip="Hit modifiers, e.g. Crit, BA, FA" class="w-fit">Mods</QuickTooltip>
                </td>
                <td class="w-16 font-semibold">
                  <QuickTooltip tooltip="Hit damage" class="w-fit">DMG</QuickTooltip>
                </td>
                <td class="font-semibold">
                  <QuickTooltip tooltip="Choose which buffs to view" class="flex w-fit items-center gap-1">
                    {@render changeBuffType("Party")}
                    {@render middot()}
                    {@render changeBuffType("Self")}
                    {@render middot()}
                    {@render changeBuffType("Misc")}
                  </QuickTooltip>
                </td>
              </tr>
            </thead>
            <tbody>
              {#each cast.hits as hit, i}
                <tr>
                  <td class="h-7 font-mono">#{i + 1}</td>
                  {#if i === 0}
                    <td class="font-mono">
                      <QuickTooltip
                        tooltip={`${timestampToMinutesAndSeconds(hit.timestamp, false, true)}`}
                        class="w-fit"
                      >
                        +{customRound((hit.timestamp - cast.timestamp) / 1000)}s
                      </QuickTooltip>
                    </td>
                  {:else}
                    <td class="font-mono">
                      <QuickTooltip
                        tooltip={`${timestampToMinutesAndSeconds(hit.timestamp, false, true)}`}
                        class="w-fit"
                      >
                        +{customRound((hit.timestamp - cast.hits[i - 1].timestamp) / 1000)}s
                      </QuickTooltip>
                    </td>
                  {/if}
                  <td class="font-mono">
                    <div class="flex items-center gap-1">
                      {#if hit.crit}
                        <QuickTooltip tooltip="Critical Hit" class="w-fit">C</QuickTooltip>
                      {/if}
                      {#if hit.backAttack}
                        <QuickTooltip tooltip="Back Attack" class="w-fit">B</QuickTooltip>
                      {/if}
                      {#if hit.frontAttack}
                        <QuickTooltip tooltip="Front Attack" class="w-fit">F</QuickTooltip>
                      {/if}
                      {#if !hit.crit && !hit.backAttack && !hit.frontAttack}
                        -
                      {/if}
                    </div>
                  </td>
                  <td class="font-mono">
                    <QuickTooltip tooltip={hit.damage.toLocaleString()} class="w-fit">
                      {abbreviateNumber(hit.damage)}
                    </QuickTooltip>
                  </td>
                  <td>
                    <div class="flex">
                      {#if allGroupedBuffs && allGroupedBuffs[i] && allGroupedBuffs[i].size > 0}
                        {#each allGroupedBuffs[i] as [_, groupedBuffs]}
                          {#each groupedBuffs as [_, buff]}
                            <BuffTooltip {buff} size="size-6" />
                          {/each}
                        {/each}
                      {:else}
                        -
                      {/if}
                    </div>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    {/if}
  </div>
</Card>
