<script lang="ts">
    import type { BuffDetails, Skill, StatusEffect } from "$lib/types";
    import { getSynergyPercentageDetails } from "$lib/utils/buffs";
    import { HexToRgba } from "$lib/utils/colors";
    import { skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";
    import { tooltip } from "$lib/utils/tooltip";
    import BuffTooltipDetail from "./BuffTooltipDetail.svelte";

    export let skill: Skill;
    export let color: string;
    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;
    export let width: number;
    export let shadow = false;

    let synergyPercentageDetails: Array<BuffDetails>;

    $: {
        if (groupedSynergies.size > 0) {
            synergyPercentageDetails = getSynergyPercentageDetails(groupedSynergies, skill);
        }
    }
</script>

<tr class="h-7 px-2 py-1 text-3xs">
    <td class="pl-1">
        <img
            class="h-5 w-5"
            src={$skillIcon.path + getSkillIcon(skill.icon)}
            alt={skill.name}
            use:tooltip={{ content: skill.name }} />
    </td>
    <td colspan="2">
        <div class="truncate">
            {skill.name}
        </div>
    </td>
    {#if groupedSynergies.size > 0}
        {#each synergyPercentageDetails as synergy (synergy.id)}
            <td class="px-1 text-center">
                {#if synergy.percentage}
                    <BuffTooltipDetail {synergy} />
                {/if}
            </td>
        {/each}
    {/if}
    <div
        class="absolute left-0 -z-10 h-7 px-2 py-1"
        class:shadow-md={shadow}
        style="background-color: {HexToRgba(color, 0.6)}; width: {width}%" />
</tr>
