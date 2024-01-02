<script lang="ts">
    import { EntityType, type Entity, type Skill } from "$lib/types";
    import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
    import LogPlayerBreakdownRow from "./LogPlayerBreakdownRow.svelte";
    import { classIconCache, colors, settings } from "$lib/utils/settings";
    import PlayerBreakdownHeader from "../shared/PlayerBreakdownHeader.svelte";
    import { cardIds } from "$lib/constants/cards";
    import { tooltip } from "$lib/utils/tooltip";
    import { formatPlayerName } from "$lib/utils/strings";
    import { takingScreenshot } from "$lib/utils/stores";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";

    export let entity: Entity;
    export let duration: number;
    export let totalDamageDealt: number;

    let color = "#ffffff";
    let skills: Array<Skill> = [];
    let skillDamagePercentages: Array<number> = [];
    let abbreviatedSkillDamage: Array<(string | number)[]> = [];
    let skillDps: Array<(string | number)[]> = [];

    let hasBackAttacks = false;
    let hasFrontAttacks = false;
    let anySupportBrand = false;
    let anySupportBuff = false;

    let playerName: string;

    $: {
        playerName = formatPlayerName(entity, $settings.general.showNames, $settings.general.showGearScore, false);
    }
    let dps = abbreviateNumberSplit(entity.damageStats.dps);
    let damageDealt = abbreviateNumberSplit(entity.damageStats.damageDealt);
    let damagePercentage = ((entity.damageStats.damageDealt / totalDamageDealt) * 100).toFixed(1);

    let critPercentage = "0.0";
    let critDmgPercentage = "0.0";
    let baPercentage = "0.0";
    let faPercentage = "0.0";

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

    skills = Object.values(entity.skills).sort((a, b) => b.totalDamage - a.totalDamage);
    if (entity.class === "Arcanist") {
        skills = skills.filter((skill) => !cardIds.includes(skill.id));
    }

    if (Object.hasOwn($colors, entity.class)) {
        color = $colors[entity.class].color;
    } else if (entity.entityType === EntityType.ESTHER) {
        color = "#4dc8d0";
    }

    if (skills.length > 0) {
        let mostDamageSkill = skills[0].totalDamage;
        skillDamagePercentages = skills.map((skill) => (skill.totalDamage / mostDamageSkill) * 100);
        abbreviatedSkillDamage = skills.map((skill) => abbreviateNumberSplit(skill.totalDamage));
        skillDps = skills.map((skill) => abbreviateNumberSplit(skill.totalDamage / (duration / 1000)));
        hasBackAttacks = skills.some((skill) => skill.backAttacks > 0);
        hasFrontAttacks = skills.some((skill) => skill.frontAttacks > 0);
        anySupportBuff = skills.some((skill) => skill.buffedBySupport > 0);
        anySupportBrand = skills.some((skill) => skill.debuffedBySupport > 0);
    }
</script>

<thead
    class="z-30 h-6"
    on:contextmenu|preventDefault={() => {
        console.log("titlebar clicked");
    }}>
    <tr class="bg-zinc-900">
        <PlayerBreakdownHeader
            meterSettings={$settings.logs}
            {hasFrontAttacks}
            {hasBackAttacks}
            {anySupportBuff}
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
                    use:tooltip={{ content: entity.class }} />
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
                <td class="px-1 text-center">
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
                    {round((entity.damageStats.buffedBySupport / entity.damageStats.damageDealt) * 100)}<span
                        class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if anySupportBrand && $settings.logs.breakdown.percentBrand}
                <td class="px-1 text-center">
                    {round((entity.damageStats.debuffedBySupport / entity.damageStats.damageDealt) * 100)}<span
                        class="text-3xs text-gray-300">%</span>
                </td>
            {/if}
            {#if $settings.logs.breakdown.avgDamage}
                <td class="px-1 text-center"> - </td>
                <td class="px-1 text-center"> - </td>
            {/if}
            {#if $settings.logs.breakdown.maxDamage}
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
            <div
                class="absolute left-0 -z-10 h-7 px-2 py-1"
                class:shadow-md={!$takingScreenshot}
                style="background-color: {$settings.general.splitLines
                    ? RGBLinearShade(HexToRgba(color, 0.6))
                    : HexToRgba(color, 0.6)}; width: 100%" />
        </tr>
    {/if}
    {#each skills as skill, i (skill.id)}
        <LogPlayerBreakdownRow
            {skill}
            {color}
            {hasFrontAttacks}
            {hasBackAttacks}
            {anySupportBuff}
            {anySupportBrand}
            abbreviatedSkillDamage={abbreviatedSkillDamage[i]}
            playerDamageDealt={entity.damageStats.damageDealt}
            damagePercentage={skillDamagePercentages[i]}
            skillDps={skillDps[i]}
            {duration}
            index={i} />
    {/each}
</tbody>
