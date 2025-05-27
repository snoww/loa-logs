<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { raidGates } from "$lib/constants/encounters";
  import { IconStar } from "$lib/icons";
  import type { EncounterPreview, EncountersOverview } from "$lib/types";
  import { abbreviateNumber, formatDurationFromMs, formatTimestamp } from "$lib/utils/numbers";
  import { getClassIcon } from "$lib/utils/strings";

  let { overview }: { overview: EncountersOverview } = $props();
</script>

{#snippet encounterPreview(encounter: EncounterPreview)}
  {@const gate = raidGates[encounter.bossName]}
  <tr class="items-center border-b border-neutral-700/50 hover:bg-neutral-800">
    <td>
      <div class="p-2" class:text-lime-400={encounter.cleared}>
        #{encounter.id}
      </div>
    </td>
    <td class="w-full p-2 font-medium">
      <div class="flex flex-col gap-1">
        <div class="flex gap-1 text-nowrap text-neutral-300">
          {#if encounter.difficulty}
            <p
              class="py-.5 rounded-sm bg-neutral-700/80 px-1 text-xs"
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
          {#if gate}
            <p class="py-.5 truncate rounded-sm bg-neutral-700/80 px-1 text-xs">
              {gate}
            </p>
          {/if}
        </div>
        <a
          href="/logs/encounter/{encounter.id}"
          class="hover:text-accent-500 group flex items-center gap-1 hover:underline"
        >
          {#if encounter.favorite}
            <IconStar class="shrink-0 text-yellow-400" style="fill: currentColor;" />
          {/if}
          <QuickTooltip tooltip={encounter.bossName} class="truncate">
            {encounter.bossName}
          </QuickTooltip>
        </a>
      </div>
    </td>
    <td class="p-3">
      <div class="mask-r-from-80% mask-r-to-100% flex">
        {#each encounter.classes as classId, i}
          <QuickTooltip tooltip={encounter.names[i]} class="shrink-0">
            <img src={getClassIcon(classId)} alt="class-{classId}" class="size-8" />
          </QuickTooltip>
        {/each}
      </div>
    </td>
    <td class="p-1">
      <div class="flex">
        <QuickTooltip tooltip={encounter.localPlayer} class="truncate">
          {encounter.localPlayer}
        </QuickTooltip>
      </div>
    </td>
    <td class="hidden p-1 text-right md:table-cell">
      {abbreviateNumber(encounter.myDps)}
    </td>
    <td class="p-1 text-right">
      {formatDurationFromMs(encounter.duration)}
    </td>
    <td class="pr-2 text-right text-xs">
      {formatTimestamp(encounter.fightStart)}
    </td>
  </tr>
{/snippet}

<table class="w-full table-fixed">
  <thead class="sticky top-0 z-10 bg-[#121212]/95 shadow-lg backdrop-blur-lg">
    <tr>
      <th class="w-14 p-3">ID</th>
      <th class="w-[25%] p-3 text-left">Encounter</th>
      <th class="p-3 text-left">Classes</th>
      <th class="w-24 px-1 text-left lg:w-32">Name</th>
      <th class="hidden w-20 px-1 text-right md:table-cell">DPS</th>
      <th class="w-24 px-1 text-right">Duration</th>
      <th class="w-24 pr-2 text-right">Date</th>
    </tr>
  </thead>
  <tbody class="text-neutral-200">
    {#each overview.encounters as enc}
      {@render encounterPreview(enc)}
    {/each}
  </tbody>
</table>
