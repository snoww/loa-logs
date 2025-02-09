<script lang="ts">
    import type { Skill } from "$lib/types";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
    import { settings, skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";
    import { generateSkillTooltip, tooltip } from "$lib/utils/tooltip";

    interface Props {
        skill: Skill;
        color: string;
        hasFrontAttacks: boolean;
        hasBackAttacks: boolean;
        anySupportBuff: boolean;
        anySupportIdentity: boolean;
        anySupportBrand: boolean;
        abbreviatedSkillDamage: (string | number)[];
        skillDps: (string | number)[];
        skillDpsRaw: number;
        playerDamageDealt: number;
        width: number;
        duration: number;
        meterSettings: string;
        shadow?: boolean;
        index: number;
    }

    let {
        skill,
        color,
        hasFrontAttacks,
        hasBackAttacks,
        anySupportBuff,
        anySupportIdentity,
        anySupportBrand,
        abbreviatedSkillDamage,
        skillDps,
        skillDpsRaw,
        playerDamageDealt,
        width,
        duration,
        meterSettings,
        shadow = false,
        index
    }: Props = $props();

    let critPercentage = $state("0.0");
    let critDmgPercentage = $state("0.0");
    let baPercentage = $state("0.0");
    let faPercentage = $state("0.0");
    let averagePerCast = $derived(skill.totalDamage / skill.casts);
    let adjustedCritPercentage: string | undefined = $state(undefined);

    let currentSettings = $state($settings.logs);
    if (meterSettings === "logs") {
        currentSettings = $settings.logs;
    } else {
        currentSettings = $settings.meter;
    }

    $effect(() => {
        if (currentSettings.breakdown.adjustedCritRate) {
            if (skill.adjustedCrit) {
                adjustedCritPercentage = round(skill.adjustedCrit * 100);
            } else {
                let filter = averagePerCast * 0.05;
                let adjustedCrits = 0;
                let adjustedHits = 0;
                if (skill.skillCastLog.length > 0) {
                    for (const c of skill.skillCastLog) {
                        for (const h of c.hits) {
                            if (h.damage > filter) {
                                adjustedCrits += h.crit ? 1 : 0;
                                adjustedHits += 1;
                            }
                        }
                    }
                    if (adjustedHits > 0) {
                        adjustedCritPercentage = round((adjustedCrits / adjustedHits) * 100);
                    }
                }
            }
        }
        if (skill.hits !== 0) {
            critDmgPercentage = round((skill.critDamage / skill.totalDamage) * 100);
            critPercentage = round((skill.crits / skill.hits) * 100);
            if (currentSettings.positionalDmgPercent && (skill.frontAttackDamage > 0 || skill.backAttackDamage > 0)) {
                faPercentage = round((skill.frontAttackDamage / skill.totalDamage) * 100);
                baPercentage = round((skill.backAttackDamage / skill.totalDamage) * 100);
            } else {
                faPercentage = round((skill.frontAttacks / skill.hits) * 100);
                baPercentage = round((skill.backAttacks / skill.hits) * 100);
            }
        }
    });
</script>

<td class="pl-1">
    <img
        class="size-5"
        src={$skillIcon.path + getSkillIcon(skill.icon)}
        alt={skill.name}
        use:tooltip={{ content: skill.name }} />
</td>
<td class="-left-px" colspan="2">
    <div class="truncate">
        <span use:tooltip={{ content: generateSkillTooltip(skill) }}>
            {skill.name}
        </span>
    </div>
</td>
{#if currentSettings.breakdown.damage}
    <td class="px-1 text-center" use:tooltip={{ content: skill.totalDamage.toLocaleString() }}>
        {abbreviatedSkillDamage[0]}<span class="text-3xs text-gray-300">{abbreviatedSkillDamage[1]}</span>
    </td>
{/if}
{#if currentSettings.breakdown.dps}
    <td class="px-1 text-center" use:tooltip={{ content: skillDpsRaw.toLocaleString() }}>
        {skillDps[0]}<span class="text-3xs text-gray-300">{skillDps[1]}</span>
    </td>
{/if}
{#if currentSettings.breakdown.damagePercent}
    <td class="px-1 text-center">
        {round((skill.totalDamage / playerDamageDealt) * 100)}<span class="text-xs text-gray-300">%</span>
    </td>
{/if}
{#if currentSettings.breakdown.critRate}
    <td class="px-1 text-center">
        {critPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if meterSettings === "logs" && currentSettings.breakdown.adjustedCritRate}
    <td class="px-1 text-center">
        {#if adjustedCritPercentage}
            {adjustedCritPercentage}<span class="text-3xs text-gray-300">%</span>
        {:else}
            -
        {/if}
    </td>
{/if}
{#if currentSettings.breakdown.critDmg}
    <td class="px-1 text-center">
        {critDmgPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if hasFrontAttacks && currentSettings.breakdown.frontAtk}
    <td class="px-1 text-center">
        {faPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if hasBackAttacks && currentSettings.breakdown.backAtk}
    <td class="px-1 text-center">
        {baPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if anySupportBuff && currentSettings.breakdown.percentBuffBySup}
    <td class="px-1 text-center">
        {#if skill.totalDamage > 0}
            {round((skill.buffedBySupport / skill.totalDamage) * 100)}<span class="text-3xs text-gray-300">%</span>
        {:else}
            0.0<span class="text-3xs text-gray-300">%</span>
        {/if}
    </td>
{/if}
{#if anySupportBrand && currentSettings.breakdown.percentBrand}
    <td class="px-1 text-center">
        {#if skill.totalDamage > 0}
            {round((skill.debuffedBySupport / skill.totalDamage) * 100)}<span class="text-3xs text-gray-300">%</span>
        {:else}
            0.0<span class="text-3xs text-gray-300">%</span>
        {/if}
    </td>
{/if}
{#if anySupportIdentity && currentSettings.breakdown.percentIdentityBySup}
    <td class="px-1 text-center">
        {#if skill.totalDamage > 0}
            {round((skill.buffedByIdentity / skill.totalDamage) * 100)}<span class="text-3xs text-gray-300">%</span>
        {:else}
            0.0<span class="text-3xs text-gray-300">%</span>
        {/if}
    </td>
{/if}
{#if currentSettings.breakdown.avgDamage}
    {@const averagePerHit = skill.totalDamage / skill.hits}
    <td class="px-1 text-center" use:tooltip={{ content: Math.round(averagePerHit).toLocaleString() }}>
        {abbreviateNumberSplit(averagePerHit)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(averagePerHit)[1]}</span>
    </td>
    <td class="px-1 text-center" use:tooltip={{ content: Math.round(averagePerCast).toLocaleString() }}>
        {abbreviateNumberSplit(averagePerCast)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(averagePerCast)[1]}</span>
    </td>
{/if}
{#if currentSettings.breakdown.maxDamage}
    <td class="px-1 text-center" use:tooltip={{ content: skill.maxDamage.toLocaleString() }}>
        {abbreviateNumberSplit(skill.maxDamage)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(skill.maxDamage)[1]}</span>
    </td>
    {#if meterSettings === "logs"}
        {#if skill.maxDamageCast}
            <td class="px-1 text-center" use:tooltip={{ content: skill.maxDamageCast.toLocaleString() }}>
                {abbreviateNumberSplit(skill.maxDamageCast)[0]}<span class="text-3xs text-gray-300"
                    >{abbreviateNumberSplit(skill.maxDamageCast)[1]}</span>
            </td>
        {:else}
            <td class="px-1 text-center"> - </td>
        {/if}
    {/if}
{/if}
{#if currentSettings.breakdown.casts}
    <td
        class="px-1 text-center"
        use:tooltip={{
            content: `<div class="py-1">${
                skill.casts.toLocaleString() + " " + (skill.casts === 1 ? "cast" : "casts")
            }</div>`
        }}>
        {abbreviateNumberSplit(skill.casts)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(skill.casts)[1]}</span>
    </td>
{/if}
{#if currentSettings.breakdown.cpm}
    <td class="px-1 text-center">
        <div
            use:tooltip={{
                content: `<div class="py-1">${
                    skill.casts.toLocaleString() + " " + (skill.casts === 1 ? "cast" : "casts")
                }</div>`
            }}>
            {round(skill.casts / (duration / 1000 / 60))}
        </div>
    </td>
{/if}
{#if currentSettings.breakdown.hits}
    <td
        class="px-1 text-center"
        use:tooltip={{
            content: `<div class="py-1">${
                skill.hits.toLocaleString() + " " + (skill.hits === 1 ? "hit" : "hits")
            }</div>`
        }}>
        {abbreviateNumberSplit(skill.hits)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(skill.hits)[1]}</span>
    </td>
{/if}
{#if currentSettings.breakdown.hpm}
    <td class="px-1 text-center">
        {#if skill.hits === 0}
            <div class="">0</div>
        {:else}
            <div
                use:tooltip={{
                    content: `<div class="py-1">${
                        skill.hits.toLocaleString() + " " + (skill.hits === 1 ? "hit" : "hits")
                    }</div>`
                }}>
                {round(skill.hits / (duration / 1000 / 60))}
            </div>
        {/if}
    </td>
{/if}
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={shadow}
    style="background-color: {index % 2 === 1 && $settings.general.splitLines
        ? RGBLinearShade(HexToRgba(color, 0.6))
        : HexToRgba(color, 0.6)}; width: {width}%"></td>
