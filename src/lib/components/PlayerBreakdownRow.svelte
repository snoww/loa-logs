<script lang="ts">
    import type { Skill } from "$lib/types";
    import { HexToRgba } from "$lib/utils/colors";
    import { cubicOut } from "svelte/easing";
    import { tweened } from "svelte/motion";


    export let skill: Skill;
    export let color: string;
    export let hasFrontAttacks: boolean;
    export let hasBackAttacks: boolean;
    export let abbreviatedSkillDamage: (string | number)[];
    export let skillDps: (string | number)[];
    export let playerDamageDealt: number;
    export let damagePercentage: number;

    const tweenedValue = tweened(0, {
        duration: 400,
        easing: cubicOut
    })

    let critPercentage = "0.0";
    let baPercentage = "0.0";
    let faPercentage = "0.0";

    $: {
        tweenedValue.set(damagePercentage);
        if (skill.hits !== 0) {
            critPercentage = (skill.crits / skill.hits * 100).toFixed(1);
            faPercentage = (skill.frontAttacks / skill.hits * 100).toFixed(1) ;
            baPercentage = (skill.backAttacks / skill.hits * 100).toFixed(1);
        }
    }

</script>


    <td class="px-2 truncate">
        {skill.name}
    </td>
    <td class="px-1 text-center">
        {abbreviatedSkillDamage[0]}<span class="text-3xs text-gray-300">{abbreviatedSkillDamage[1]}</span>
    </td>
    <td class="px-1 text-center">
        {skillDps[0]}<span class="text-3xs text-gray-300">{skillDps[1]}</span>
    </td>
    <td class="px-1 text-center">
        {(skill.totalDamage / playerDamageDealt * 100).toFixed(1)}<span class="text-xs text-gray-300">%</span>
    </td>
    <td class="px-1 text-center">
        {critPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
    {#if hasFrontAttacks}
    <td class="px-1 text-center">
        {faPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
    {/if}
    {#if hasBackAttacks}
    <td class="px-1 text-center">
        {baPercentage}<span class="text-3xs text-gray-300">%</span>
    </td>
    {/if}
    <td class="px-1 text-center">
        {skill.casts.toLocaleString()}
    </td>
    <div class="absolute left-0 h-7 px-2 py-1 -z-10"
        style="background-color: {HexToRgba(color, 0.6)}; width: {$tweenedValue}%"
    ></div>
