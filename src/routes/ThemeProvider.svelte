<script lang="ts">
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { settings } from "$lib/stores.svelte";
  import type { FontScale } from "$lib/settings";

  const currentWindow = getCurrentWebviewWindow();
  const scaleMap: Record<FontScale, string> = {
    "0": "small",
    "1": "medium",
    "2": "large",
    "3": "x-large"
  };

  $effect(() => {
    const documentElement = document.documentElement;
    const general = settings.app.general;

    if(currentWindow.label === "logs") {
      const fontSize = scaleMap[general.logScale] ?? "medium";
      documentElement.style.setProperty("--font-size", fontSize);
    }
    else {
      const fontSize = scaleMap[general.scale] ?? "medium";
      documentElement.style.setProperty("--font-size", fontSize);
    }

    const palette = general.accentColor.split("-")[1];
    documentElement.dataset.theme = palette;
    documentElement.dataset.view = currentWindow.label;
  });

</script>