<script lang="ts">
  import { melt, type Toast, type ToastsElements } from "@melt-ui/svelte";
  import type { ToastData } from "./Toaster.svelte";
  import { fly } from "svelte/transition";
  import { IconX } from "$lib/icons";
  import { page } from "$app/state";

  let {
    elements,
    toast
  }: {
    elements: ToastsElements;
    toast: Toast<ToastData>;
  } = $props();

  const { content, title, description, close } = $derived(elements);
  const { id, data } = $derived(toast);

  const live = $derived(page.route.id === "/(live)/live");
</script>

<div
  use:melt={$content(id)}
  in:fly={{ duration: 150, x: live ? "" : "100%", y: live ? "100%" : "" }}
  out:fly={{ duration: 150, x: live ? "" : "100%", y: live ? "100%" : "" }}
  class="relative rounded-lg border bg-neutral-800/80 text-white shadow-md drop-shadow-xl backdrop-blur-xl {data.color
    ? data.color
    : 'border-accent-500/20'}"
>
  <div
    class="relative flex max-w-[calc(100vw-2rem)] items-center justify-between gap-4 p-2 px-4 {live
      ? 'w-fit'
      : 'w-[24rem]'}
  "
  >
    <div class="flex flex-col">
      {#if data.title}
        <h3 use:melt={$title(id)} class="flex items-center gap-2 font-semibold">
          {data.title}
        </h3>
      {/if}
      <div class="{live ? 'pr-6' : ''} pb-0.5 text-sm/snug" use:melt={$description(id)}>
        {data.description}
      </div>
    </div>
    <button use:melt={$close(id)} class="group absolute right-2 top-1.5 grid size-6 place-items-center">
      <IconX class="size-4 group-hover:opacity-70" />
    </button>
  </div>
</div>
