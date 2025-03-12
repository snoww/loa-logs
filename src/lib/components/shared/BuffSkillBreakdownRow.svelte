<script lang="ts">
    import type { BuffDetails, Skill, StatusEffect } from "$lib/types";
    import { getSynergyPercentageDetails, hyperAwakeningIds } from "$lib/utils/buffs";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { settings, skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";
    import { generateSkillTooltip, tooltip } from "$lib/utils/tooltip";
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";
    import BuffTooltipDetail from "./BuffTooltipDetail.svelte";
    import type { EntityState } from "$lib/entity.svelte";
    import { takingScreenshot } from "$lib/utils/stores";

    interface Props {
        skill: Skill;
        entityState: EntityState;
        groupedSynergies: Map<string, Map<number, StatusEffect>>;
        width: number;
        index: number;
    }

    let { skill, entityState, groupedSynergies, width, index }: Props = $props();

    let synergyPercentageDetails: Array<BuffDetails> = $derived(getSynergyPercentageDetails(groupedSynergies, skill));
    let isHyperAwakening = hyperAwakeningIds.has(skill.id);

    const tweenedValue = new Tween(entityState.enc.live ? 0 : width, {
        duration: 400,
        easing: cubicOut
    });
    $effect(() => {
        tweenedValue.set(width ?? 0);
    });
</script>

<tr class="text-3xs h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}">
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
        class:shadow-md={!takingScreenshot}
        style="background-color: {index % 2 === 1 && $settings.general.splitLines
            ? RGBLinearShade(HexToRgba(entityState.color, 0.6))
            : HexToRgba(entityState.color, 0.6)}; width: {tweenedValue.current}%"></td>
</tr>
