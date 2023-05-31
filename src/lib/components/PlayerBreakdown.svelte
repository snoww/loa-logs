<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import { EntityType, type Entity, type Skill } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { flip } from "svelte/animate";
    import PlayerBreakdownRow from "./PlayerBreakdownRow.svelte";
    import { settings } from "$lib/utils/settings";
    import PlayerBreakdownHeader from "./shared/PlayerBreakdownHeader.svelte";

    export let entity: Entity | null;
    export let duration: number;
    export let handleRightClick: () => void;

    let color = "#ffffff";
    let skills: Array<Skill> = [];
    let skillDamagePercentages: Array<number> = [];
    let abbreviatedSkillDamage: Array<(string | number)[]> = [];
    let skillDps: Array<(string | number)[]> = [];

    let hasBackAttacks = true;
    let hasFrontAttacks = true;
    let anySupportBrand = false;
    let anySupportBuff = false;

    $: {
        if (entity) {
            if (Object.hasOwn(classColors, entity.class)) {
                color = classColors[entity.class].color;
            } else if (entity.entityType === EntityType.ESTHER) {
                color = "#4dc8d0";
            }
            skills = Object.values(entity.skills).sort((a, b) => b.totalDamage - a.totalDamage);
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
        }
    }
</script>

<thead class="sticky top-0 z-40 h-6">
    <tr class="bg-zinc-900 tracking-tighter">
        <PlayerBreakdownHeader
            meterSettings={$settings.meter}
            {hasFrontAttacks}
            {hasBackAttacks}
            {anySupportBuff}
            {anySupportBrand} />
    </tr>
</thead>
<tbody on:contextmenu|preventDefault={handleRightClick} class="relative z-10">
    {#if entity}
        {#each skills as skill, i (skill.id)}
            <tr class="h-7 px-2 py-1 text-3xs" animate:flip={{ duration: 200 }}>
                <PlayerBreakdownRow
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
                    {duration} />
            </tr>
        {/each}
    {/if}
</tbody>
