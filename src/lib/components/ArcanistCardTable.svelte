<script lang="ts">
  import { cardIds } from "$lib/constants/cards";
  import { settings } from "$lib/stores.svelte.js";
  import type { Entity } from "$lib/types";
  import { getSkillIcon, rgbLinearShadeAdjust } from "$lib/utils";

  interface Props {
    player: Entity;
    duration: number;
  }

  let { player, duration }: Props = $props();

  let cards = Object.values(player.skills)
    .sort((a, b) => b.casts - a.casts)
    .filter((skill) => cardIds.includes(skill.id) || skill.id === 19282 || skill.id === 19288);

  let totalDraws = $state(0);
  let maxDraw = 0;
  let drawPercentages: number[] = $state([]);
  let relativeDrawPercentages: number[] = $state([]);
  if (cards.length > 0) {
    totalDraws = cards.reduce((acc, skill) => acc + skill.casts, 0);
    maxDraw = cards[0]!.casts;
    drawPercentages = cards.map((skill) => (skill.casts / totalDraws) * 100);
    relativeDrawPercentages = cards.map((skill) => (skill.casts / maxDraw) * 100);
  }
</script>

{#if cards.length > 0}
  <div class="mt-4 font-medium">Card Draw Distribution</div>
  <div class="mt-1 text-sm">
    <div>
      Total Cards Drawn: <span class="font-semibold">{totalDraws.toLocaleString()}</span>
    </div>
    <div>
      Draws per min: <span class="font-semibold">{(totalDraws / (duration / 1000 / 60)).toFixed(1)} cards/min</span>
    </div>
  </div>
  <table class="relative isolate mt-2 w-full table-fixed">
    <thead class="z-30 h-6">
      <tr class="bg-neutral-900">
        <th class="w-full px-2 text-left font-normal"></th>
        <th class="w-14 font-normal">Draws</th>
        <th class="w-20 font-normal">Draw %</th>
      </tr>
    </thead>
    <tbody class="relative z-10">
      {#each cards as card, i}
        <tr class="text-xxs h-6 px-2 py-1 {settings.app.general.underlineHovered ? 'hover:underline' : ''}">
          <td class="px-1">
            <div class="flex items-center space-x-1">
              <img class="size-5" src={getSkillIcon(card.icon)} alt={card.name} />
              <div class="truncate pl-px">
                {card.name}
              </div>
            </div>
          </td>
          <td class="px-1 text-center">
            {card.casts}
          </td>
          <td class="px-1 text-center">
            {drawPercentages[i]!.toFixed(1)}<span class="text-xxs text-neutral-300">%</span>
          </td>
          <td
            class="absolute left-0 -z-10 h-6 px-2 py-1"
            style="background-color: {i % 2 === 1 && settings.app.general.splitLines
              ? rgbLinearShadeAdjust(settings.classColors['Arcanist'] ?? '#fff', -0.2, 0.6)
              : `rgb(from ${settings.classColors['Arcanist'] ?? '#fff'} r g b / 0.6)`}; width: {relativeDrawPercentages[
              i
            ]}%"
          ></td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}
