<script lang="ts" module>
  export { damageValue, percentValue, skillTooltip, middot };
</script>

<script lang="ts">
  import type { Skill } from "$lib/types";
  import SkillTooltip from "./tooltips/SkillTooltip.svelte";
</script>

{#snippet middot()}
  <div class="mx-1 text-neutral-400">&middot;</div>
{/snippet}

{#snippet skillTooltip(skill: Skill)}
  <SkillTooltip {skill} />
{/snippet}

<!-- Render value + units -->
{#snippet damageValue(val: [number, string])}
  {val[0]}<span class="text-xxs text-gray-300">{val[1]}</span>
{/snippet}

<!-- Render value + percent -->
{#snippet percentValue(val: string | number)}
  {val}<span class="text-xxs text-gray-300">%</span>
{/snippet}
