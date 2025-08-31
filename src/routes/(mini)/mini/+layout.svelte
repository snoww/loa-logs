<script lang="ts">
  import { settings } from "$lib/stores.svelte";
  import { setup } from "$lib/utils/setup";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
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
    const appWindow = getCurrentWebviewWindow();
    if (settings.app.general.miniEdit) {
      appWindow.setIgnoreCursorEvents(false);
    } else {
      appWindow.setIgnoreCursorEvents(true);
    }
  });

  $effect.pre(() => {
    if (!settings.app.general.mini) {
      getCurrentWebviewWindow().hide();
    }
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
