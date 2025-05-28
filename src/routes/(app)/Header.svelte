<script lang="ts">
  import { page } from "$app/state";
  import { settings } from "$lib/stores.svelte";
  import { IconExternalLink, IconMenu, IconX } from "$lib/icons";
  import { createDialog, melt } from "@melt-ui/svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount, type Snippet } from "svelte";
  import { fade, fly } from "svelte/transition";

  const { title, sticky = false, children }: { title: string; sticky?: boolean; children?: Snippet } = $props();

  let pathname = $derived(page.url.pathname);

  const {
    elements: { trigger, overlay, content, close, portalled },
    states: { open }
  } = createDialog({
    forceVisible: true
  });

  let version = $state("");

  onMount(() => {
    (async () => {
      version = await getVersion();
    })();
  });
</script>

<div
  class="flex h-16 items-center justify-between bg-neutral-900/70 px-8 py-5 shadow-sm shadow-neutral-800 {sticky
    ? 'sticky top-0 z-20 drop-shadow-lg backdrop-blur-lg'
    : ''}"
>
  <div class="flex items-center gap-4">
    <button use:melt={$trigger}>
      <IconMenu class="size-7 hover:opacity-60" />
    </button>
    <div class="text-xl font-medium">{title}</div>
  </div>
  {#if children}
    {@render children()}
  {/if}
</div>

{#snippet route(name: string, path: string)}
  <a href={path} class="hover:text-accent-500 rounded-md px-3 py-1" class:bg-neutral-800={pathname.startsWith(path)}>
    {name}
  </a>
{/snippet}

{#if $open}
  <div use:melt={$portalled} class="select-none text-white {settings.appSettings.general.accentColor}">
    <div use:melt={$overlay} class="fixed inset-0 z-30 bg-neutral-950/50" transition:fade={{ duration: 100 }}></div>
    <div
      use:melt={$content}
      class="fixed left-0 top-0 z-30 flex h-screen min-w-[15rem] flex-col bg-neutral-900 shadow-md"
      transition:fly={{ x: -240, duration: 100 }}
    >
      <div class="m-4 flex items-center">
        <p class="text-xl font-semibold">LOA Logs</p>
        <button use:melt={$close} class="ml-auto px-3 hover:opacity-60">
          <IconX class="size-7" />
        </button>
      </div>
      <div class="mx-4 mb-2 h-px bg-neutral-700"></div>
      <div class="grid gap-1 px-2">
        {@render route("Past Encounters", "/logs")}
        {@render route("Uploading", "/upload")}
        {@render route("Changelog", "/changelog")}
        {@render route("Settings", "/settings")}
      </div>
      <div class="m-2 h-px bg-neutral-700"></div>
      <div class="grid gap-1 px-2">
        <a
          href="https://ko-fi.com/synow"
          target="_blank"
          class="hover:text-accent-500 flex items-center gap-2 rounded-md px-3 py-1"
        >
          <div>Donate</div>
          <IconExternalLink class="size-4" />
        </a>
      </div>
      <!-- todo donate, lostark running, version check -->
      {#if version}
        <div class="absolute bottom-0 mx-4 my-2 p-1 text-sm">
          version {version}
        </div>
      {/if}
    </div>
  </div>
{/if}
