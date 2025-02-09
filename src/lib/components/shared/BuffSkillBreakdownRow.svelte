<script lang="ts">
    import type { BuffDetails, Skill, StatusEffect } from "$lib/types";
    import { getSynergyPercentageDetails, hyperAwakeningIds } from "$lib/utils/buffs";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { settings, skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";
    import { generateSkillTooltip, tooltip } from "$lib/utils/tooltip";
    import BuffTooltipDetail from "./BuffTooltipDetail.svelte";

    interface Props {
        skill: Skill;
        color: string;
        groupedSynergies: Map<string, Map<number, StatusEffect>>;
        width: number;
        shadow?: boolean;
        index: number;
    }

    let { skill, color, groupedSynergies, width, shadow = false, index }: Props = $props();

    let synergyPercentageDetails: Array<BuffDetails> = $state(getSynergyPercentageDetails(groupedSynergies, skill));
    let isHyperAwakening = hyperAwakeningIds.has(skill.id);
</script>

<tr class="h-7 px-2 py-1 text-3xs {$settings.general.underlineHovered ? 'hover:underline' : ''}">
    <td class="pl-1">
        <img
            class="size-5"
            src={$skillIcon.path + getSkillIcon(skill.icon)}
            alt={skill.name}
            use:tooltip={{ content: skill.name }} />
    </td>
    <td colspan="2">
        <div class="truncate">
            <span use:tooltip={{ content: generateSkillTooltip(skill) }}>
                {skill.name}
            </span>
        </div>
    </td>
    {#if groupedSynergies.size > 0}
        {#each synergyPercentageDetails as synergy (synergy.id)}
            <td class="px-1 text-center">
                {#if synergy.percentage}
                    <BuffTooltipDetail {synergy} />
                {:else if isHyperAwakening}
                    -
                {/if}
            </td>
        {/each}
    {/if}
    <td
        class="absolute left-0 -z-10 h-7 px-2 py-1"
        class:shadow-md={shadow}
        style="background-color: {index % 2 === 1 && $settings.general.splitLines
            ? RGBLinearShade(HexToRgba(color, 0.6))
            : HexToRgba(color, 0.6)}; width: {width}%"></td>
</tr>
