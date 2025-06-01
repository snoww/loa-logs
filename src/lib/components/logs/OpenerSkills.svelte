<script lang="ts">
  import Card from "$lib/components/Card.svelte";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { IconChevronRight } from "$lib/icons";
  import type { Skill } from "$lib/types";
  import { getOpenerSkills } from "$lib/utils/dpsCharts";

  import { getSkillIcon } from "$lib/utils";

  interface Props {
    skills: Record<number, Skill>;
  }

  let { skills }: Props = $props();

  let skillsArray = $derived(
    Object.values(skills)
      .sort((a, b) => b.totalDamage - a.totalDamage)
      .filter(
        (skill) =>
          !skill.name.includes("(Summon)") &&
          skill.name !== "Weapon Attack" &&
          !skill.name.includes("Basic Attack") &&
          skill.name !== "Bleed" &&
          skill.castLog.length > 0
      )
  );

  let openerSkills = $derived(getOpenerSkills(skillsArray, 15));
</script>

<Card class="mt-4">
  <div class="flex bg-black/10 px-3 py-2 font-medium">
    <QuickTooltip tooltip="First 15 skills caste">
      <div>Opener Rotation</div>
    </QuickTooltip>
  </div>
  <div class="flex flex-wrap items-center p-2">
    {#each openerSkills as skill, i (i)}
      <QuickTooltip tooltip={skill.name}>
        <img class="rounded-xs m-1 h-10 w-10" src={getSkillIcon(skill.icon)} alt={skill.name} />
      </QuickTooltip>
      {#if i < openerSkills.length - 1}
        <IconChevronRight class="mx-2 size-5 text-neutral-400" />
      {/if}
    {/each}
  </div>
</Card>
