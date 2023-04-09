<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import type { Entity, Skill } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import LogPlayerBreakdownRow from "./LogPlayerBreakdownRow.svelte";

    export let player: Entity;
    export let duration: number;
    export let handleRightClick: () => void;

    let color = "#ffffff";
    let skills: Array<Skill> = [];
    let skillDamagePercentages: Array<number> = [];
    let abbreviatedSkillDamage: Array<(string | number)[]> = [];
    let skillDps: Array<(string | number)[]> = [];
    
    let hasBackAttacks = true;
    let hasFrontAttacks = true;

    async function processSkills() {
        skills = Object.values(player.skills).sort((a, b) => b.totalDamage - a.totalDamage);

        for (let skill of skills) {
            if (skill.icon.startsWith("http")) {
                continue;
            }
            let fileName;
            if (skill.icon) {
                fileName = skill.icon;
            } else {
                fileName = "unknown.png";
            }
            skill.icon = convertFileSrc(await join(await resourceDir(), 'images', 'skills', fileName));
        }
        if (Object.hasOwn(classColors, player.class)){
            color = classColors[player.class].color;
        }
    
        if (skills.length > 0) {
            let mostDamageSkill = skills[0].totalDamage;
            skillDamagePercentages = skills.map(skill => (skill.totalDamage / mostDamageSkill) * 100);
            abbreviatedSkillDamage = skills.map(skill => abbreviateNumberSplit(skill.totalDamage));
            skillDps = skills.map(skill => abbreviateNumberSplit(skill.totalDamage / (duration / 1000)));
            hasBackAttacks = skills.some(skill => skill.backAttacks > 0);
            hasFrontAttacks = skills.some(skill => skill.frontAttacks > 0);
        }

        return skills;
    }    
</script>

<thead class="h-6 z-30" on:contextmenu|preventDefault={() => {console.log("titlebar clicked")}}>
    <tr class="bg-zinc-900">
        <th class="text-left px-2 font-normal w-full"></th>
        <th class="font-normal w-14">DMG</th>
        <th class="font-normal w-14">DPS</th>
        <th class="font-normal w-14">D%</th>
        <th class="font-normal w-14">Crit</th>
        {#if hasFrontAttacks}
        <th class="font-normal w-14">F.A</th>
        {/if}
        {#if hasBackAttacks}
        <th class="font-normal w-14">B.A</th>
        {/if}
        <th class="font-normal w-14">Casts</th>
    </tr>
</thead>
<tbody on:contextmenu|preventDefault={handleRightClick}>
    {#await processSkills() then skills}
    {#each skills as skill, i (skill.id)}
    <tr class="h-7 px-2 py-1 text-3xs">
        <LogPlayerBreakdownRow
            skill={skill}
            color={color}
            hasFrontAttacks={hasFrontAttacks}
            hasBackAttacks={hasBackAttacks} 
            abbreviatedSkillDamage={abbreviatedSkillDamage[i]}
            playerDamageDealt={player.damageStats.damageDealt}
            damagePercentage={skillDamagePercentages[i]}
            skillDps={skillDps[i]}
            />
    </tr>
    {/each}
    {/await}
</tbody>

