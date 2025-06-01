<script lang="ts">
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { IconCornerLeftUp } from "$lib/icons";

  let { handleRightClick }: { handleRightClick: () => void } = $props();
</script>

<QuickTooltip tooltip="Back to Overview">
  <button class="-ml-1 flex gap-1 truncate text-neutral-200" onclick={handleRightClick} aria-label="Return to Overview">
    <IconCornerLeftUp />
    <span> Back </span>
  </button>
</QuickTooltip>
