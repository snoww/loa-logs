<script lang="ts">
  import { encounterFilter, settings } from "$lib/stores.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import "../app.css";
  import { onMount } from "svelte";
  import type { AppSettings } from "$lib/settings";
  import Loader from "./loader.svelte";

  interface Props {
    children?: import("svelte").Snippet;
  }

  interface LoadResult {
    settings: AppSettings;
  }

  let { children }: Props = $props();
  let isLoading = $state(true);

  onMount(async () => {
    const loadResult = await invoke<LoadResult>("load");
    settings.set(loadResult.settings);
    encounterFilter.setMinDuration(loadResult.settings.logs.minEncounterDuration)
    isLoading = false;

  });
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />
  {#if isLoading}
    <Loader />
  {:else}
    <div class="{settings.app.general.accentColor} text-sm text-white">
      {@render children?.()}
    </div>
  {/if}
