<script lang="ts">
  import { page } from "$app/state";
  import { IconCamera, IconCloudUpload, IconCloudYes, IconRefresh } from "$lib/icons";
  import { settings } from "$lib/stores.svelte.js";
  import type { Encounter } from "$lib/types";
  import { takeScreenshot, UWUOWO_URL } from "$lib/utils";
  import { uploadLog } from "$lib/utils/sync";
  import { uploadSuccess, uploadTokenError } from "$lib/utils/toasts";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { addToast } from "$lib/components/Toaster.svelte";

  let { encounter = $bindable(), screenshotDiv }: { encounter: Encounter; screenshotDiv?: HTMLElement } = $props();
  let uploading = $state(false);

  let sync = $derived(encounter.sync);

  async function upload() {
    if (!settings.sync.validToken) {
      addToast(uploadTokenError);
      return;
    }
    uploading = true;
    try {
      sync = await uploadLog(page.params.id, encounter);
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
    class="focus:outline-hidden active:bg-accent-500/80 text-nowrap rounded-lg px-2 py-1 text-sm transition hover:bg-neutral-800/40"
    onclick={() => takeScreenshot(screenshotDiv)}
  >
    <QuickTooltip tooltip="Take screenshot and copy to clipboard" placement="top" class="flex items-center gap-1">
      <IconCamera class="size-5" />
      Screenshot
    </QuickTooltip>
  </button>
  {#if encounter && encounter.cleared && encounter.difficulty && encounter.bossOnlyDamage}
    {#if sync}
      <a
        href={UWUOWO_URL + "/logs/" + sync}
        target="_blank"
        class="focus:outline-hidden active:bg-accent-500/80 text-nowrap rounded-lg px-2 py-0.5 text-sm transition hover:bg-neutral-800/40"
        onclick={() => {}}
      >
        <QuickTooltip tooltip="Open log in browser" placement="top" class="flex items-center gap-1">
          <IconCloudYes class="size-5.5" />
          Share
        </QuickTooltip>
      </a>
    {:else if uploading}
      <button
        class="focus:outline-hidden flex items-center gap-1 text-nowrap rounded-lg px-2 py-0.5 text-sm transition"
        disabled
      >
        <IconRefresh class="animate-[spin_1s_linear_infinite_reverse]" />
        Uploading
      </button>
    {:else}
      <button
        class="focus:outline-hidden active:bg-accent-500/80 text-nowrap rounded-lg px-2 py-0.5 text-sm transition hover:bg-neutral-800/40"
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
