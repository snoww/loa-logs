<script lang="ts">
  import { settings } from "$lib/stores.svelte.js";
  import { createDialog, melt } from "@melt-ui/svelte";
  import { fade } from "svelte/transition";
  import { browser } from "$app/environment";
  import { BETA_MODAL_KEY } from "$lib/api";

  const {
    elements: { portalled, overlay, content, title, description, close },
    states: { open }
  } = createDialog({ forceVisible: true });

  if (browser && !localStorage.getItem(BETA_MODAL_KEY)) {
    $open = true;
  }

  function dismiss() {
    localStorage.setItem(BETA_MODAL_KEY, "true");
    $open = false;
  }
</script>

{#if $open}
  <div use:melt={$portalled}>
    <div use:melt={$overlay} class="fixed inset-0 z-50 bg-black/50" transition:fade={{ duration: 150 }}></div>
    <div
      use:melt={$content}
      class="fixed top-1/2 left-1/2 z-50 w-[36rem] max-w-[90vw] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-neutral-800/60 p-6 shadow-lg drop-shadow-xl backdrop-blur-xl
      {settings.app.general.accentColor} flex flex-col gap-4 text-white"
    >
      <h2 use:melt={$title} class="text-xl font-semibold">
        Welcome to LOA Logs <span class="text-accent-500">Beta</span>
      </h2>
      <div use:melt={$description} class="flex flex-col gap-2 text-sm text-neutral-300">
        <p>
          You're running a beta build of LOA Logs. This version includes experimental features and changes that are
          still being tested before a stable release.
        </p>
        <p>
          This version will
        </p>
        <p>
          Please report any issues on Discord.
        </p>
      </div>
      <div class="flex justify-end">
        <button
          use:melt={$close}
          class="rounded-md bg-accent-500/70 px-4 py-1.5 text-sm hover:bg-accent-500/60 focus:ring-0"
          onclick={dismiss}
        >
          Got it
        </button>
      </div>
    </div>
  </div>
{/if}
