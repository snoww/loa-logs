<script lang="ts">
  import { createTooltip, melt } from "@melt-ui/svelte";
  import type { Snippet } from "svelte";
  import { fade } from "svelte/transition";

  const { tooltip, children, ...rest }: { children: Snippet; tooltip: string | Snippet | null; class?: string } =
    $props();

  const {
    elements: { trigger, content, arrow },
    states: { open }
  } = createTooltip({
    group: true,
    openDelay: 0,
    closeDelay: 0,
    closeOnPointerDown: false,
    forceVisible: true
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
    class="z-10 rounded-md border border-neutral-700 bg-neutral-800 p-0 shadow-xl"
  >
    <div use:melt={$arrow} class="rounded-tl border-l border-t border-neutral-700"></div>
    <p class="px-2 py-1 text-sm text-neutral-100">
      {#if typeof tooltip === "string"}
        {tooltip}
      {:else if tooltip}
        {@render tooltip()}
      {/if}
    </p>
  </div>
{/if}
