<script lang="ts">
  import type { Snippet } from "svelte";
  import "../app.css";
  import ThemeProvider from "./ThemeProvider.svelte";

  interface Props {
    children?: Snippet;
  }

  let { children }: Props = $props();
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />
<ThemeProvider />
{@render children?.()}
