<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import "../app.css";
  import ThemeProvider from "./ThemeProvider.svelte";
  import { appContext } from "$lib/context.svelte";
  import { getAppVersion } from "$lib/api";
  interface Props {
    children?: Snippet;
  }

  let { children }: Props = $props();

  onMount(async () => {
    $appContext.version = await getAppVersion();
  })
  

</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />
<ThemeProvider/>
{@render children?.()}
