<script lang="ts">
    import { EntityType, type Entity, type Skill } from "$lib/types";
    import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
    import LogPlayerBreakdownRow from "./LogPlayerBreakdownRow.svelte";
    import { classIconCache, colors, settings } from "$lib/utils/settings";
    import PlayerBreakdownHeader from "../shared/PlayerBreakdownHeader.svelte";
    import { cardIds } from "$lib/constants/cards";
    import { generateClassTooltip, tooltip } from "$lib/utils/tooltip";
    import { formatPlayerName } from "$lib/utils/strings";
    import { localPlayer, takingScreenshot } from "$lib/utils/stores";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";

    interface Props {
        entity: Entity;
        duration: number;
        totalDamageDealt: number;
    }

    let { entity, duration, totalDamageDealt }: Props = $props();

    let color = $state("#ffffff");
    let skills: Array<Skill> = $state([]);
    let skillDamagePercentages: Array<number> = $state([]);
    let abbreviatedSkillDamage: Array<(string | number)[]> = $state([]);
    let skillDps: Array<(string | number)[]> = $state([]);
    let skillDpsRaw: Array<number> = $state([]);

    let hasBackAttacks = $state(false);
    let hasFrontAttacks = $state(false);
    let anySupportBrand = $state(false);
    let anySupportIdentity = $state(false);
    let anySupportBuff = $state(false);

    let playerName: string = $derived(formatPlayerName(entity, $settings.general));

    let dps = abbreviateNumberSplit(entity.damageStats.dps);
    let damageDealt = abbreviateNumberSplit(entity.damageStats.damageDealt);
    let damagePercentage = ((entity.damageStats.damageDealt / totalDamageDealt) * 100).toFixed(1);
    let damageWithoutHa = entity.damageStats.damageDealt - (entity.damageStats.hyperAwakeningDamage ?? 0);

    let critPercentage = $state("0.0");
    let critDmgPercentage = $state("0.0");
    let baPercentage = $state("0.0");
    let faPercentage = $state("0.0");

    if (entity.skillStats.hits !== 0) {
        critDmgPercentage = round((entity.damageStats.critDamage / entity.damageStats.damageDealt) * 100);
        critPercentage = round((entity.skillStats.crits / entity.skillStats.hits) * 100);
        if (
            $settings.logs.positionalDmgPercent &&
            (entity.damageStats.frontAttackDamage > 0 || entity.damageStats.backAttackDamage > 0)
        ) {
            faPercentage = round((entity.damageStats.frontAttackDamage / entity.damageStats.damageDealt) * 100);
            baPercentage = round((entity.damageStats.backAttackDamage / entity.damageStats.damageDealt) * 100);
        } else {
            faPercentage = round((entity.skillStats.frontAttacks / entity.skillStats.hits) * 100);
            baPercentage = round((entity.skillStats.backAttacks / entity.skillStats.hits) * 100);
        }
    }

    $effect(() => {
        if (entity.class === "Arcanist") {
            skills = Object.values(entity.skills)
                .sort((a, b) => b.totalDamage - a.totalDamage)
                .filter((skill) => !cardIds.includes(skill.id));
        } else {
            skills = Object.values(entity.skills).sort((a, b) => b.totalDamage - a.totalDamage);
        }
    });

    if (Object.hasOwn($colors, entity.class)) {
        if ($settings.general.constantLocalPlayerColor && $localPlayer == entity.name) {
            color = $colors["Local"].color;
        } else {
            color = $colors[entity.class].color;
        }
    } else if (entity.entityType === EntityType.ESTHER) {
        color = "#4dc8d0";
    }

    $effect(() => {
        if (skills.length > 0) {
            let mostDamageSkill = skills[0].totalDamage;
            skillDamagePercentages = skills.map((skill) => (skill.totalDamage / mostDamageSkill) * 100);
            abbreviatedSkillDamage = skills.map((skill) => abbreviateNumberSplit(skill.totalDamage));
            skillDps = skills.map((skill) => abbreviateNumberSplit(skill.totalDamage / (duration / 1000)));
            skillDpsRaw = skills.map((skill) => Math.round(skill.totalDamage / (duration / 1000)));
            hasBackAttacks = skills.some((skill) => skill.backAttacks > 0);
            hasFrontAttacks = skills.some((skill) => skill.frontAttacks > 0);
            anySupportBuff = skills.some((skill) => skill.buffedBySupport > 0);
            anySupportIdentity = skills.some((skill) => skill.buffedByIdentity > 0);
            anySupportBrand = skills.some((skill) => skill.debuffedBySupport > 0);
        }
    });
</script>

<thead class="z-30 h-6">
    <tr class="bg-zinc-900">
        <PlayerBreakdownHeader
            meterSettings={"logs"}
            {hasFrontAttacks}
            {hasBackAttacks}
            {anySupportBuff}
            {anySupportIdentity}
            {anySupportBrand} />
    </tr>
