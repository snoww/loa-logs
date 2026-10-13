<script lang="ts">
  import Toaster from "$lib/components/Toaster.svelte";
  import { settings } from "$lib/stores.svelte";
  import { setup } from "$lib/utils/setup";
  import { onMount } from "svelte";

  let { children }: { children?: import("svelte").Snippet } = $props();

  onMount(() => {
    setup();
  });
</script>

<Toaster />
<div
  class="live-meter h-screen min-h-screen overflow-hidden {settings.app.general.transparent
    ? 'bg-neutral-900/25'
    : 'bg-neutral-900/95'}"
>
  {@render children?.()}
</div>

<style lang="postcss">
  :global(.live-meter *, .noscroll) {
    -ms-overflow-style: none;
    scrollbar-width: auto;
  }
  :global(.live-meter *::-webkit-scrollbar, .noscroll::-webkit-scrollbar) {
    display: none;
  }
</style>
