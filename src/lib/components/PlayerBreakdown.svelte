<script lang="ts">
    import { EntityType, type Entity, type Skill } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { flip } from "svelte/animate";
    import PlayerBreakdownRow from "./PlayerBreakdownRow.svelte";
    import { colors, settings } from "$lib/utils/settings";
    import PlayerBreakdownHeader from "./shared/PlayerBreakdownHeader.svelte";
    import { cardIds } from "$lib/constants/cards";
    import { localPlayer } from "$lib/utils/stores";

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
    let anySupportIdentity = false;
    let anySupportBuff = false;

    $: {
        if (entity) {
            if (Object.hasOwn($colors, entity.class)) {
                if ($settings.general.constantLocalPlayerColor && $localPlayer == entity.name) {
                    color = $colors["Local"].color;
                } else {
                    color = $colors[entity.class].color;
                }
            } else if (entity.entityType === EntityType.ESTHER) {
                color = "#4dc8d0";
            }
            skills = Object.values(entity.skills).sort((a, b) => b.totalDamage - a.totalDamage);
            if (entity.class === "Arcanist") {
                skills = skills.filter((skill) => !cardIds.includes(skill.id));
            }
            if (skills.length > 0) {
                let mostDamageSkill = skills[0].totalDamage;
                skillDamagePercentages = skills.map((skill) => (skill.totalDamage / mostDamageSkill) * 100);
                abbreviatedSkillDamage = skills.map((skill) => abbreviateNumberSplit(skill.totalDamage));
                skillDps = skills.map((skill) => abbreviateNumberSplit(skill.totalDamage / (duration / 1000)));
                hasBackAttacks = skills.some((skill) => skill.backAttacks > 0);
                hasFrontAttacks = skills.some((skill) => skill.frontAttacks > 0);
                anySupportBuff = skills.some((skill) => skill.buffedBySupport > 0);
                anySupportIdentity = skills.some((skill) => skill.buffedByIdentity > 0);
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
            {anySupportIdentity}
            {anySupportBrand} />
    </tr>
</thead>
<tbody on:contextmenu|preventDefault={handleRightClick} class="relative z-10">
    {#if entity}
        {#each skills as skill, i (skill.id)}
            <tr class="h-7 px-2 py-1 text-3xs {$settings.general.underlineHovered ? 'hover:underline' : ''}" animate:flip={{ duration: 200 }}>
                <PlayerBreakdownRow
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
                    {duration} 
                    index={i}/>
            </tr>
        {/each}
    {/if}
</tbody>
