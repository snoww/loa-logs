<script lang="ts">
    import { Buff, BuffDetails, type Entity, type StatusEffect } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { formatPlayerName } from "$lib/utils/strings";
    import { colors, classIconCache, settings } from "$lib/utils/settings";
    import { localPlayer, takingScreenshot } from "$lib/utils/stores";
    import BuffTooltipDetail from "../shared/BuffTooltipDetail.svelte";
    import { generateClassTooltip, tooltip } from "$lib/utils/tooltip";
    import { round } from "$lib/utils/numbers";
    import { addBardBubbles, supportSkills } from "$lib/utils/buffs";

    interface Props {
        player: Entity;
        groupedSynergies: Map<string, Map<number, StatusEffect>>;
        percentage: number;
    }

    let { player, groupedSynergies, percentage }: Props = $props();

    let color = $state("#ffffff");
    let playerName: string = $derived(formatPlayerName(player, $settings.general));
    let synergyPercentageDetails: Array<BuffDetails> = $state([]);

    if (Object.hasOwn($colors, player.class)) {
        if ($settings.general.constantLocalPlayerColor && $localPlayer == player.name) {
            color = $colors["Local"].color;
        } else {
            color = $colors[player.class].color;
        }
    }

    let damageDealt = player.damageStats.damageDealt;
    let damageDealtWithoutHA = player.damageStats.damageDealt - (player.damageStats.hyperAwakeningDamage ?? 0);

    if (groupedSynergies.size > 0) {
        synergyPercentageDetails = [];
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
                        round((player.damageStats.buffedBy[id] / (isHat ? damageDealt : damageDealtWithoutHA)) * 100),
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
                                (player.damageStats.debuffedBy[id] / (isHat ? damageDealt : damageDealtWithoutHA)) * 100
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
            synergyPercentageDetails.push(buff);
        });
    }
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
        <span use:tooltip={{ content: playerName }}>
            {playerName}
        </span>
    </div>
</td>
{#if groupedSynergies.size > 0}
    {#each synergyPercentageDetails as synergy}
        <td class="px-1 text-center text-3xs">
            {#if synergy.percentage}
                <BuffTooltipDetail {synergy} />
            {/if}
        </td>
    {/each}
{/if}
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={!$takingScreenshot}
    style="background-color: {HexToRgba(color, 0.6)}; width: {percentage}%"></td>
