<script lang="ts">
    import { cardIds } from "$lib/constants/cards";
    import { MeterTab, type Entity, type Skill, type StatusEffect, BuffDetails } from "$lib/types";
    import { getSynergyPercentageDetailsSum } from "$lib/utils/buffs";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { colors, classIconCache, settings } from "$lib/utils/settings";
    import { formatPlayerName } from "$lib/utils/strings";
    import { tooltip } from "$lib/utils/tooltip";
    import BuffSkillBreakdownRow from "./BuffSkillBreakdownRow.svelte";
    import BuffTooltipDetail from "./shared/BuffTooltipDetail.svelte";
    import { localPlayer } from "$lib/utils/stores";

    interface Props {
        groupedSynergies: Map<string, Map<number, StatusEffect>>;
        player: Entity;
        tab: MeterTab;
    }

    let { groupedSynergies, player, tab }: Props = $props();

    let color: string = $state("");
    let skillDamagePercentages: Array<number> = $state([]);
    let skills = $state(Array<Skill>());
    let playerName: string = $derived(formatPlayerName(player, $settings.general));
    let buffSummary: BuffDetails[] = $derived.by(() => {
        if (tab === MeterTab.SELF_BUFFS || tab === MeterTab.PARTY_BUFFS) {
            return getSynergyPercentageDetailsSum(groupedSynergies, skills, player.damageStats);
        }
        return [];
    });

    $effect(() => {
        if (player.class === "Arcanist") {
            skills = Object.values(player.skills)
                .sort((a, b) => b.totalDamage - a.totalDamage)
                .filter((skill) => !cardIds.includes(skill.id));
        } else {
            skills = Object.values(player.skills).sort((a, b) => b.totalDamage - a.totalDamage);
        }
    });

    $effect(() => {
        if (skills.length > 0) {
            let mostDamageSkill = skills[0].totalDamage;
            skillDamagePercentages = skills.map((skill) => (skill.totalDamage / mostDamageSkill) * 100);
        }
    });

    $effect(() => {
        if (Object.hasOwn($colors, player.class)) {
            if ($settings.general.constantLocalPlayerColor && $localPlayer == player.name) {
                color = $colors["Local"].color;
            } else {
                color = $colors[player.class].color;
            }
        }
    });
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
        <td
            class="absolute left-0 -z-10 h-7 w-full px-2 py-1"
            style="background-color: {$settings.general.splitLines
                ? RGBLinearShade(HexToRgba(color, 0.6))
                : HexToRgba(color, 0.6)}"></td>
    </tr>
{/if}
{#each skills as skill, i (skill.id)}
    <BuffSkillBreakdownRow {groupedSynergies} {skill} {color} damagePercentage={skillDamagePercentages[i]} index={i} />
{/each}
