<script lang="ts">
  import { createTooltip, melt } from "@melt-ui/svelte";
  import type { Snippet } from "svelte";
  import { fade } from "svelte/transition";

  const {
    tooltip,
    children,
    widthExtendible = false,
    hoverable = true,
    placement = "top",
    ...rest
  }: {
    children: Snippet;
    tooltip: Snippet | string | null;
    hoverable?: boolean;
    widthExtendible?: boolean;
    placement?: "top" | "bottom";
    class?: string;
    contentClass?: string;
  } = $props();

  const {
    elements: { trigger, content, arrow },
    states: { open },
    options
  } = createTooltip({
    group: true,
    openDelay: 0,
    closeDelay: 0,
    closeOnPointerDown: false,
    forceVisible: false
  });

  $effect(() => {
    options.positioning.update((v) => ({ ...v, placement }));
    options.disableHoverableContent.set(!hoverable);
  });
</script>

{#if tooltip}
  <div use:melt={$trigger} class={rest.class || ""}>
    {@render children()}
  </div>
{:else}
  {@render children()}
{/if}

{#if $open}
  <div
    use:melt={$content}
    transition:fade={{ duration: 100 }}
    class="z-50 rounded-lg border border-neutral-600 bg-neutral-800 p-2 text-center shadow-sm {rest.contentClass || ''}"
    class:max-w-[400px]={!widthExtendible}
  >
    <div class="border-t border-l border-neutral-600" use:melt={$arrow}></div>
    {#if typeof tooltip === "function"}
      {@render tooltip()}
    {:else if typeof tooltip === "string"}
      <p class="px-4 py-1 text-sm">{tooltip}</p>
    {/if}
  </div>
{/if}
