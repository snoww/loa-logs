<script lang="ts">
  import { settings, updateInfo } from "$lib/stores.svelte.js";
  import { createDialog, melt } from "@melt-ui/svelte";
  import { fade } from "svelte/transition";
  import { markdown } from "./Markdown.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { relaunch } from "@tauri-apps/plugin-process";

  const {
    elements: { portalled, overlay, content, title, description },
    states: { open }
  } = createDialog();

  $effect(() => {
    if (updateInfo.available) {
      $open = true;
    }
  });
</script>

{#if $open}
  <div use:melt={$portalled}>
    <div use:melt={$overlay} class="fixed inset-0 z-50 bg-black/50" transition:fade={{ duration: 150 }}></div>
    <div
      class="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[40rem] max-w-[60rem] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-neutral-800/40 p-4 shadow-lg drop-shadow-xl backdrop-blur-xl
      {settings.app.general.accentColor} flex flex-col items-center gap-4 text-white"
      use:melt={$content}
    >
      <h2 use:melt={$title} class="sticky top-0 py-2 text-xl font-semibold">New Update Available!</h2>
      {#if updateInfo.manifest?.body}
        <div use:melt={$description} class="overflow-y-scroll rounded-md border border-neutral-700">
          {@render markdown(updateInfo.manifest.body)}
        </div>
      {/if}
      <div class="flex items-center py-2">
        <button
          class="bg-accent-500/70 hover:bg-accent-500/60 rounded-md px-2 py-1 focus:ring-0"
          onclick={async () => {
            await invoke("unload_driver");
            await invoke("remove_driver");
            await relaunch();
          }}
        >
          <span>Update Now</span>
        </button>
      </div>
    </div>
  </div>
{/if}
