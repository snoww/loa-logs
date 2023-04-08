<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import type { Entity, StatusEffect } from "$lib/types";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import LogBuffBreakdownRow from "./LogBuffBreakdownRow.svelte";

    export let groupedSynergies: Map<string, Map<number, StatusEffect>>;
    export let player: Entity;

    let color: string;
    let skillDamagePercentages: Array<number> = [];


    async function processSkills() {
        let skills = Object.values(player.skills).sort((a, b) => b.totalDamage - a.totalDamage);

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
        }         

        return skills;
    }

</script>

{#await processSkills() then skills}
{#each skills as skill, i (skill.id)}
<tr class="h-7 px-2 py-1 text-3xs">
    <LogBuffBreakdownRow groupedSynergies={groupedSynergies} skill={skill} color={color} damagePercentage={skillDamagePercentages[i]} />
</tr>
{/each}
    
{/await}