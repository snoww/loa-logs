<script lang="ts">
  import { bossHpMap } from "$lib/constants/bossHpBars";
  import { bossHpBarColors } from "$lib/constants/colors";
  import type { Entity } from "$lib/types";
  import { broadcastLiveMessage } from "$lib/utils/live";
  import { abbreviateNumberSplit, getBossHpBars } from "$lib/utils/numbers";
  import { settings } from "$lib/utils/settings";
  import { menuTooltip } from "$lib/utils/tooltip";
  import { onDestroy } from "svelte";
  import { linear } from "svelte/easing";
  import { Tween } from "svelte/motion";

  interface Props {
    boss: Entity;
  }

  let { boss }: Props = $props();

  let bossHp = $derived.by(() => {
    if (boss.currentHp < 0) {
      return 0;
    } else {
      return boss.currentHp;
    }
  });

  let bossShield = $derived(boss.currentShield);
  let bossHPBars = $derived.by(() => {
    if (Object.hasOwn(bossHpMap, boss.name) && $settings.meter.bossHpBar) {
      return getBossHpBars(boss.name, boss.maxHp);
    } else {
      return 0;
    }
  });
  let bossCurrentBars = $derived.by(() => {
    if (bossHPBars !== 0 && !boss.isDead) {
      if (bossHp === boss.maxHp) {
        return bossHPBars;
      } else {
        if (bossShield > 0) {
          return Math.round(((bossHp + bossShield) / boss.maxHp) * bossHPBars);
        } else {
          return Math.ceil((bossHp / boss.maxHp) * bossHPBars);
        }
      }
    } else {
      return 0;
    }
  });
  let bossPreviousBars = $state(0);
  let bossCurrentPercentage = $derived.by(() => {
    if (boss.isDead || bossHp < 0) {
      return 0;
    } else {
      return (bossHp / boss.maxHp) * 100;
    }
  });
  let colorIndex = $state(0);
  let bossBarColor = $state([bossHpBarColors[0], bossHpBarColors[1]]);
  const tweenBossHpBar = new Tween(100, {
    duration: 200,
    easing: linear
  });

  let bossCurrentHp = $derived.by(() => {
    if (boss.isDead || bossHp < 0) {
      return abbreviateNumberSplit(0);
    } else {
      return abbreviateNumberSplit(bossHp);
    }
  });
  let bossMaxHp = $derived(abbreviateNumberSplit(boss.maxHp));
  let bossShieldHp = $derived(abbreviateNumberSplit(bossShield));

  $effect.pre(() => {
    if ($settings.general.experimentalFeatures) {
      broadcastLiveMessage({
        type: "bossStatus",
        data: {
          name: boss.name,
          isDead: boss.isDead,
          currentHp: boss.currentHp,
          maxHp: boss.maxHp,
          currentShield: boss.currentShield,
          totalBars: bossHPBars,
          currentBars: bossCurrentBars
        }
      });
    }
  });

  onDestroy(() => {
    if ($settings.general.experimentalFeatures) {
      broadcastLiveMessage({
        type: "bossStatus",
        data: null
      });
    }
  });

  $effect.pre(() => {
    if (bossHPBars !== 0 && !boss.isDead) {
      if (bossHp === boss.maxHp) {
        bossPreviousBars = bossCurrentBars;
      }
      if (bossPreviousBars === 0) {
        bossPreviousBars = bossCurrentBars;
      }
      if (bossCurrentBars !== bossPreviousBars) {
        bossPreviousBars = bossCurrentBars;
        colorIndex++;
        bossBarColor = [
          bossHpBarColors[colorIndex % bossHpBarColors.length],
          bossHpBarColors[(colorIndex + 1) % bossHpBarColors.length]
        ];
      }
      if (bossHp !== boss.maxHp) {
        let bossHpPerBar = boss.maxHp / bossHPBars;
        tweenBossHpBar.set(((bossHp % bossHpPerBar) / bossHpPerBar) * 100);
      } else {
        tweenBossHpBar.set(100);
      }
    } else if (!boss.isDead) {
      tweenBossHpBar.set(bossCurrentPercentage);
    }
  });

  $effect.pre(() => {
    if (boss.isDead || bossHp < 0) {
      colorIndex = 0;
      tweenBossHpBar.set(0);
    }
  });
</script>

<div class="h-7 border-y border-black bg-zinc-900/[.3]">
  {#if bossHPBars !== 0}
    {#if bossShield > 0}
      <div class="absolute -z-10 h-7 bg-neutral-400" style="width: 100%;"></div>
    {:else}
      <div
        class="absolute -z-10 h-7"
        style="background-color: {bossBarColor[0]};width: {tweenBossHpBar.current}%;"
      ></div>
    {/if}
    {#if bossCurrentBars <= 1}
      <div class="absolute -z-20 h-7 w-full bg-zinc-900"></div>
    {:else}
      <div class="absolute -z-20 h-7 w-full" style="background-color: {bossBarColor[1]};"></div>
    {/if}
    {#if $settings.meter.splitBossHpBar}
      <div class="absolute h-7 w-full">
        <div class="grid h-7 grid-cols-4 divide-x-2 divide-zinc-800/60">
          <div></div>
          <div></div>
          <div></div>
          <div></div>
        </div>
      </div>
    {/if}
  {:else}
    <div class="absolute -z-10 h-7 w-full bg-zinc-900"></div>
    {#if bossShield > 0}
      <div class="absolute z-0 h-7 bg-neutral-400" style="width: 100%;"></div>
    {:else}
      <div class="absolute z-0 h-7 bg-red-800" style="width: {tweenBossHpBar.current}%;"></div>
    {/if}
  {/if}
  <div class="relative tracking-tighter">
    <div
      class="box-border flex h-7 items-center justify-center space-x-1 px-12 pb-px"
      use:menuTooltip={{ content: boss.name }}
    >
      <div class="truncate">
        {boss.name}
      </div>
      <!-- BossName 0k/0k(+0k) (0x)-->
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
    {#if bossHp <= 0}
      <div class="absolute inset-y-0 right-0 h-7 pb-px pr-2">
        <div class="flex h-7 items-center justify-center">Dead</div>
      </div>
    {:else if bossCurrentBars > 1}
      <div class="absolute inset-y-0 right-0 h-7 pb-px pr-2">
        <div class="flex h-7 items-center justify-center">
          {bossCurrentBars}x
        </div>
      </div>
    {/if}
  </div>
</div>
