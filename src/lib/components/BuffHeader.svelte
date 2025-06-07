<script lang="ts">
  import type { StatusEffect } from "$lib/types";
  import BuffTooltip from "./tooltips/BuffTooltip.svelte";

  interface Props {
    buffs: Map<number, StatusEffect>;
  }

  let { buffs: buffs }: Props = $props();
  let width = $derived(`${Math.max(3.5, buffs.size * 1.5 + 1)}rem`);
</script>

<th style="width: {width}">
  <div class="flex justify-center space-x-1">
    {#each [...buffs] as [id, buff] (id)}
      <BuffTooltip {buff} />
    {/each}
  </div>
</th>
