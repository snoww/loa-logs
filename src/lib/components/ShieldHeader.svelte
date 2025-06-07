<script lang="ts">
  import type { StatusEffect } from "$lib/types";
  import BuffTooltip from "./tooltips/BuffTooltip.svelte";

  interface Props {
    shields: Map<number, StatusEffect>;
  }

  let { shields }: Props = $props();

  let width = $derived(shields.size > 1 ? `${shields.size * 1.5 + 1}rem` : "3.5rem");
</script>

<th style="width: {width}">
  <div class="flex justify-center space-x-1">
    {#each [...shields] as [id, synergy] (id)}
      <BuffTooltip buff={synergy} sourceSkillIcon={true} />
    {/each}
  </div>
</th>
