<script lang="ts">
  import { EncounterState } from "$lib/encounter.svelte";
  import { timestampToMinutesAndSeconds } from "$lib/utils";

  let { enc }: { enc: EncounterState } = $props();

  let durationPretty = $derived.by(() => {
    if (enc.duration <= 0) {
      return timestampToMinutesAndSeconds(0, false, false, true);
    } else {
      return timestampToMinutesAndSeconds(enc.duration, false, false, true);
    }
  });
</script>

<div class="w-full text-xs tracking-tight">
  <div data-tauri-drag-region class="mx-auto flex w-80 items-center justify-between gap-2 bg-neutral-900/45 px-2 py-1">
    <div class="flex items-center gap-1 truncate">
      <div data-tauri-drag-region class="w-9">
        {durationPretty}
      </div>
      <div data-tauri-drag-region class="truncate">
        {enc.encounter?.currentBoss ? enc.encounter.currentBoss.name : "No Boss"}
      </div>
    </div>
    <div>
      <div data-tauri-drag-region class="text-nowrap">LOA Logs</div>
    </div>
  </div>
</div>
