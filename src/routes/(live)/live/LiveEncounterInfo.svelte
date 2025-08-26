<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import {
    IconCamera,
    IconChevronDown,
    IconMinus,
    IconPause,
    IconPlay,
    IconPointer,
    IconRefresh,
    IconSave,
    IconSettings,
    IconUndo
  } from "$lib/icons";
  import { misc, screenshot, settings } from "$lib/stores.svelte.js";
  import { EntityType } from "$lib/types";
  import { abbreviateNumber, takeScreenshot, timestampToMinutesAndSeconds } from "$lib/utils";
  import { createDropdownMenu, melt } from "@melt-ui/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { MediaQuery } from "svelte/reactivity";
  import { fly } from "svelte/transition";
  import QuickTooltip from "$lib/components/QuickTooltip.svelte";
  import LiveShareButton from "$lib/components/LiveShareButton.svelte";

  let { enc, screenshotDiv }: { enc: EncounterState; screenshotDiv?: HTMLElement } = $props();

  const appWindow = getCurrentWebviewWindow();

  const {
    elements: { menu, item, trigger, arrow },
    states: { open }
  } = createDropdownMenu({
    preventScroll: false
  });

  let durationPretty = $derived.by(() => {
    if (enc.duration <= 0) {
      return timestampToMinutesAndSeconds(0, false, false, true);
    } else {
      return timestampToMinutesAndSeconds(enc.duration, false, false, true);
    }
  });

  const minWidth = new MediaQuery("min-width: 460px");
  const showTwo = new MediaQuery("min-width: 470px");
  const showThree = new MediaQuery("min-width: 480px");
  const showFour = new MediaQuery("min-width: 490px");
  const minHeight = new MediaQuery("min-height: 200px");
</script>

