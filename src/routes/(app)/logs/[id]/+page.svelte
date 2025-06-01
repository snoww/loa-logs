<script lang="ts">
  import LogDamageMeter from "$lib/components/logs/LogDamageMeter.svelte";
  import type { PageData } from "./$types";
  import Header from "./Header.svelte";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();
  let encounter = $derived(data);
</script>

<Header {encounter} />
<div class="mx-auto flex max-w-[100rem] flex-col gap-2 px-2 py-4 sm:px-4">
  <LogDamageMeter {encounter} />
</div>
