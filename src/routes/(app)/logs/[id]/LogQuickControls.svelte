<script lang="ts">
  import { page } from "$app/state";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { addToast } from "$lib/components/Toaster.svelte";
  import { IconCamera, IconCloudUpload, IconCloudYes, IconRefresh } from "$lib/icons";
  import { settings } from "$lib/stores.svelte.js";
  import type { Encounter } from "$lib/types";
  import { LOA_BIBLE_URL, takeScreenshot } from "$lib/utils";
  import { uploadLog } from "$lib/utils/sync";
  import { uploadSuccess, uploadTokenError } from "$lib/utils/toasts";

  let { encounter = $bindable(), screenshotDiv }: { encounter: Encounter; screenshotDiv?: HTMLElement } = $props();
  let uploading = $state(false);
  const id = page.params.id || "";

  let sync = $derived(encounter.sync);

  let showReupload = $state(false);

  async function forceReupload() {
    showReupload = false;
    sync = undefined;
    await upload();
  }

  async function upload() {
    if (!settings.sync.validToken) {
      addToast(uploadTokenError);
      return;
    }
    uploading = true;
    try {
      sync = await uploadLog(id, encounter);
      if (sync) {
        addToast(uploadSuccess);
      }
    } catch (error) {
      console.error("Upload failed:", error);
    } finally {
      uploading = false;
    }
  }
</script>

<div class="flex border-l border-neutral-900/80 pl-1">
  <button
    class="rounded-lg px-2 py-1 text-sm text-nowrap transition hover:bg-neutral-800/40 focus:outline-hidden active:bg-accent-500/80"
    onclick={() => takeScreenshot(screenshotDiv)}
  >
    <QuickTooltip tooltip="Take screenshot and copy to clipboard" placement="top" class="flex items-center gap-1">
      <IconCamera class="size-5" />
      Screenshot
    </QuickTooltip>
  </button>
  {#if encounter && encounter.cleared && encounter.difficulty}
    {#if sync}
      {#if showReupload}
        <button
          class="rounded-lg px-2 py-0.5 text-sm text-nowrap transition hover:bg-neutral-800/40 focus:outline-hidden active:bg-accent-500/80"
          onclick={() => forceReupload()}
          onmouseleave={() => (showReupload = false)}
        >
          <QuickTooltip tooltip="Re-upload log" placement="top" class="flex items-center gap-1">
            <IconCloudUpload class="size-5" />
            Reupload
          </QuickTooltip>
        </button>
      {:else}
        <a
          href={LOA_BIBLE_URL + "/logs/" + sync}
          target="_blank"
          class="rounded-lg px-2 py-0.5 text-sm text-nowrap transition hover:bg-neutral-800/40 focus:outline-hidden active:bg-accent-500/80"
          onpointermove={(e) => {
            if (e.ctrlKey) showReupload = true;
          }}
        >
          <QuickTooltip tooltip="Open log in browser" placement="top" class="flex items-center gap-1">
            <IconCloudYes class="size-5.5" />
            Share
          </QuickTooltip>
        </a>
      {/if}
    {:else if uploading}
      <button
        class="flex items-center gap-1 rounded-lg px-2 py-0.5 text-sm text-nowrap transition focus:outline-hidden"
        disabled
      >
        <IconRefresh class="animate-[spin_1s_linear_infinite_reverse]" />
        Uploading
      </button>
    {:else}
      <button
        class="rounded-lg px-2 py-0.5 text-sm text-nowrap transition hover:bg-neutral-800/40 focus:outline-hidden active:bg-accent-500/80"
        onclick={() => upload()}
      >
        <QuickTooltip tooltip="Upload log" placement="top" class="flex items-center gap-1">
          <IconCloudUpload class="size-5" />
          Upload
        </QuickTooltip>
      </button>
    {/if}
  {/if}
</div>
