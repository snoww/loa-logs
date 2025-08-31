<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { settings } from "$lib/stores.svelte";
  import type { EncounterDbInfo } from "$lib/types";
  import { createDialog, melt } from "@melt-ui/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { fade } from "svelte/transition";

  const {
    elements: { trigger, portalled, overlay, content, title, description, close },
    states: { open }
  } = createDialog();

  let optimized = $state(false);
  let refresh = $state(false);

  let dialogInfo = $state({
    title: "",
    message: "",
    action: undefined as (() => void) | undefined
  });

  let encounterDbInfo: EncounterDbInfo = $state({
    totalEncounters: 0,
    totalEncountersFiltered: 0,
    size: ""
  } as EncounterDbInfo);

  $effect.pre(() => {
    refresh;
    (async () => {
      encounterDbInfo = await invoke("get_db_info", { minDuration: settings.app.logs.minEncounterDuration });
    })();
  });
</script>

<div class="flex items-center gap-2">
  <div>Database Folder:</div>
  <button
    class="rounded-md bg-neutral-700 p-1 hover:bg-neutral-700/80"
    onclick={async () => {
      await invoke("open_db_path");
    }}
  >
    Open</button
  >
</div>
<div class="flex items-center gap-2">
  <QuickTooltip tooltip={"Use this feature if searching is slow"}>Optimize Database:</QuickTooltip>
  <button
    class="w-20 rounded-md p-1 {optimized ? 'disabled bg-neutral-600' : 'bg-accent-600/80 hover:bg-accent-600/70'}"
    use:melt={$trigger}
    onclick={() => {
      dialogInfo = {
        title: "Optimize Database",
        message:
          "Are you sure you want to optimize the database? You should only do this your search is slow. This may take some time and the app might freeze.",
        action: async () => {
          await invoke("write_log", { message: "optimizing database..." });
          await invoke("optimize_database");
          optimized = true;
          refresh = !refresh;
          $open = false;
        }
      };
    }}
  >
    {#if optimized}
      Optimized
    {:else}
      Optimize
    {/if}
  </button>
</div>
<label class="flex items-center gap-2">
  <input
    type="checkbox"
    bind:checked={settings.app.general.keepFavorites}
    class="form-checkbox checked:text-accent-600 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0"
  />
  <div class="ml-5">
    <div>Keep Favorites</div>
    <div class="text-xs text-neutral-300">
      Encounters marked as favorites will not be deleted using the options below
    </div>
  </div>
</label>
<div class="flex items-center gap-2">
  <div>Database Size:</div>
  <div class="font-mono">
    {encounterDbInfo.size}
  </div>
</div>
<div class="flex items-center gap-2">
  <QuickTooltip tooltip={"Total encounters"}>Total Encounters Saved:</QuickTooltip>
  <div class="font-mono">
    {encounterDbInfo.totalEncounters.toLocaleString()}
  </div>
</div>
{#if encounterDbInfo.totalEncounters - encounterDbInfo.totalEncountersFiltered > 0}
  <div class="flex items-center gap-2">
    <QuickTooltip tooltip={"Total encounters > minimum duration"}>Total Encounters Filtered:</QuickTooltip>
    <div class="font-mono">
      {encounterDbInfo.totalEncountersFiltered.toLocaleString()}
    </div>
  </div>
  <div class="flex items-center gap-2">
    <div>Delete Encounters Below Minimum Duration:</div>
    <button
      class="rounded-md bg-red-800 p-1 hover:bg-red-800/80"
      use:melt={$trigger}
      onclick={() => {
        dialogInfo = {
          title: "Delete Encounters Below Minimum Duration",
          message:
            "Are you sure you want to delete all encounters below the minimum duration? This action cannot be undone.",
          action: async () => {
            await invoke("delete_encounters_below_min_duration", {
              minDuration: settings.app.logs.minEncounterDuration,
              keepFavorites: settings.app.general.keepFavorites
            });
            refresh = !refresh;
            $open = false;
          }
        };
      }}
    >
      Delete
    </button>
  </div>
{/if}
{#if encounterDbInfo.totalEncounters > 0}
  <div class="flex items-center gap-2">
    <div>Delete all uncleared encounters:</div>
    <button
      class="rounded-md bg-red-800 p-1 hover:bg-red-800/80"
      use:melt={$trigger}
      onclick={() => {
        dialogInfo = {
          title: "Delete Uncleared Encounters",
          message: "Are you sure you want to delete all uncleared encounters? This action cannot be undone.",
          action: async () => {
            await invoke("delete_all_uncleared_encounters", {
              keepFavorites: settings.app.general.keepFavorites
            });
            refresh = !refresh;
            $open = false;
          }
        };
      }}
    >
      Delete
    </button>
  </div>
{/if}
{#if encounterDbInfo.totalEncounters > 0}
  <div class="flex items-center gap-2">
    <div>Delete all encounters:</div>
    <button
      class="rounded-md bg-red-800 p-1 hover:bg-red-800/80"
      use:melt={$trigger}
      onclick={() => {
        dialogInfo = {
          title: "Delete All Encounters",
          message: "Are you sure you want to delete all encounters? This action cannot be undone.",
          action: async () => {
            await invoke("delete_all_encounters", { keepFavorites: settings.app.general.keepFavorites });
            refresh = !refresh;
            $open = false;
          }
        };
      }}
    >
      Delete
    </button>
  </div>
{/if}

{#if $open}
  <div use:melt={$portalled}>
    <div use:melt={$overlay} class="fixed inset-0 z-50 bg-black/50" transition:fade={{ duration: 150 }}></div>
    <div
      class="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[90vw] max-w-[450px] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-neutral-800 p-4 shadow-lg
      {settings.app.general.accentColor} flex flex-col items-center gap-4 text-white"
      use:melt={$content}
    >
      <h2 use:melt={$title} class="font-semibold">{dialogInfo.title}</h2>
      <p use:melt={$description} class="text-center">{dialogInfo.message}</p>
      <div class="flex items-center gap-28 pt-5">
        <button use:melt={$close} class="rounded-md bg-neutral-700 p-1 hover:bg-neutral-700/80"> Close </button>
        <button class="rounded-md bg-red-500/70 p-1 hover:bg-red-500/60" onclick={dialogInfo.action}> Confirm </button>
      </div>
    </div>
  </div>
{/if}
