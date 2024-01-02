<script lang="ts">
    import { MiniSkill, type Skill } from "$lib/types";
    import { getOpenerSkills } from "$lib/utils/dpsCharts";
    import { skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";
    import { menuTooltip, tooltip } from "$lib/utils/tooltip";

    export let skills: { [skillId: number]: Skill };

    let skillsArray = Object.values(skills)
        .sort((a, b) => b.totalDamage - a.totalDamage)
        .filter(
            (skill) =>
                !skill.name.includes("(Summon)") &&
                skill.name !== "Weapon Attack" &&
                !skill.name.includes("Basic Attack") &&
                skill.name !== "Bleed" &&
                skill.castLog.length > 0
        )
        .map((skill) => {
            return new MiniSkill(skill.name, skill.icon, [...skill.castLog]);
        });


    let openerSkills = getOpenerSkills(skillsArray, 15);
</script>

<div class="mt-2 mb-4">
    <div class="text-lg font-medium flex justify-start">
        <div use:menuTooltip={{content: "First 15 skills casted"}}>
            Opener Rotation
        </div>
    </div>
    <div class="flex flex-wrap pt-2 items-center" style="width: calc(100vw - 4.5rem);">
        {#each openerSkills as skill, i (i)}
            <div use:tooltip={{ content: skill.name }}>
                <img class="m-1 h-10 w-10 rounded-sm" src={$skillIcon.path + getSkillIcon(skill.icon)} alt={skill.name} />
            </div>
            {#if i < openerSkills.length - 1}
            <svg class="mx-2 size-5 fill-gray-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="m305.5-62.5-78-79 341-340.5-341-341 78-78.5L725-482 305.5-62.5Z"/></svg>
            {/if}
        {/each}
    </div>
</div>
