<script lang="ts" module>
  export { damageValue, percentValue, skillTooltip, middot, badTooltip, fadTooltip };
</script>

<script lang="ts">
  import { EntityState } from "$lib/entity.svelte";
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

{#snippet badTooltip(state: EntityState)}
  <span>
    Raw Back Attack
    {@render percentValue(state.baPercentage)}
  </span>
{/snippet}

{#snippet fadTooltip(state: EntityState)}
  <span>
    Raw Front Attack
    {@render percentValue(state.faPercentage)}
  </span>
{/snippet}
