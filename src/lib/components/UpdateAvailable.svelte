<script lang="ts">
  import { settings, updateInfo } from "$lib/stores.svelte.js";
  import { createDialog, melt } from "@melt-ui/svelte";
  import { fade } from "svelte/transition";
  import { markdown } from "./Markdown.svelte";
  import { checkNinevehRunning, installBetaUpdate, relaunchApp } from "$lib/api";

  const {
    elements: { portalled, overlay, content, title, description },
    states: { open }
  } = createDialog();

  let ninevehWarning = $state(false);
  let installing = $state(false);

  $effect(() => {
    if (updateInfo.available) {
      $open = true;
    }
  });

  async function doUpdate() {
    installing = true;
    if (updateInfo.isBeta) {
      await installBetaUpdate();
    }
    await relaunchApp();
  }

  async function onUpdateClick() {
    const ninevehRunning = await checkNinevehRunning();
    if (ninevehRunning) {
      ninevehWarning = true;
    } else {
      await doUpdate();
    }
  }

  async function onConfirmUpdate() {
    await doUpdate();
  }
</script>

{#if $open}
  <div use:melt={$portalled}>
    <div use:melt={$overlay} class="fixed inset-0 z-50 bg-black/50" transition:fade={{ duration: 150 }}></div>
    <div
      class="fixed top-1/2 left-1/2 z-50 max-h-[85vh] w-[40rem] max-w-[60rem] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-neutral-800/40 p-4 shadow-lg drop-shadow-xl backdrop-blur-xl
      {settings.app.general.accentColor} flex flex-col items-center gap-4 text-white"
      use:melt={$content}
    >
      <h2 use:melt={$title} class="sticky top-0 py-2 text-xl font-semibold">New Update Available!</h2>
      {#if ninevehWarning}
        <p use:melt={$description} class="text-center text-sm text-neutral-300">
          Updating will close any active game connections. Do you want to proceed?
        </p>
        <div class="flex items-center gap-3 py-2">
          <button
            class="rounded-md bg-neutral-600/70 px-2 py-1 hover:bg-neutral-600/60 focus:ring-0"
            onclick={() => (ninevehWarning = false)}
          >
            Cancel
          </button>
          <button
            class="rounded-md bg-accent-500/70 px-2 py-1 hover:bg-accent-500/60 focus:ring-0 disabled:opacity-50"
            disabled={installing}
            onclick={onConfirmUpdate}
          >
            {installing ? "Updating..." : "Update Now"}
          </button>
        </div>
      {:else}
        {#if updateInfo.manifest?.body}
          <div use:melt={$description} class="overflow-y-scroll rounded-md border border-neutral-700">
            {@render markdown(updateInfo.manifest.body)}
          </div>
        {/if}
        <div class="flex items-center py-2">
          <button
            class="rounded-md bg-accent-500/70 px-2 py-1 hover:bg-accent-500/60 focus:ring-0 disabled:opacity-50"
            disabled={installing}
            onclick={onUpdateClick}
          >
            {installing ? "Updating..." : "Update Now"}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
