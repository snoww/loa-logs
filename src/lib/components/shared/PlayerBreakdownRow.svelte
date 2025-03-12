<script lang="ts">
    import type { EntityState } from "$lib/entity.svelte";
    import { SkillState } from "$lib/skill.svelte";
    import type { Skill } from "$lib/types";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { abbreviateNumberSplit, round } from "$lib/utils/numbers";
    import { settings, skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";
    import { generateSkillTooltip, tooltip } from "$lib/utils/tooltip";
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";

    interface Props {
        skill: Skill;
        entityState: EntityState;
        width: number;
        shadow?: boolean;
        index: number;
    }

    let { skill, entityState, width, shadow = false, index }: Props = $props();

    let skillState = $derived(new SkillState(skill, entityState));
    let tweenedValue = new Tween(entityState.enc.live ? 0 : width, {
        duration: 400,
        easing: cubicOut
    });
    $effect(() => {
        tweenedValue.set(width ?? 0);
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
{#if entityState.enc.curSettings.breakdown.damage}
    <td class="px-1 text-center" use:tooltip={{ content: skill.totalDamage.toLocaleString() }}>
        {skillState.skillDamageString[0]}<span class="text-3xs text-gray-300">{skillState.skillDamageString[1]}</span>
    </td>
{/if}
{#if entityState.enc.curSettings.breakdown.dps}
    <td class="px-1 text-center" use:tooltip={{ content: skillState.skillDps.toLocaleString() }}>
        {skillState.skillDpsString[0]}<span class="text-3xs text-gray-300">{skillState.skillDpsString[1]}</span>
    </td>
{/if}
{#if entityState.enc.curSettings.breakdown.damagePercent}
    <td class="px-1 text-center">
        {round((skill.totalDamage / entityState.damageDealt) * 100)}<span class="text-xs text-gray-300">%</span>
    </td>
{/if}
{#if entityState.enc.curSettings.breakdown.critRate}
    <td class="px-1 text-center">
        {skillState.critPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if !entityState.enc.live && entityState.enc.curSettings.breakdown.adjustedCritRate}
    <td class="px-1 text-center">
        {#if skillState.adjustedCrit}
            {skillState.adjustedCrit}<span class="text-3xs text-gray-300">%</span>
        {:else}
            -
        {/if}
    </td>
{/if}
{#if entityState.enc.curSettings.breakdown.critDmg}
    <td class="px-1 text-center">
        {skillState.critDmgPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if entityState.anyFrontAttacks && entityState.enc.curSettings.breakdown.frontAtk}
    <td class="px-1 text-center">
        {skillState.faPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if entityState.anyBackAttacks && entityState.enc.curSettings.breakdown.backAtk}
    <td class="px-1 text-center">
        {skillState.baPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
{/if}
{#if entityState.anySupportBuff && entityState.enc.curSettings.breakdown.percentBuffBySup}
    <td class="px-1 text-center">
        {#if skill.totalDamage > 0}
            {round((skill.buffedBySupport / skill.totalDamage) * 100)}<span class="text-3xs text-gray-300">%</span>
        {:else}
            0.0<span class="text-3xs text-gray-300">%</span>
        {/if}
    </td>
{/if}
{#if entityState.anySupportBrand && entityState.enc.curSettings.breakdown.percentBrand}
    <td class="px-1 text-center">
        {#if skill.totalDamage > 0}
            {round((skill.debuffedBySupport / skill.totalDamage) * 100)}<span class="text-3xs text-gray-300">%</span>
        {:else}
            0.0<span class="text-3xs text-gray-300">%</span>
        {/if}
    </td>
{/if}
{#if entityState.anySupportIdentity && entityState.enc.curSettings.breakdown.percentIdentityBySup}
    <td class="px-1 text-center">
        {#if skill.totalDamage > 0}
            {round((skill.buffedByIdentity / skill.totalDamage) * 100)}<span class="text-3xs text-gray-300">%</span>
        {:else}
            0.0<span class="text-3xs text-gray-300">%</span>
        {/if}
    </td>
{/if}
{#if entityState.anySupportHat && entityState.enc.curSettings.breakdown.percentHatBySup}
    <td class="px-1 text-center">
        {#if skill.totalDamage > 0}
            {round(((skill.buffedByHat ?? 0) / skill.totalDamage) * 100)}<span class="text-3xs text-gray-300">%</span>
        {:else}
            0.0<span class="text-3xs text-gray-300">%</span>
        {/if}
    </td>
{/if}
{#if entityState.enc.curSettings.breakdown.avgDamage}
    {@const averagePerHit = skill.totalDamage / skill.hits}
    <td class="px-1 text-center" use:tooltip={{ content: Math.round(averagePerHit).toLocaleString() }}>
        {abbreviateNumberSplit(averagePerHit)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(averagePerHit)[1]}</span>
    </td>
    <td class="px-1 text-center" use:tooltip={{ content: Math.round(skillState.averagePerCast).toLocaleString() }}>
        {abbreviateNumberSplit(skillState.averagePerCast)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(skillState.averagePerCast)[1]}</span>
    </td>
{/if}
{#if entityState.enc.curSettings.breakdown.maxDamage}
    <td class="px-1 text-center" use:tooltip={{ content: skill.maxDamage.toLocaleString() }}>
        {abbreviateNumberSplit(skill.maxDamage)[0]}<span class="text-3xs text-gray-300"
            >{abbreviateNumberSplit(skill.maxDamage)[1]}</span>
    </td>
    {#if !entityState.enc.live}
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
{#if entityState.enc.curSettings.breakdown.casts}
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
{#if entityState.enc.curSettings.breakdown.cpm}
    <td class="px-1 text-center">
        <div
            use:tooltip={{
                content: `<div class="py-1">${
                    skill.casts.toLocaleString() + " " + (skill.casts === 1 ? "cast" : "casts")
                }</div>`
            }}>
            {round(skill.casts / (entityState.enc.duration / 1000 / 60))}
        </div>
    </td>
{/if}
{#if entityState.enc.curSettings.breakdown.hits}
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
{#if entityState.enc.curSettings.breakdown.hpm}
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
                {round(skill.hits / (entityState.enc.duration / 1000 / 60))}
            </div>
        {/if}
    </td>
{/if}
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={shadow}
    style="background-color: {index % 2 === 1 && $settings.general.splitLines
        ? RGBLinearShade(HexToRgba(entityState.color, 0.6))
        : HexToRgba(entityState.color, 0.6)}; width: {tweenedValue.current}%"></td>
