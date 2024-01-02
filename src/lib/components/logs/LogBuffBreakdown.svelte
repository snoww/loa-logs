<script lang="ts">
    import { cardIds } from "$lib/constants/cards";
    import { MeterTab, type Entity, type StatusEffect, BuffDetails } from "$lib/types";
    import { getSynergyPercentageDetailsSum } from "$lib/utils/buffs";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { colors, classIconCache, settings } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import { formatPlayerName } from "$lib/utils/strings";
    import { tooltip } from "$lib/utils/tooltip";
    import BuffTooltipDetail from "../shared/BuffTooltipDetail.svelte";
    import LogBuffBreakdownRow from "./LogBuffBreakdownRow.svelte";

    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;
    export let player: Entity;
    export let tab: MeterTab;

    let playerName: string;

    $: {
        playerName = formatPlayerName(player, $settings.general.showNames, $settings.general.showGearScore, false);
    }

    let color: string;
    let skillDamagePercentages: Array<number> = [];

    let skills = Object.values(player.skills).sort((a, b) => b.totalDamage - a.totalDamage);
    if (player.class === "Arcanist") {
        skills = skills.filter((skill) => !cardIds.includes(skill.id));
    }
    if (Object.hasOwn($colors, player.class)) {
        color = $colors[player.class].color;
    }

    if (skills.length > 0) {
        let mostDamageSkill = skills[0].totalDamage;
        skillDamagePercentages = skills.map((skill) => (skill.totalDamage / mostDamageSkill) * 100);
    }
    let buffSummary: BuffDetails[];
    if (tab === MeterTab.SELF_BUFFS || tab === MeterTab.PARTY_BUFFS) {
        buffSummary = getSynergyPercentageDetailsSum(groupedSynergies, skills, player.damageStats.damageDealt);
    }
</script>

{#if tab === MeterTab.SELF_BUFFS || tab === MeterTab.PARTY_BUFFS}
    <tr class="h-7 px-2 py-1 text-3xs {$settings.general.underlineHovered ? 'hover:underline' : ''}">
        <td class="pl-1">
            <img
                class="table-cell size-5"
                src={$classIconCache[player.classId]}
                alt={player.class}
                use:tooltip={{ content: player.class }} />
        </td>
        <td colspan="2">
            <div class="truncate">
                <span use:tooltip={{ content: playerName }}>
                    {playerName}
                </span>
            </div>
        </td>
        {#if groupedSynergies.size > 0}
            {#each buffSummary as synergy (synergy.id)}
                <td class="px-1 text-center">
                    {#if synergy.percentage}
                        <BuffTooltipDetail {synergy} />
                    {/if}
                </td>
            {/each}
        {/if}
        <div
            class="absolute left-0 -z-10 h-7 w-full px-2 py-1"
            class:shadow-md={!$takingScreenshot}
            style="background-color: {$settings.general.splitLines ? RGBLinearShade(HexToRgba(color, 0.6)) : HexToRgba(color, 0.6)}" />
    </tr>
{/if}
{#each skills as skill, i (skill.id)}
    <LogBuffBreakdownRow {groupedSynergies} {skill} {color} damagePercentage={skillDamagePercentages[i]} index={i} />
{/each}
