<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  import { goto, invalidateAll } from "$app/navigation";
  import { page } from "$app/state";
  import { settings, updateSettings } from "$lib/utils/settings";
  import { invoke } from "@tauri-apps/api";
  import { checkUpdate } from "@tauri-apps/api/updater";
  import { appWindow } from "@tauri-apps/api/window";
  interface Props {
    children?: import("svelte").Snippet;
  }

  let { children }: Props = $props();

  let events: Set<UnlistenFn> = new Set();

  onMount(() => {
    (async () => {
      await checkForUpdate();

      if ($updateSettings.available) {
        await showWindow();
      }

      let encounterUpdateEvent = await listen("show-latest-encounter", async (event) => {
        await goto("/logs/encounter/" + event.payload);
        await showWindow();
      });
      let openUrlEvent = await listen("redirect-url", async (event) => {
        await invalidateAll();
        await goto("/" + event.payload);
        await showWindow();
      });

      events.add(encounterUpdateEvent);
      events.add(openUrlEvent);

      setInterval(checkForUpdate, 60 * 15 * 1000);
    })();
  });
  onDestroy(() => {
    events.forEach((unlisten) => unlisten());
  });

  async function showWindow() {
    await appWindow.show();
    await appWindow.unminimize();
    await appWindow.setFocus();
  }

  async function checkForUpdate() {
    try {
      const { shouldUpdate, manifest } = await checkUpdate();
      if (shouldUpdate) {
        $updateSettings.available = true;
        const oldManifest = $updateSettings.manifest;
        $updateSettings.manifest = manifest;
        if (oldManifest?.version !== $updateSettings.manifest?.version) {
          $updateSettings.dismissed = false;
        }
        $updateSettings.isNotice = !!manifest?.version.includes("2025");
      }
    } catch (e) {
      await invoke("write_log", { message: e });
    }
  }

  $effect.pre(() => {
    if ($settings.general.logScale === "1") {
      document.documentElement.style.setProperty("font-size", "medium");
    } else if ($settings.general.logScale === "2") {
      document.documentElement.style.setProperty("font-size", "large");
    } else if ($settings.general.logScale === "3") {
      document.documentElement.style.setProperty("font-size", "x-large");
    } else if ($settings.general.logScale === "0") {
      document.documentElement.style.setProperty("font-size", "small");
    }
  });
</script>

{@render children?.()}
