<script lang="ts">
  import { settings } from "$lib/stores.svelte";
  import "../app.css";

  interface Props {
    children?: import("svelte").Snippet;
  }

  let { children }: Props = $props();
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />
<div class="{settings.app.general.accentColor} text-sm text-white">
  {@render children?.()}
</div>
