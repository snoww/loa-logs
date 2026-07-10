<script lang="ts">
  import { deleteEncounters, getDbInfo, openDbPath, optimizeDatabase, writeLog } from "$lib/api";
  import DatePicker from "$lib/components/DatePicker.svelte";
  import { settings } from "$lib/stores.svelte";
  import type { EncounterDbInfo } from "$lib/types";
  import { createDialog, melt } from "@melt-ui/svelte";
  import { fade } from "svelte/transition";

  const {
    elements: { trigger, portalled, overlay, content, title, description, close },
    states: { open }
  } = createDialog();

  function getLocalDateString(date: Date) {
    const localDate = new Date(date.getTime() - date.getTimezoneOffset() * 60_000);
    return localDate.toISOString().slice(0, 10);
  }

  let optimized = $state(false);
  let refreshing = $state(false);
  let actionInProgress = $state(false);
  let refresh = $state(false);
  let cutoffDate = $state("");

  let dialogInfo = $state({
    title: "",
    message: "",
    action: undefined as (() => Promise<void>) | undefined
  });

  let encounterDbInfo: EncounterDbInfo = $state({
    totalEncounters: 0,
    totalEncountersFiltered: 0,
    size: ""
  });

  let shortEncounterCount = $derived(encounterDbInfo.totalEncounters - encounterDbInfo.totalEncountersFiltered);

  function openConfirmation(title: string, message: string, action: () => Promise<void>) {
    dialogInfo = { title, message, action };
  }

  async function runCleanup(action: () => Promise<void>) {
    actionInProgress = true;
    try {
      await action();
      refresh = !refresh;
      $open = false;
    } finally {
      actionInProgress = false;
    }
  }

  $effect.pre(() => {
    refresh;
    (async () => {
      refreshing = true;
      try {
        encounterDbInfo = await getDbInfo(settings.app.logs.minEncounterDuration);
      } finally {
        refreshing = false;
      }
    })();
  });
</script>

