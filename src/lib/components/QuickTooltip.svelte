<script lang="ts">
  import { createTooltip, melt } from "@melt-ui/svelte";
  import type { Snippet } from "svelte";
  import { fade } from "svelte/transition";

  const {
    tooltip,
    tooltipProps,
    placement,
    delay = 0,
    children,
    ...rest
  }: {
    delay?: number;
    placement?: "top" | "bottom" | "left" | "right";
    children: Snippet;
    tooltip: string | Snippet | Snippet<[any]> | null;
    tooltipProps?: any;
    class?: string;
  } = $props();

  const {
    elements: { trigger, content, arrow },
    states: { open }
  } = createTooltip({
    group: true,
    openDelay: delay,
    closeDelay: 0,
    forceVisible: true,
    positioning: {
      placement: placement
    }
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
    class="z-50 rounded-md border border-neutral-700 bg-neutral-800 p-0 shadow-xl"
  >
    <div use:melt={$arrow} class="rounded-tl border-l border-t border-neutral-700"></div>
    <p class="px-2 py-1 text-sm text-neutral-100">
      {#if typeof tooltip === "string"}
        {tooltip}
      {:else if tooltip && tooltipProps}
        <!-- workaround for passing tooltips snippets that expect props -->
        {@render tooltip(tooltipProps)}
      {:else if tooltip}
      <!-- regular tooltip without any props -->
        {@render tooltip()}
      {/if}
    </p>
  </div>
{/if}
