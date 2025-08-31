<script lang="ts">
  import { goto } from "$app/navigation";
  import UpdateAvailable from "$lib/components/UpdateAvailable.svelte";
  import Toaster from "$lib/components/Toaster.svelte";
  import { settings } from "$lib/stores.svelte";
  import { checkForUpdate } from "$lib/utils";
  import { getVersion } from "@tauri-apps/api/app";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onDestroy, onMount } from "svelte";

  let { children }: { children?: import("svelte").Snippet } = $props();

  let events: Set<UnlistenFn> = new Set();

  onMount(() => {
    (async () => {
      let encounterUpdateEvent = await listen("show-latest-encounter", async (event) => {
        await goto("/logs/" + event.payload);
        await showWindow();
      });
      let openUrlEvent = await listen("redirect-url", async (event) => {
        await goto("/" + event.payload);
        await showWindow();
      });

      events.add(encounterUpdateEvent);
      events.add(openUrlEvent);

      let version = await getVersion();
      if (settings.version !== version) {
        settings.version = version;
        goto("/changelog");
        await showWindow();
      }
    })();

    // check for app updates
    const interval = setInterval(checkForUpdate, 60 * 15 * 1000);
    return () => {
      clearInterval(interval);
    };
  });
  onDestroy(() => {
    events.forEach((unlisten) => unlisten());
  });

  async function showWindow() {
    const appWindow = getCurrentWebviewWindow();
    await appWindow.show();
    await appWindow.unminimize();
    await appWindow.setFocus();
  }

  $effect.pre(() => {
    if (settings.app.general.logScale === "1") {
      document.documentElement.style.setProperty("font-size", "medium");
    } else if (settings.app.general.logScale === "2") {
      document.documentElement.style.setProperty("font-size", "large");
    } else if (settings.app.general.logScale === "3") {
      document.documentElement.style.setProperty("font-size", "x-large");
    } else if (settings.app.general.logScale === "0") {
      document.documentElement.style.setProperty("font-size", "small");
    }
  });
</script>

<UpdateAvailable />
<Toaster />
<div class="min-h-screen select-none bg-neutral-900">
  {@render children?.()}
</div>

<style lang="postcss">
  @reference "../../app.css";
  :global(*::-webkit-scrollbar) {
    @apply right-0! block! size-2! bg-neutral-800!;
  }
  :global(*::-webkit-scrollbar-thumb) {
    @apply rounded-md! bg-neutral-600!;
  }
  :global(*::-webkit-scrollbar-corner) {
    @apply bg-neutral-800!;
  }
</style>