{#snippet dataInfo(name: string, tooltip: string, data: string, dataTooltip: string)}
  <div class="flex items-center gap-1 tracking-tighter text-neutral-400" data-tauri-drag-region>
    <QuickTooltip {tooltip}>
      <div data-tauri-drag-region>{name}</div>
    </QuickTooltip>
    <QuickTooltip tooltip={dataTooltip}>
      <div data-tauri-drag-region class="text-neutral-300">
        {data}
      </div>
    </QuickTooltip>
  </div>
{/snippet}

<div class="flex h-7 items-center bg-neutral-900/80 px-1 pr-2">
  <div data-tauri-drag-region class="flex w-full items-center justify-between">
    <!-- encounter info -->
    <!-- time, dmg, dps, ttk -->
    <div data-tauri-drag-region class="flex cursor-default items-center gap-1">
      {#if settings.app.general.bossOnlyDamage}
        <QuickTooltip tooltip="Boss Only Damage">
          <img src="/images/icons/boss.png" alt="Boss Only Damage" class="size-5 shrink-0" data-tauri-drag-region />
        </QuickTooltip>
      {/if}
      <div data-tauri-drag-region class="w-10">
        {durationPretty}
      </div>
      <div class="flex items-center gap-2">
        {@render dataInfo(
          "T. DMG",
          "Total Damage Dealt",
          abbreviateNumber(enc.totalDamageDealt),
          enc.totalDamageDealt.toLocaleString()
        )}
        {@render dataInfo("T. DPS", "Total DPS", abbreviateNumber(enc.dps), Math.round(enc.dps).toLocaleString())}
        {#if enc.timeToKill}
          {@render dataInfo("TTK", "Time to Kill", enc.timeToKill, enc.timeToKill)}
        {/if}
      </div>
    </div>

    {#if !screenshot.state}
      <!-- menu icons depending on how big window is -->
      <div class="flex items-center gap-0.5 text-neutral-300">
        {#if minWidth.current}
          <button class="group" onclick={() => invoke("open_most_recent_encounter")}>
            <QuickTooltip tooltip="Open Recent Log">
              <IconUndo class="group-hover:text-accent-500/80 size-5" />
            </QuickTooltip>
          </button>
        {/if}
        {#if showTwo.current}
          <button class="group" onclick={() => takeScreenshot(screenshotDiv)}>
            <QuickTooltip tooltip="Take Screenshot">
              <IconCamera class="group-hover:text-accent-500/80 size-5" />
            </QuickTooltip>
          </button>
        {/if}
        {#if showThree.current}
          <button class="group" onclick={() => emit("reset-request")}>
            <QuickTooltip tooltip="Reset Session">
              <IconRefresh class="group-hover:text-accent-500/80 size-4.5" />
            </QuickTooltip>
          </button>
        {/if}
        {#if showFour.current}
          <button
            class="group"
            onclick={() => {
              misc.paused = !misc.paused;
              emit("pause-request");
            }}
          >
            <QuickTooltip tooltip={misc.paused ? "Resume Session" : "Pause Session"}>
              {#if misc.paused}
                <IconPlay class="group-hover:text-accent-500/80 text-accent-500/80 size-5 animate-pulse" />
              {:else}
                <IconPause class="group-hover:text-accent-500/80 size-5 opacity-80" />
              {/if}
            </QuickTooltip>
          </button>
        {/if}
        {#if settings.app.general.experimentalFeatures}
          <LiveShareButton />
        {/if}

        <!-- dropdown menu trigger -->
        <button use:melt={$trigger} class="">
          <IconChevronDown
            class="hover:text-accent-500/80 size-5 transform transition-all duration-300 {$open
              ? 'text-accent-500/80 -rotate-180'
              : ''}"
          />
        </button>

        <!-- minimize window -->
        <button class="group" onclick={() => appWindow.hide()}>
          <QuickTooltip tooltip="Minimize">
            <IconMinus class="group-hover:text-accent-500/80 size-5" />
          </QuickTooltip>
        </button>
      </div>
    {/if}
  </div>
</div>

<!-- dropdown menu content -->
{#if $open}
  <div
    use:melt={$menu}
    class="noscroll flex flex-col gap-0.5 rounded-md bg-neutral-800/80 p-1 px-2 text-sm shadow-lg drop-shadow-lg backdrop-blur-xl {!minHeight.current
      ? 'max-h-20 overflow-y-scroll'
      : ''} {settings.app.general.accentColor} text-neutral-300"
    transition:fly={{ duration: 150, y: -10 }}
  >
    {#if !minWidth.current}
      <button
        use:melt={$item}
        class="group flex items-center justify-between gap-2"
        onclick={() => invoke("open_most_recent_encounter")}
      >
        <p class="group-hover:text-accent-500/80">Recent</p>
        <IconUndo class="group-hover:text-accent-500/80 size-5" />
      </button>
    {/if}
    {#if !showTwo.current}
      <button
        use:melt={$item}
        class="group flex items-center justify-between gap-2"
        onclick={() => takeScreenshot(screenshotDiv)}
      >
        <p class="group-hover:text-accent-500/80">Screenshot</p>
        <IconCamera class="group-hover:text-accent-500/80 size-5" />
      </button>
    {/if}
    {#if !showThree.current}
      <button
        use:melt={$item}
        class="group flex items-center justify-between gap-2"
        onclick={() => emit("reset-request")}
      >
        <p class="group-hover:text-accent-500/80">Reset</p>
        <IconRefresh class="group-hover:text-accent-500/80 size-5" />
      </button>
    {/if}
    {#if !showFour.current}
      <button
        use:melt={$item}
        class="group flex items-center justify-between gap-2"
        onclick={() => {
          misc.paused = !misc.paused;
          emit("pause-request");
        }}
      >
        <p class="group-hover:text-accent-500/80">{misc.paused ? "Resume" : "Pause"}</p>
        {#if misc.paused}
          <IconPlay class="group-hover:text-accent-500/80 text-accent-500/80 size-5 animate-pulse" />
        {:else}
          <IconPause class="group-hover:text-accent-500/80 size-5 opacity-80" />
        {/if}
      </button>
    {/if}
    <button
      use:melt={$item}
      class="group flex items-center justify-between gap-2"
      onclick={() => {
        if (enc.encounter) emit("save-request");
      }}
    >
      <p class="group-hover:text-accent-500/80">Save</p>
      <IconSave class="group-hover:text-accent-500/80 size-5" />
    </button>
    <QuickTooltip tooltip="click icon in system tray to disable" delay={300}>
      <button
        use:melt={$item}
        class="group flex items-center justify-between gap-2"
        onclick={() => appWindow.setIgnoreCursorEvents(true)}
      >
        <p class="group-hover:text-accent-500/80">Clickthrough</p>
        <IconPointer class="group-hover:text-accent-500/80 size-5" />
      </button>
    </QuickTooltip>
    <button
      use:melt={$item}
      class="group flex items-center justify-between gap-2"
      onclick={() => invoke("open_url", { url: "settings" })}
    >
      <p class="group-hover:text-accent-500/80">Settings</p>
      <IconSettings class="group-hover:text-accent-500/80 size-5" />
    </button>
    <div use:melt={$arrow}></div>
  </div>
{/if}
