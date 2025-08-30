<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { IconChevronDown, IconTrash } from "$lib/icons";
  import { settings } from "$lib/stores.svelte.js";
  import { createDialog, createDropdownMenu, melt } from "@melt-ui/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { fade, fly } from "svelte/transition";

  const {
    elements: { menu, item, trigger, arrow, separator },
    states: { open }
  } = createDropdownMenu({
    preventScroll: false
  });

  const {
    elements: { trigger: dialogTrigger, portalled, overlay, content, title, description, close },
    states: { open: dialogOpen }
  } = createDialog();
</script>

{#snippet toggle()}
  <div
    class="peer-checked:bg-accent-500/80 peer-focus:outline-hidden peer h-5 w-9 rounded-full border-neutral-600 bg-neutral-800 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:border after:border-neutral-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white"
  ></div>
{/snippet}

<button
  use:melt={$trigger}
  class="focus:outline-hidden border-l-1 flex flex-row items-center text-nowrap rounded-r-lg border-neutral-900/80 px-2 py-1 text-sm text-white transition {$open &&
    'bg-accent-600/60'}"
>
  Settings
  <IconChevronDown class="ml-0.5 mt-0.5 size-4 transform transition-all duration-300 {$open ? '-rotate-180' : ''}" />
</button>

{#if $open}
  <div
    use:melt={$menu}
    class="flex flex-col rounded-md bg-neutral-700 pt-2 shadow-lg {settings.app.general.accentColor} text-white"
    transition:fly={{ duration: 150, y: -10 }}
  >
    <div
      use:melt={$item}
      on:m-click={(e) => {
        e.preventDefault();
      }}
      class="mx-2 mb-2 flex items-center justify-between gap-2"
    >
      <span class="cursor-default text-sm">Show Names</span>
      <label class="relative inline-flex cursor-pointer items-center">
        <input type="checkbox" class="peer sr-only" bind:checked={settings.app.general.showNames} />
        {@render toggle()}
      </label>
    </div>

    <div
      use:melt={$item}
      on:m-click={(e) => {
        e.preventDefault();
      }}
      class="mx-2 mb-2 flex items-center justify-between gap-2"
    >
      <span class="cursor-default text-sm">Split Party Damage</span>
      <label class="relative inline-flex cursor-pointer items-center">
        <input type="checkbox" class="peer sr-only" bind:checked={settings.app.logs.splitPartyDamage} />
        {@render toggle()}
      </label>
    </div>

    <div
      use:melt={$item}
      on:m-click={(e) => {
        e.preventDefault();
      }}
      class="mx-2 mb-2 flex items-center justify-between gap-2"
    >
      <span class="cursor-default text-sm">Show Esther</span>
      <label class="relative inline-flex cursor-pointer items-center">
        <input type="checkbox" class="peer sr-only" bind:checked={settings.app.general.showEsther} />
        {@render toggle()}
      </label>
    </div>

    <div
      use:melt={$item}
      on:m-click={(e) => {
        e.preventDefault();
      }}
      class="mx-2 mb-2 flex items-center justify-between gap-2"
    >
      <span class="cursor-default text-sm">Positional Damage %</span>
      <label class="relative inline-flex cursor-pointer items-center">
        <input type="checkbox" class="peer sr-only" bind:checked={settings.app.logs.positionalDmgPercent} />
        {@render toggle()}
      </label>
    </div>

    <div
      use:melt={$item}
      on:m-click={(e) => {
        e.preventDefault();
      }}
      class="mx-2 mb-2 flex items-center justify-between gap-2"
    >
      <span class="cursor-default text-sm">Offensive Buffs Only</span>
      <label class="relative inline-flex cursor-pointer items-center">
        <input type="checkbox" class="peer sr-only" bind:checked={settings.app.buffs.default} />
        {@render toggle()}
      </label>
    </div>

    <div use:melt={$separator} class="mt-1 h-px bg-neutral-600"></div>
    <button
      use:melt={$dialogTrigger}
      class="hover:bg-accent-600/40 flex items-center gap-2 px-2 py-1 text-left text-sm transition"
    >
      Delete Log
      <IconTrash />
    </button>
    <a use:melt={$item} href="/settings" class="hover:bg-accent-600/40 rounded-b-md px-2 py-1 transition">
      <span class="text-sm">All Settings</span>
    </a>

    <div use:melt={$arrow}></div>
  </div>
{/if}

{#if $dialogOpen}
  <div use:melt={$portalled}>
    <div use:melt={$overlay} class="fixed inset-0 z-50 bg-black/50" transition:fade={{ duration: 150 }}></div>
    <div
      class="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[90vw] max-w-[450px] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-neutral-800 p-4 shadow-lg
      {settings.app.general.accentColor} flex flex-col items-center gap-4 text-white"
      use:melt={$content}
    >
      <h2 use:melt={$title} class="font-semibold">Delete Encounter</h2>
      <p use:melt={$description} class="text-center">
        Are you sure you want to delete this encounter? This action is irreversible.
      </p>
      <div class="flex items-center gap-28 pt-5">
        <button use:melt={$close} class="rounded-md bg-neutral-700 p-1 hover:bg-neutral-700/80"> Close </button>
        <button
          use:melt={$close}
          class="rounded-md bg-red-500/70 p-1 hover:bg-red-500/60"
          on:click={async () => {
            await invoke("delete_encounter", { id: page.params.id });
            goto("/logs");
          }}
        >
          Confirm
        </button>
      </div>
    </div>
  </div>
{/if}
