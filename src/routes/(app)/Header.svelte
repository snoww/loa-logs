<script lang="ts">
  import { page } from "$app/state";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import { addToast } from "$lib/components/Toaster.svelte";
  import { IconArrowUp, IconDiscord, IconExternalLink, IconMenu, IconRefresh, IconX } from "$lib/icons";
  import { settings, updateInfo } from "$lib/stores.svelte";
  import { checkForUpdate } from "$lib/utils";
  import { noUpdateAvailable } from "$lib/utils/toasts";
  import { createDialog, melt } from "@melt-ui/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount, type Snippet } from "svelte";
  import { fade, fly } from "svelte/transition";

  const { title, children }: { title: string; children?: Snippet } = $props();

  let pathname = $derived(page.url.pathname);

  const {
    elements: { trigger, overlay, content, close, portalled },
    states: { open }
  } = createDialog({
    forceVisible: true
  });

  let version = $state("");
  let loaRunning = $state(false);

  onMount(() => {
    (async () => {
      version = await getVersion();
      loaRunning = await invoke("check_loa_running");
    })();

    const interval = setInterval(async () => {
      if ($open) {
        loaRunning = await invoke("check_loa_running");
      }
    }, 5000);
    return () => clearInterval(interval);
  });

  let starting = $state(false);
  let checking = $state(false);
</script>

<div
  class="sticky top-0 z-20 h-16 bg-neutral-900/70 px-8 py-5 shadow-sm shadow-neutral-800 drop-shadow-lg backdrop-blur-lg"
>
  <div class="mx-auto flex max-w-[180rem] items-center justify-between">
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
</div>

{#snippet route(name: string, path: string)}
  <a href={path} class="hover:text-accent-500 rounded-md px-3 py-1" class:bg-neutral-800={pathname.startsWith(path)}>
    {name}
  </a>
{/snippet}

{#if $open}
  <div use:melt={$portalled} class="select-none text-white {settings.app.general.accentColor}">
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

      <div class="flex gap-1 px-2">
        <a
          href="https://ko-fi.com/synow"
          target="_blank"
          class="hover:text-accent-500 flex items-center gap-2 rounded-md px-3 py-1"
        >
          <div>Donate</div>
          <IconExternalLink class="size-4" />
        </a>
      </div>
      <div class="flex items-center gap-1 px-2">
        <a
          href="https://discord.gg/RXvTMV2YHu"
          target="_blank"
          class="hover:text-accent-500 flex items-center gap-2 rounded-md px-3 py-1"
        >
          <div>ramen shop</div>
          <IconDiscord class="size-4" />
        </a>
      </div>
      <div class="flex items-center gap-1 px-2 pt-1">
        <button
          class="flex items-center gap-2 rounded-md px-3 py-1 text-sm {loaRunning || starting
            ? 'cursor-default text-neutral-300/80'
            : 'hover:text-accent-500'}"
          onclick={() => {
            starting = true;
            invoke("start_loa_process");
            setTimeout(() => {
              starting = false;
            }, 10000);
          }}
          disabled={loaRunning || starting}
        >
          {#if loaRunning}
            Lost Ark is running
          {:else if !loaRunning && starting}
            starting...
          {:else}
            Start Lost Ark
          {/if}
        </button>
      </div>
      <!-- todo version check -->
      {#if version}
        <div class="absolute bottom-0 mx-4 my-2 p-1 text-sm">
          version {version}
        </div>
      {/if}
      {#if !updateInfo.available}
        <button
          class="group absolute bottom-3.5 right-4"
          onclick={async () => {
            checking = true;
            const update = await checkForUpdate();
            setTimeout(() => {
              if (!update) {
                addToast(noUpdateAvailable);
              }
              checking = false;
            }, 900);
          }}
        >
          <QuickTooltip tooltip="Check for updates">
            <IconRefresh
              class="group-hover:text-accent-500/80 size-4 transform {checking
                ? 'animate-[spin_1s_linear_infinite_reverse]'
                : ''}"
            />
          </QuickTooltip>
        </button>
      {:else}
        <button
          class="group absolute bottom-3.5 right-4"
          onclick={async () => {
            // emit update
            updateInfo.available = false;
            updateInfo.available = true;
          }}
        >
          <QuickTooltip tooltip="Update available">
            <IconArrowUp class="group-hover:text-accent-500/70 text-accent-500/80 size-4 animate-bounce" />
          </QuickTooltip>
        </button>
      {/if}
    </div>
  </div>
{/if}
