<script lang="ts">
    import { cardIds } from "$lib/constants/cards";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { getSkillIcon } from "$lib/utils/strings";
    import { colors, settings, skillIcon } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import type { Entity } from "$lib/types";

    export let player: Entity;
    export let duration: number;

    let cards = Object.values(player.skills)
        .sort((a, b) => b.casts - a.casts)
        .filter((skill) => cardIds.includes(skill.id) || skill.id === 19282);

    let totalDraws = 0;
    let maxDraw = 0;
    let drawPercentages: number[] = [];
    let relativeDrawPercentages: number[] = [];
    if (cards.length > 0) {
        totalDraws = cards.reduce((acc, skill) => acc + skill.casts, 0);
        maxDraw = cards[0].casts;
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
        <div class="">
            Draws per min: <span class="font-semibold"
                >{(totalDraws / (duration / 1000 / 60)).toFixed(1)} cards/min</span>
        </div>
    </div>
    <table class="relative mt-2 table-fixed" style="width: calc(100vw - 4rem)">
        <thead class="z-30 h-6">
            <tr class="bg-zinc-900">
                <th class="w-full px-2 text-left font-normal" />
                <th class="w-14 font-normal">Draws</th>
                <th class="w-20 font-normal">Draw %</th>
            </tr>
        </thead>
        <tbody class="relative z-10">
            {#each cards as card, i}
                <tr class="h-6 px-2 py-1 text-3xs {$settings.general.underlineHovered ? 'hover:underline' : ''}">
                    <td class="px-1">
                        <div class="flex items-center space-x-1">
                            <img class="size-5" src={$skillIcon.path + getSkillIcon(card.icon)} alt={card.name} />
                            <div class="truncate pl-px">
                                {card.name}
                            </div>
                        </div>
                    </td>
                    <td class="px-1 text-center">
                        {card.casts}
                    </td>
                    <td class="px-1 text-center">
                        {drawPercentages[i].toFixed(1)}<span class="text-3xs text-gray-300">%</span>
                    </td>
                    <div
                        class="absolute left-0 -z-10 h-6 px-2 py-1"
                        class:shadow-md={!$takingScreenshot}
                        style="background-color: {i % 2 === 1 && $settings.general.splitLines
                            ? RGBLinearShade(HexToRgba($colors['Arcanist'].color, 0.6))
                            : HexToRgba($colors['Arcanist'].color, 0.6)}; width: {relativeDrawPercentages[i]}%" />
                </tr>
            {/each}
        </tbody>
    </table>
{/if}
