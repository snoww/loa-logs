<script lang="ts">
    import type { EncounterState } from "$lib/encounter.svelte";
    import { EntityState } from "$lib/entity.svelte";
    import { Buff, BuffDetails, type Entity, type StatusEffect } from "$lib/types";
    import { addBardBubbles, supportSkills } from "$lib/utils/buffs";
    import { HexToRgba } from "$lib/utils/colors";
    import { round } from "$lib/utils/numbers";
    import { classIconCache } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import { generateClassTooltip, tooltip } from "$lib/utils/tooltip";
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";
    import BuffTooltipDetail from "./BuffTooltipDetail.svelte";

    interface Props {
        enc: EncounterState;
        player: Entity;
        groupedSynergies: Map<string, Map<number, StatusEffect>>;
        percentage: number;
    }

    let { enc, player, groupedSynergies, percentage }: Props = $props();

    let entityState = $derived(new EntityState(player, enc));

    let synergyPercentageDetails: Array<BuffDetails> = $derived.by(() => {
        let damageDealt = player.damageStats.damageDealt;
        let damageDealtWithoutHA = player.damageStats.damageDealt - (player.damageStats.hyperAwakeningDamage ?? 0);
        if (groupedSynergies.size > 0) {
            let tempSynergyPercentageDetails: Array<BuffDetails> = [];
            groupedSynergies.forEach((synergies, key) => {
                let synergyDamage = 0;
                let buff = new BuffDetails();
                let isHat = false;
                synergies.forEach((syn, id) => {
                    if (supportSkills.haTechnique.includes(id)) {
                        isHat = true;
                    }
                    if (player.damageStats.buffedBy[id]) {
                        let b = new Buff(
                            syn.source.icon,
                            round(
                                (player.damageStats.buffedBy[id] / (isHat ? damageDealt : damageDealtWithoutHA)) * 100
                            ),
                            syn.source.skill?.icon
                        );
                        addBardBubbles(key, b, syn);
                        buff.buffs.push(b);
                        synergyDamage += player.damageStats.buffedBy[id];
                    } else if (player.damageStats.debuffedBy[id]) {
                        buff.buffs.push(
                            new Buff(
                                syn.source.icon,
                                round(
                                    (player.damageStats.debuffedBy[id] / (isHat ? damageDealt : damageDealtWithoutHA)) *
                                        100
                                ),
                                syn.source.skill?.icon
                            )
                        );
                        synergyDamage += player.damageStats.debuffedBy[id];
                    }
                });

                if (synergyDamage > 0) {
                    buff.percentage = round((synergyDamage / (isHat ? damageDealt : damageDealtWithoutHA)) * 100);
                }
                tempSynergyPercentageDetails.push(buff);
            });

            return tempSynergyPercentageDetails;
        }
        return [];
    });

    const tweenedValue = new Tween(enc.live ? 0 : percentage, {
        duration: 400,
        easing: cubicOut
    });

    $effect(() => {
        tweenedValue.set(percentage ?? 0);
    });

    let alpha = $derived(enc.live && !enc.curSettings.showClassColors ? 0 : 0.6);
</script>

<td class="pl-1">
    <img
        class="table-cell size-5"
        src={$classIconCache[player.classId]}
        alt={player.class}
        use:tooltip={{ content: generateClassTooltip(player) }} />
</td>
<td colspan="2">
    <div class="truncate">
        <span use:tooltip={{ content: entityState.name }}>
            {entityState.name}
        </span>
    </div>
</td>
{#if groupedSynergies.size > 0}
    {#each synergyPercentageDetails as synergy}
        <td class="text-3xs px-1 text-center">
            {#if synergy.percentage}
                <BuffTooltipDetail {synergy} />
            {/if}
        </td>
    {/each}
{/if}
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={!$takingScreenshot}
    style="background-color: {HexToRgba(entityState.color, alpha)}; width: {tweenedValue.current}%"></td>
