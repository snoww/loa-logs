<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte";
  import { misc, settings } from "$lib/stores.svelte";
  import { MeterState, MeterTab } from "$lib/types";
  import DamageMeterPartySplit from "../shared/DamageMeterPartySplit.svelte";
  import PlayerBreakdown from "../shared/PlayerBreakdown.svelte";
  import BossBreakdown from "../shared/BossBreakdown.svelte";
  import BossTable from "../shared/BossTable.svelte";
  import Buffs from "../shared/Buffs.svelte";
  import DamageTaken from "../shared/DamageTaken.svelte";
  import LiveBossInfo from "./LiveBossInfo.svelte";
  import LiveEncounterInfo from "./LiveEncounterInfo.svelte";
  import LiveFooter from "./LiveFooter.svelte";
  import { addToast } from "../Toaster.svelte";
  import { missingInfo } from "$lib/utils/toasts";
  import { isNameValid } from "$lib/utils/strings";

  let { enc }: { enc: EncounterState } = $props();

  let meterState = $state(MeterState.PARTY);
  let tab = $state(MeterTab.DAMAGE);
  let playerName = $state("");
  let player = $derived.by(() => {
    if (playerName && enc.encounter) {
      return enc.encounter.entities[playerName];
    }
  });
  let focusedBoss = $state("");

  function inspectPlayer(name: string) {
    meterState = MeterState.PLAYER;
    playerName = name;
    scrollToTop();
  }

  function inspectBoss(name: string) {
    if (enc.encounter?.entities[name]) {
      focusedBoss = name;
      scrollToTop();
    }
  }

  $effect(() => {
    if (misc.raidInProgress) {
      reset();
    }
  });

  $effect(() => {
    misc.reset;
    reset();
  });

  $effect(() => {
    if (misc.missingInfo) {
      addToast(missingInfo);
    }
  });

  $effect(() => {
    if (enc.encounter && enc.duration > 0 && !misc.missingInfo) {
      if (enc.encounter.localPlayer === "You" || !isNameValid(enc.encounter.localPlayer)) {
        misc.missingInfo = true;
      }
    }
  });

  function reset() {
    meterState = MeterState.PARTY;
    enc.reset();
    playerName = "";
    focusedBoss = "";
    misc.missingInfo = false;
  }

  function handleRightClick(e?: MouseEvent) {
    if (e) {
      e.preventDefault();
    }
    if (meterState === MeterState.PLAYER) {
      meterState = MeterState.PARTY;
      playerName = "";
      focusedBoss = "";
      scrollToTop();
    }
  }
  function scrollToTop() {
    window.scrollTo(0, 0);
  }
  let screenshotDiv: HTMLDivElement | undefined = $state();
</script>

<LiveEncounterInfo {enc} {screenshotDiv} />
{#if enc.encounter?.currentBoss && settings.app.meter.bossHpBar}
  <LiveBossInfo boss={enc.encounter.currentBoss} />
{/if}
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="select-none overflow-scroll"
  style="height: calc(100% - 1.5rem - 1.75rem {enc.encounter?.currentBoss && settings.app.meter.bossHpBar
    ? ' - 1.75rem'
    : ''});"
  bind:this={screenshotDiv}
  oncontextmenu={handleRightClick}
>
  {#if tab === MeterTab.DAMAGE}
    {#if meterState === MeterState.PARTY}
      <DamageMeterPartySplit {enc} {inspectPlayer} />
    {:else if meterState === MeterState.PLAYER && player !== undefined}
      <table class="relative isolate w-full table-fixed">
        <PlayerBreakdown entity={player} {enc} {handleRightClick} />
      </table>
    {/if}
  {:else if tab === MeterTab.PARTY_BUFFS || tab === MeterTab.SELF_BUFFS}
    <Buffs {tab} {enc} focusedPlayer={player} {inspectPlayer} {handleRightClick} />
  {:else if tab === MeterTab.TANK}
    <DamageTaken {enc} />
  {:else if tab === MeterTab.BOSS}
    {#if !focusedBoss}
      <BossTable {enc} {inspectBoss} />
    {:else}
      <BossBreakdown {enc} boss={enc.encounter!.entities[focusedBoss]} handleRightClick={() => (focusedBoss = "")} />
    {/if}
  {/if}
</div>
<LiveFooter bind:tab />