</thead>
<tbody class="relative z-10">
    {#if entity.entityType !== EntityType.ESTHER}
        <tr class="h-7 px-2 py-1 text-3xs {$settings.general.underlineHovered ? 'hover:underline' : ''}">
            <td class="pl-1">
                <img
                    class="table-cell size-5"
                    src={$classIconCache[entity.classId]}
                    alt={entity.class}
                    use:tooltip={{ content: generateClassTooltip(entity) }} />
            </td>
            <td colspan="2">
                <div class="truncate">
                    <span use:tooltip={{ content: playerName }}>
                        {playerName}
                    </span>
                </div>
            </td>
            {#if $settings.logs.breakdown.damage}
                <td class="px-1 text-center" use:tooltip={{ content: entity.damageStats.damageDealt.toLocaleString() }}>
                    {damageDealt[0]}<span class="text-3xs text-gray-300">{damageDealt[1]}</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.dps}
                <td class="px-1 text-center" use:tooltip={{ content: entity.damageStats.dps.toLocaleString() }}>
                    {dps[0]}<span class="text-3xs text-gray-300">{dps[1]}</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.damagePercent}
                <td class="px-1 text-center">
                    {damagePercentage}<span class="text-xs text-gray-300">%</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.critRate}
                <td class="px-1 text-center">
                    {critPercentage}<span class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.adjustedCritRate}
                <td class="px-1 text-center"> - </td>
            {/if}
            {#if $settings.logs.breakdown.critDmg}
                <td class="px-1 text-center">
                    {critDmgPercentage}<span class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if hasFrontAttacks && $settings.logs.breakdown.frontAtk}
                <td class="px-1 text-center">
                    {faPercentage}<span class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if hasBackAttacks && $settings.logs.breakdown.backAtk}
                <td class="px-1 text-center">
                    {baPercentage}<span class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if anySupportBuff && $settings.logs.breakdown.percentBuffBySup}
                <td class="px-1 text-center">
                    {round((entity.damageStats.buffedBySupport / damageWithoutHa) * 100)}<span
                        class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if anySupportBrand && $settings.logs.breakdown.percentBrand}
                <td class="px-1 text-center">
                    {round((entity.damageStats.debuffedBySupport / damageWithoutHa) * 100)}<span
                        class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if anySupportIdentity && $settings.logs.breakdown.percentIdentityBySup}
                <td class="px-1 text-center">
                    {round((entity.damageStats.buffedByIdentity / damageWithoutHa) * 100)}<span
                        class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.avgDamage}
                <td class="px-1 text-center"> - </td>
                <td class="px-1 text-center"> - </td>
            {/if}
            {#if $settings.logs.breakdown.maxDamage}
                <td class="px-1 text-center"> - </td>
                <td class="px-1 text-center"> - </td>
            {/if}
            {#if $settings.logs.breakdown.casts}
                <td
                    class="px-1 text-center"
                    use:tooltip={{
                        content: `<div class="py-1">${
                            entity.skillStats.casts.toLocaleString() +
                            " " +
                            (entity.skillStats.casts === 1 ? "cast" : "casts")
                        }</div>`
                    }}>
                    {abbreviateNumberSplit(entity.skillStats.casts)[0]}<span class="text-3xs text-gray-300"
                        >{abbreviateNumberSplit(entity.skillStats.casts)[1]}</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.cpm}
                <td class="px-1 text-center">
                    <div
                        use:tooltip={{
                            content: `<div class="py-1">${
                                entity.skillStats.casts.toLocaleString() +
                                " " +
                                (entity.skillStats.casts === 1 ? "cast" : "casts")
                            }</div>`
                        }}>
                        {round(entity.skillStats.casts / (duration / 1000 / 60))}
                    </div>
                </td>
            {/if}
            {#if $settings.logs.breakdown.hits}
                <td
                    class="px-1 text-center"
                    use:tooltip={{
                        content: `<div class="py-1">${
                            entity.skillStats.hits.toLocaleString() +
                            " " +
                            (entity.skillStats.hits === 1 ? "hit" : "hits")
                        }</div>`
                    }}>
                    {abbreviateNumberSplit(entity.skillStats.hits)[0]}<span class="text-3xs text-gray-300"
                        >{abbreviateNumberSplit(entity.skillStats.hits)[1]}</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.hpm}
                <td class="px-1 text-center">
                    {#if entity.skillStats.hits === 0}
                        <div class="">0</div>
                    {:else}
                        <div
                            use:tooltip={{
                                content: `<div class="py-1">${
                                    entity.skillStats.hits.toLocaleString() +
                                    " " +
                                    (entity.skillStats.hits === 1 ? "hit" : "hits")
                                }</div>`
                            }}>
                            {round(entity.skillStats.hits / (duration / 1000 / 60))}
                        </div>
                    {/if}
                </td>
            {/if}
            <td
                class="absolute left-0 -z-10 h-7 px-2 py-1"
                class:shadow-md={!$takingScreenshot}
                style="background-color: {$settings.general.splitLines
                    ? RGBLinearShade(HexToRgba(color, 0.6))
                    : HexToRgba(color, 0.6)}; width: 100%"></td>
        </tr>
    {/if}
    {#each skills as skill, i (skill.id)}
        <LogPlayerBreakdownRow
            {skill}
            {color}
            {hasFrontAttacks}
            {hasBackAttacks}
            {anySupportBuff}
            {anySupportIdentity}
            {anySupportBrand}
            abbreviatedSkillDamage={abbreviatedSkillDamage[i]}
            playerDamageDealt={entity.damageStats.damageDealt}
            damagePercentage={skillDamagePercentages[i]}
            skillDps={skillDps[i]}
            skillDpsRaw={skillDpsRaw[i]}
            {duration}
            index={i} />
    {/each}
</tbody>
