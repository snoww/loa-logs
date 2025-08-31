<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { addToast } from "$lib/components/Toaster.svelte";
  import { settings, syncProgress } from "$lib/stores.svelte";
  import type { Encounter } from "$lib/types";
  import { UWUOWO_URL } from "$lib/utils";
  import { checkAccessToken, uploadLog } from "$lib/utils/sync";
  import { uploadSuccess, uploadTokenError } from "$lib/utils/toasts";
  import { createRadioGroup, melt } from "@melt-ui/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Header from "../Header.svelte";

  const {
    elements: { root, item },
    helpers: { isChecked }
  } = createRadioGroup({
    defaultValue: settings.sync.visibility
  });

  let token: string = $state(settings.sync.accessToken || "");
  let timer: number | undefined;
  const debounce = (v: string) => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      settings.sync.accessToken = v;
    }, 750);
  };

  $effect.pre(() => {
    (async () => {
      settings.sync.validToken = await checkAccessToken(settings.sync.accessToken);
    })();
  });

  function syncPastLogs(force = false) {
    if (!settings.sync.validToken) {
      addToast(uploadTokenError);
      return;
    }

    if (syncProgress.syncing) {
      return;
    }

    syncProgress.syncing = true;
    syncProgress.stop = false;
    syncProgress.uploaded = 0;

    (async () => {
      const ids = (await invoke("get_sync_candidates", { forceResync: force })) as number[];
      syncProgress.total = ids.length;

      for (let i = 0; i < ids.length; i++) {
        let id = ids[i];
        const encounter = (await invoke("load_encounter", { id: id.toString() })) as Encounter;
        let upstream = await uploadLog(id, encounter, true, true);
        if (upstream) {
          syncProgress.uploaded++;
          addToast(uploadSuccess);
        }
        syncProgress.message = "Processing logs... (" + i + "/" + ids.length + ")";
        if (syncProgress.stop) {
          break;
        }
      }
      syncProgress.syncing = false;

      if (syncProgress.uploaded > 0) {
        syncProgress.message = "Uploaded " + syncProgress.uploaded + " logs.";
      } else {
        syncProgress.message = "No new logs were uploaded.";
      }

      if (syncProgress.stop) {
        syncProgress.uploaded = 0;
        syncProgress.total = 0;
        syncProgress.stop = false;
      }
    })();
  }
</script>

{#snippet settingOption(setting: string, name: string, description: string)}
  {@const syncSettings = settings.sync as any}
  <div class="w-fit">
    <label class="flex items-center gap-2">
      <input
        type="checkbox"
        bind:checked={syncSettings[setting]}
        class="form-checkbox checked:text-accent-600/80 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0"
      />
      <div class="ml-5">
        <div class="text-sm">{name}</div>
        <div class="text-xs text-neutral-300">{description}</div>
      </div>
    </label>
  </div>
{/snippet}

<Header title="Uploading" />
<div class="mx-auto flex max-w-[180rem] flex-col gap-4 px-12 py-4">
  <div class="flex flex-col gap-2 rounded">
    <div class="flex items-center gap-4">
      <p class="text-base font-semibold">Upload Token</p>
      {#if !settings.sync.validToken}
        <a
          href={UWUOWO_URL + "/me/upload"}
          target="_blank"
          class="bg-accent-500/80 hover:bg-accent-500/70 w-fit rounded-md border border-neutral-700 p-1 text-xs"
        >
          Get Token
        </a>
      {/if}
    </div>
    <input
      type="password"
      bind:value={
        () => token,
        (v) => {
          token = v;
          debounce(v);
        }
      }
      class="focus:border-accent-500 block w-80 rounded-lg border border-neutral-600 bg-neutral-700 text-xs text-neutral-300 placeholder-neutral-400 focus:ring-0"
      placeholder="paste access token"
    />
    {#if !settings.sync.validToken && settings.sync.accessToken}
      <p class="text-red-500">Invalid token, please generate a new one</p>
    {:else if settings.sync.validToken && settings.sync.accessToken}
      <p class="text-green-500">Valid token</p>
    {/if}
  </div>
  <div class="flex flex-col gap-1">
    <p class="text-base font-semibold">Visibility Settings</p>
    <p class="text-sm text-neutral-300">
      All uploaded logs have names <span class="text-accent-500">hidden</span> by default.
    </p>
    <p class="text-sm text-neutral-300">
      Visibility settings are character specific and can be changed once you link your roster on <a
        href="{UWUOWO_URL}/me/rosters"
        target="_blank"
        class="text-accent-500 underline">uwuowo</a
      >
    </p>
  </div>
  {@render settingOption(
    "auto",
    "Auto Upload",
    "Automatically uploads logs when cleared (end game relevant raids, hell modes, with boss only damage on)"
  )}
  <div class="flex flex-col gap-2">
    <p class="text-base font-semibold">Past Logs</p>
    <div class="flex items-center gap-2">
      {#if !syncProgress.syncing}
        <button
          class="rounded-md border border-neutral-700 bg-neutral-800/80 px-2 py-1 hover:bg-neutral-700/80"
          onclick={() => syncPastLogs()}
        >
          <QuickTooltip tooltip="Upload all eligible logs in the database">Upload</QuickTooltip>
        </button>
        <button
          class="rounded-md border border-neutral-700 bg-neutral-800/80 px-2 py-1 hover:bg-neutral-700/80"
          onclick={() => syncPastLogs(true)}
        >
          <QuickTooltip tooltip="Retry uploading of all logs">Force Re-upload</QuickTooltip>
        </button>
      {:else}
        <button
          class="rounded-md border border-neutral-700 bg-neutral-800/80 px-2 py-1 hover:bg-neutral-700/80"
          onclick={() => (syncProgress.stop = true)}
        >
          Stop Sync
        </button>
      {/if}
    </div>
    {#if syncProgress.message}
      <p class="text-neutral-200">{syncProgress.message}</p>
    {/if}
  </div>
</div>
