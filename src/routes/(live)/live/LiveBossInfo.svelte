<script lang="ts">
  import { bossHpBarColors, bossHpMap } from "$lib/constants/encounters";
  import { settings } from "$lib/stores.svelte.js";
  import type { Entity } from "$lib/types";
  import { broadcastLiveMessage } from "$lib/utils/live.svelte.js";
  import { onDestroy } from "svelte";
  import { linear } from "svelte/easing";
  import { Tween } from "svelte/motion";
  import { abbreviateNumberSplit, getBossHpBars } from "$lib/utils";

  let { boss }: { boss: Entity } = $props();

  let bossHp = $derived(boss.currentHp < 0 ? 0 : boss.currentHp);
  let bossShield = $derived(boss.currentShield);
  let bossTotalBars = $derived.by(() => {
    if (Object.hasOwn(bossHpMap, boss.name) && settings.app.meter.bossHpBar) {
      return getBossHpBars(boss.name, boss.maxHp);
    } else {
      return 1;
    }
  });
  let bossCurrentBars = $derived.by(() => {
    if (bossTotalBars && !boss.isDead) {
      if (bossHp === boss.maxHp) {
        return bossTotalBars;
      } else {
        if (bossShield > 0) {
          return Math.round(((bossHp + bossShield) / boss.maxHp) * bossTotalBars);
        } else {
          return Math.ceil((bossHp / boss.maxHp) * bossTotalBars);
        }
      }
    } else {
      return 0;
    }
  });

  let bossCurrentPercentage = $derived.by(() => {
    if (boss.isDead || bossHp < 0) {
      return 0;
    } else {
      return (bossHp / boss.maxHp) * 100;
    }
  });

  // [current bar color, next bar color]
  let bossBarColor = $derived([
    bossHpBarColors[bossCurrentBars % bossHpBarColors.length],
    bossHpBarColors[(bossCurrentBars - 1) % bossHpBarColors.length]
  ]);

  let bossCurrentHp = $derived.by(() => {
    if (boss.isDead || bossHp < 0) {
      return abbreviateNumberSplit(0);
    } else {
      return abbreviateNumberSplit(bossHp);
    }
  });
  let bossMaxHp = $derived(abbreviateNumberSplit(boss.maxHp));
  let bossShieldHp = $derived(abbreviateNumberSplit(bossShield));

  const tweenBossHpBar = new Tween(100, {
    duration: 200,
    easing: linear
  });

  $effect(() => {
    if (boss.isDead || bossHp <= 0) {
      tweenBossHpBar.set(0);
    } else if (bossHp !== boss.maxHp) {
      const bossHpPerBar = boss.maxHp / bossTotalBars;
      tweenBossHpBar.set(((bossHp % bossHpPerBar) / bossHpPerBar) * 100);
    } else {
      tweenBossHpBar.set(100);
    }
  });

  $effect.pre(() => {
    if (settings.app.general.experimentalFeatures) {
      broadcastLiveMessage({
        type: "bossStatus",
        data: {
          name: boss.name,
          isDead: boss.isDead,
          currentHp: boss.currentHp,
          maxHp: boss.maxHp,
          currentShield: boss.currentShield,
          totalBars: bossTotalBars,
          currentBars: bossCurrentBars
        }
      });
    }
  });

  onDestroy(() => {
    if (settings.app.general.experimentalFeatures) {
      broadcastLiveMessage({
        type: "bossStatus",
        data: null
      });
    }
  });
</script>

<div class="relative isolate h-7 select-none border-y border-black bg-neutral-900/70">
  <!-- hp bar background -->
  {#if bossHp}
    {#if bossShield}
      <div class="absolute -z-10 h-full w-full bg-neutral-400/95"></div>
    {:else}
      <!-- current bar color -->
      <div
        class="absolute -z-10 h-full"
        style="background-color: rgb(from {bossBarColor[0]} r g b / {0.80}); width: {tweenBossHpBar.current}%;"
      ></div>
      {#if bossTotalBars > 1 && bossCurrentBars > 1}
        <!-- next bar color (i.e. background bar) -->
        <div
          class="absolute -z-20 h-full w-full"
          style="background-color: rgb(from {bossBarColor[1]} r g b / {0.80});"
        ></div>
      {/if}
      {#if settings.app.meter.splitBossHpBar}
        <div class="absolute flex h-full w-full justify-evenly divide-x divide-neutral-900/50">
          <div class="grow"></div>
          <div class="grow"></div>
          <div class="grow"></div>
          <div class="grow"></div>
        </div>
      {/if}
    {/if}
  {/if}

  <!-- boss info -->
  <div class="relative h-full px-2 tracking-tighter">
    <div class="flex h-full items-center justify-center gap-1 px-10">
      <div class="truncate">
        {boss.name}
      </div>
      <!-- name 0k/0k(+0k) (0x)-->
      <div class="flex items-baseline">
        {bossCurrentHp[0]}<span class="text-xs">{bossCurrentHp[1]}</span>/{bossMaxHp[0]}<span class="text-xs"
          >{bossMaxHp[1]}</span
        >
        {#if bossShield > 0}
          <span class="ml-0.5">(+{bossShieldHp[0]}<span class="text-xs">{bossShieldHp[1]}</span>)</span>
        {/if}
        <span class="ml-1">({bossCurrentPercentage.toFixed(1)}<span class="text-xs">%</span>)</span>
      </div>
    </div>
  </div>
  {#if boss.isDead || bossHp <= 0}
    <div class="absolute inset-y-0 right-0 pr-2 tracking-tight">
      <div class="flex h-full items-center justify-center">Dead</div>
    </div>
  {:else if bossCurrentBars > 1}
    <div class="absolute inset-y-0 right-0 pr-2 tracking-tight">
      <div class="flex h-full items-center justify-center">
        {bossCurrentBars}x
      </div>
    </div>
  {/if}
</div>
