<script lang="ts">
  import { settings } from "$lib/stores.svelte";
  import { setup } from "$lib/utils/setup";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  let { children }: { children?: import("svelte").Snippet } = $props();

  onMount(() => {
    setup();
  });

  $effect.pre(() => {
    if (settings.app.general.scale === "1") {
      document.documentElement.style.setProperty("font-size", "medium");
    } else if (settings.app.general.scale === "2") {
      document.documentElement.style.setProperty("font-size", "large");
    } else if (settings.app.general.scale === "3") {
      document.documentElement.style.setProperty("font-size", "x-large");
    } else if (settings.app.general.scale === "0") {
      document.documentElement.style.setProperty("font-size", "small");
    }
  });

  $effect(() => {

    (async () => {
      
      const appWindow = getCurrentWindow();

      if (settings.app.general.miniEdit) {
        await appWindow.setIgnoreCursorEvents(false);
      } else {
        await appWindow.setIgnoreCursorEvents(true);
      }
      
    })()
  });

  $effect.pre(() => {
    (async () => {
      if (!settings.app.general.mini) {
        const appWindow = getCurrentWindow();
        await appWindow.hide();
      }
    })()
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  oncontextmenu={(e) => e.preventDefault()}
  class="flex h-screen w-full select-none flex-col gap-2 overflow-hidden {settings.app.general.miniEdit
    ? 'rounded border-2 border-red-500 '
    : ''}"
  class:hidden={!settings.app.general.mini}
>
  {@render children?.()}
</div>