<div class="flex max-w-3xl flex-col gap-5 text-sm">
  <section class="flex flex-col gap-3">
    <div>
      <h2 class="font-medium">Database Overview</h2>
      <p class="text-xs text-neutral-300">Storage used by encounters database file.</p>
    </div>
    <div class="grid grid-cols-3 gap-2 max-sm:grid-cols-1">
      <div class="rounded-lg bg-neutral-800/60 px-3 py-2.5">
        <div class="text-xs text-neutral-400">Database size</div>
        <div class="mt-1 font-mono text-base">{refreshing && !encounterDbInfo.size ? "..." : encounterDbInfo.size}</div>
      </div>
      <div class="rounded-lg bg-neutral-800/60 px-3 py-2.5">
        <div class="text-xs text-neutral-400">Saved encounters</div>
        <div class="mt-1 font-mono text-base">{encounterDbInfo.totalEncounters.toLocaleString()}</div>
      </div>
      <div class="rounded-lg bg-neutral-800/60 px-3 py-2.5">
        <div class="text-xs text-neutral-400">Below minimum duration</div>
        <div class="mt-1 font-mono text-base">{Math.max(0, shortEncounterCount).toLocaleString()}</div>
      </div>
    </div>
  </section>

  <section class="flex flex-col gap-2 border-t border-neutral-700/70 pt-4">
    <div class="flex items-center justify-between gap-4 py-1">
      <div>
        <div>Database folder</div>
        <div class="text-xs text-neutral-400">View the database and related files in File Explorer.</div>
      </div>
      <button class="shrink-0 rounded-md bg-neutral-700 px-3 py-1.5 hover:bg-neutral-600" onclick={openDbPath}>
        Open folder
      </button>
    </div>
    <div class="flex items-center justify-between gap-4 py-1">
      <div>
        <div>Optimize database</div>
        <div class="text-xs text-neutral-400">Rebuild the search index and reclaim unused space.</div>
      </div>
      <button
        class="w-24 shrink-0 rounded-md px-3 py-1.5 {optimized
          ? 'bg-neutral-700 text-neutral-300'
          : 'bg-accent-600/80 hover:bg-accent-600/70'}"
        disabled={optimized || actionInProgress}
        use:melt={$trigger}
        onclick={() => {
          openConfirmation(
            "Optimize Database",
            "Optimize the database now? This can take some time and the app may briefly become unresponsive.",
            () =>
              runCleanup(async () => {
                await writeLog("optimizing database...");
                await optimizeDatabase();
                optimized = true;
              })
          );
        }}
      >
        {optimized ? "Optimized" : "Optimize"}
      </button>
    </div>
  </section>

  <section class="flex flex-col gap-3 border-t border-neutral-700/70 pt-4">
    <div>
      <h2 class="font-medium">Database Cleanup</h2>
      <p class="text-xs text-neutral-300">Choose which saved encounters to delete.</p>
    </div>

    <label class="flex w-fit items-center gap-2">
      <input
        type="checkbox"
        bind:checked={settings.app.general.keepFavorites}
        class="form-checkbox size-5 rounded-sm border-0 bg-neutral-700 checked:text-accent-600 focus:ring-0"
      />
      <div class="ml-3">
        <div>Keep favorites</div>
        <div class="text-xs text-neutral-300">Exclude favorited encounters from every cleanup option below.</div>
      </div>
    </label>

    <div class="rounded-lg border border-neutral-700/70 bg-neutral-800/30 p-3">
      <div class="flex items-end justify-between gap-4 max-sm:flex-col max-sm:items-start">
        <label class="flex flex-col gap-1">
          <span>Delete encounters recorded before</span>
          <span class="text-xs text-neutral-400">Select a cutoff date to remove older history.</span>
          <DatePicker
            date={cutoffDate}
            onDateChange={(date) => (cutoffDate = date)}
            label="Delete encounters recorded before"
            maxDate={getLocalDateString(new Date())}
          />
        </label>
        <button
          class="shrink-0 rounded-md bg-red-800 px-3 py-1.5 hover:bg-red-800/80 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={!cutoffDate || actionInProgress || encounterDbInfo.totalEncounters === 0}
          use:melt={$trigger}
          onclick={() => {
            const before = new Date(`${cutoffDate}T00:00:00`).getTime();
            const formattedDate = new Date(`${cutoffDate}T00:00:00`).toLocaleDateString();
            openConfirmation(
              "Delete Older Encounters",
              `Delete all encounters recorded before ${formattedDate}? This action cannot be undone.`,
              () =>
                runCleanup(() =>
                  deleteEncounters({
                    type: "before",
                    before,
                    keepFavorites: settings.app.general.keepFavorites
                  })
                )
            );
          }}
        >
          Delete older
        </button>
      </div>
    </div>

    <div class="flex flex-col divide-y divide-neutral-700/60">
      <div class="flex items-center justify-between gap-4 py-2.5">
        <div>
          <div>Below minimum duration</div>
          <div class="text-xs text-neutral-400">
            Delete {Math.max(0, shortEncounterCount).toLocaleString()} encounters shorter than
            {settings.app.logs.minEncounterDuration} seconds.
          </div>
        </div>
        <button
          class="shrink-0 rounded-md bg-red-800 px-3 py-1.5 hover:bg-red-800/80 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={shortEncounterCount <= 0 || actionInProgress}
          use:melt={$trigger}
          onclick={() => {
            openConfirmation(
              "Delete Short Encounters",
              `Delete encounters shorter than ${settings.app.logs.minEncounterDuration} seconds? This action cannot be undone.`,
              () =>
                runCleanup(() =>
                  deleteEncounters({
                    type: "duration",
                    minDuration: settings.app.logs.minEncounterDuration,
                    keepFavorites: settings.app.general.keepFavorites
                  })
                )
            );
          }}
        >
          Delete
        </button>
      </div>
      <div class="flex items-center justify-between gap-4 py-2.5">
        <div>
          <div>Uncleared encounters</div>
          <div class="text-xs text-neutral-400">Remove wipes while keeping cleared encounters.</div>
        </div>
        <button
          class="shrink-0 rounded-md bg-red-800 px-3 py-1.5 hover:bg-red-800/80 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={actionInProgress || encounterDbInfo.totalEncounters === 0}
          use:melt={$trigger}
          onclick={() => {
            openConfirmation(
              "Delete Uncleared Encounters",
              "Delete all uncleared encounters? This action cannot be undone.",
              () =>
                runCleanup(() =>
                  deleteEncounters({ type: "uncleared", keepFavorites: settings.app.general.keepFavorites })
                )
            );
          }}
        >
          Delete
        </button>
      </div>
      <div class="flex items-center justify-between gap-4 py-2.5">
        <div>
          <div>All encounters</div>
          <div class="text-xs text-neutral-400">Remove all locally saved encounter history.</div>
        </div>
        <button
          class="shrink-0 rounded-md bg-red-700 px-3 py-1.5 hover:bg-red-700/80 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={actionInProgress || encounterDbInfo.totalEncounters === 0}
          use:melt={$trigger}
          onclick={() => {
            openConfirmation(
              "Delete All Encounters",
              "Delete all saved encounters? This action cannot be undone.",
              () =>
                runCleanup(() => deleteEncounters({ type: "all", keepFavorites: settings.app.general.keepFavorites }))
            );
          }}
        >
          Delete all
        </button>
      </div>
    </div>
  </section>
</div>

{#if $open}
  <div use:melt={$portalled}>
    <div use:melt={$overlay} class="fixed inset-0 z-50 bg-black/50" transition:fade={{ duration: 150 }}></div>
    <div
      class="fixed top-1/2 left-1/2 z-50 flex max-h-[85vh] w-[90vw] max-w-[450px] -translate-x-1/2 -translate-y-1/2 flex-col gap-4 rounded-xl bg-neutral-800 p-5 text-white shadow-lg {settings
        .app.general.accentColor}"
      use:melt={$content}
    >
      <h2 use:melt={$title} class="font-semibold">{dialogInfo.title}</h2>
      <p use:melt={$description} class="text-sm text-neutral-300">{dialogInfo.message}</p>
      <div class="flex justify-end gap-3 pt-2">
        <button
          use:melt={$close}
          disabled={actionInProgress}
          class="rounded-md bg-neutral-700 px-3 py-1.5 hover:bg-neutral-600 disabled:opacity-50"
        >
          Cancel
        </button>
        <button
          class="min-w-20 rounded-md bg-red-600/80 px-3 py-1.5 hover:bg-red-600/70 disabled:opacity-50"
          disabled={actionInProgress}
          onclick={dialogInfo.action}
        >
          {actionInProgress ? "Working..." : "Confirm"}
        </button>
      </div>
    </div>
  </div>
{/if}
