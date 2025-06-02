<script lang="ts">
  import { setup } from "$lib/utils/setup";
  import { onMount } from "svelte";

  let { children }: { children?: import("svelte").Snippet } = $props();

  onMount(() => {
    setup();
  });
</script>

<div class="live-meter h-screen min-h-screen overflow-hidden">
  {@render children?.()}
</div>
