<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { classesMap } from "$lib/constants/classes";
  import type { StatusEffect } from "$lib/types";
  import { getSkillIcon, removeUnknownHtmlTags } from "$lib/utils";

  const {
    buff,
    sourceSkillIcon,
    size = "size-5"
  }: { buff: StatusEffect; sourceSkillIcon?: boolean; size?: string } = $props();

  let icon = $derived.by(() => {
    if (sourceSkillIcon && buff.source.skill) {
      return buff.source.skill.icon;
    }
    return buff.source.icon;
  });
</script>

{#snippet tooltip()}
  <div class="flex flex-col gap-0.5 py-1 text-sm font-normal">
    {#if buff.source.skill}
      <div class="flex">
        <div>
          {classesMap[buff.source.skill.classId]}:
        </div>
        <img src={getSkillIcon(buff.source.skill.icon)} alt={buff.source.skill.name} class="mx-1 size-5 shrink-0" />
        <div>{buff.source.skill.name}</div>
      </div>
    {:else}
      <div class="flex gap-1">
        {#if buff.buffCategory === "set" && buff.source.setName}
          [Set] {buff.source.setName}:
        {:else if buff.buffCategory === "bracelet"}
          [Bracelet]
        {:else if buff.buffCategory === "elixir"}
          [Elixir]
        {:else if buff.buffCategory === "battleitem"}
          [Battle Item]
        {:else if buff.buffCategory === "dropsofether"}
          [Drops of Ether]
        {/if}
        <div>
          {@html removeUnknownHtmlTags(buff.source.name)}
        </div>
      </div>
    {/if}

    <div class="flex items-center tracking-tight">
      <img src={getSkillIcon(buff.source.icon)} alt={buff.name} class="mr-1 size-5 shrink-0" />
      <div>
        {@html removeUnknownHtmlTags(buff.source.desc)}
      </div>
    </div>
  </div>
{/snippet}

<QuickTooltip {tooltip}>
  <img src={getSkillIcon(icon)} alt={buff.name} class="table-cell {size} shrink-0" />
</QuickTooltip>
