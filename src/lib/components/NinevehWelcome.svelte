<script lang="ts">
  import { settings } from "$lib/stores.svelte.js";
  import { createDialog, melt } from "@melt-ui/svelte";
  import { fade } from "svelte/transition";
  import { browser } from "$app/environment";
  import { getLastEncounterVersion, NINEVEH_MIGRATION_MODAL_KEY } from "$lib/api";
  import { onMount } from "svelte";

  const {
    elements: { portalled, overlay, content, title, description, close },
    states: { open }
  } = createDialog({ forceVisible: true });

  onMount(() => {
    if (!browser) return;
    if (localStorage.getItem(NINEVEH_MIGRATION_MODAL_KEY)) return;

    (async () => {
      const lastVersion = await getLastEncounterVersion();
      if (!lastVersion) return;

      // fucky semver parsing
      const mainVer = lastVersion.split("-")[0];
      const [major, minor, _patch] = mainVer.split(".", 3).map(Number);
      if (major > 1 || (major === 1 && minor > 44)) return; // most recent log was after nineveh migration

      $open = true;
    })();
  });

  function dismiss() {
    localStorage.setItem(NINEVEH_MIGRATION_MODAL_KEY, "true");
    $open = false;
  }
</script>

{#snippet section(name: string, description: string, icon: string, iconRight: boolean)}
  <div class="flex flex-col gap-1 text-sm text-neutral-300">
    <div class="text-lg font-bold">{name}</div>
    <div class="flex flex-row items-center gap-4" class:flex-row-reverse={iconRight}>
      <img src="https://cdn.ags.lol/icon/{icon}.png" alt="" class="size-16" />
      <div class="flex-1">{@html description}</div>
    </div>
  </div>
{/snippet}

{#if $open}
  <div use:melt={$portalled}>
    <div use:melt={$overlay} class="fixed inset-0 z-50 bg-black/50" transition:fade={{ duration: 150 }}></div>
    <div
      use:melt={$content}
      class="fixed top-1/2 left-1/2 z-50 w-[36rem] max-w-[90vw] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-neutral-800/60 p-6 shadow-lg drop-shadow-xl backdrop-blur-xl
      {settings.app.general.accentColor} flex max-h-[90vh] flex-col gap-4 overflow-y-auto text-white"
    >
      <h2 use:melt={$title} class="text-xl font-semibold">
        Welcome to <span class="text-accent-500">Nineveh</span>!
      </h2>
      <div use:melt={$description} class="flex flex-col gap-6 text-sm text-neutral-300">
        <p>
          Starting with LOA Logs v1.45.0 and the "Twilight Isle" game update, LOA Logs is now using a new game traffic
          engine, codenamed <span class="text-accent-500">Nineveh</span>.
        </p>

        {@render section(
          "Why Nineveh?",
          "With Nineveh, LOA Logs has more control over the game traffic. This makes it easier for us to add new features and fixes rare issues, like garbled damage numbers. It also lets us support Linux!",
          "emoji_a_01_34",
          false
        )}
        {@render section(
          "What's changed?",
          "Everything still works as before, with some slight changes. The big one is that you will now need to launch the meter <b>before you select a server</b>. This lets us capture the game traffic right from the start. If you're already in-game, you will need to head back to the main menu.",
          "emoji_a_01_36",
          true
        )}
        {@render section(
          "What's new?",
          "Now that we have Nineveh, we were able to add a ton of new features that we couldn't do before. The biggest one is <b>raid DPS</b>. Logs will now break down all of your damage, so you can see exactly how much of your damage is yours, and how much damage is from your party.",
          "emoji_a_01_35",
          false
        )}
        {@render section(
          "What if things break?",
          "A dedicated group of betatesters has been hard at work validating Nineveh over the past months, and we're pretty confident things are stable. That said, if you encounter any issues please let us know on <a class='underline' target='_blank' href='https://discord.gg/RXvTMV2YHu'>Discord</a>!",
          "emoji_a_01_40",
          true
        )}
      </div>
      <div class="flex justify-end">
        <button
          use:melt={$close}
          class="rounded-md bg-accent-500/70 px-4 py-1.5 text-sm hover:bg-accent-500/60 focus:ring-0"
          onclick={dismiss}
        >
          Got it!
        </button>
      </div>
    </div>
  </div>
{/if}

