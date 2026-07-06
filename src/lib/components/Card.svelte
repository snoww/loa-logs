<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    children,
    class: extraClass = "",
    style = "",
    self = $bindable()
  }: { children: Snippet; class?: string; style?: string; self?: HTMLElement } = $props();
</script>

<div
  bind:this={self}
  class="flex flex-col divide-y divide-neutral-950/80 rounded-xs bg-neutral-800 shadow-sm shadow-neutral-800 {extraClass}"
  {style}
>
  {@render children()}
</div>
