<script lang="ts">
    import { bossHpMap } from "$lib/constants/bossHpBars";
    import { bossHpBarColors } from "$lib/constants/colors";
    import type { Entity } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import { menuTooltip } from "$lib/utils/tooltip";
    import { linear } from "svelte/easing";
    import { tweened } from "svelte/motion";

    export let boss: Entity;

    let bossHp = 0;
    let bossShield = 0;
    let bossHPBars = 0;
    let bossCurrentBars = 0;
    let bossPreviousBars = 0;
    let bossCurrentPercentage = 0;
    let colorIndex = 0;
    let bossBarColor = [bossHpBarColors[colorIndex], bossHpBarColors[colorIndex + 1]];
    const tweenBossHpBar = tweened(100, {
        duration: 200,
        easing: linear
    });

    let bossCurrentHp: (string | number)[];
    let bossMaxHp: (string | number)[];
    let bossShieldHp: (string | number)[];

    $: {
        bossShield = boss.currentShield;
        if (boss.currentHp < 0) {
            bossHp = 0;
        } else {
            bossHp = boss.currentHp;
        }
        bossCurrentHp = abbreviateNumberSplit(bossHp);
        bossMaxHp = abbreviateNumberSplit(boss.maxHp);
        bossShieldHp = abbreviateNumberSplit(bossShield);

        if (Object.hasOwn(bossHpMap, boss.name) && $settings.meter.bossHpBar) {
            bossHPBars = bossHpMap[boss.name];
        } else {
            bossHPBars = 0;
            bossCurrentBars = 0;
        }

        bossCurrentPercentage = (bossHp / boss.maxHp) * 100;
        if (bossHPBars !== 0 && !boss.isDead) {
            if (bossHp === boss.maxHp) {
                bossCurrentBars = bossHPBars;
                bossPreviousBars = bossCurrentBars;
            } else {
                if (bossShield > 0) {
                    bossCurrentBars = Math.round(((bossHp + bossShield) / boss.maxHp) * bossHPBars);
                } else {
                    bossCurrentBars = Math.ceil((bossHp / boss.maxHp) * bossHPBars);
                }
            }
            if (bossPreviousBars === 0) {
                bossPreviousBars = bossCurrentBars;
            }
            if (bossCurrentBars < bossPreviousBars) {
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

        if (boss.isDead || bossHp < 0) {
            colorIndex = 0;
            bossCurrentPercentage = 0;
            bossCurrentBars = 0;
            tweenBossHpBar.set(0);
        }
    }
</script>

<div class="h-7 border-y border-black bg-zinc-900/[.3]">
    {#if bossHPBars !== 0}
        {#if bossShield > 0}
            <div class="absolute -z-10 h-7 bg-neutral-400" style="width: 100%;" />
        {:else}
            <div class="absolute -z-10 h-7" style="background-color: {bossBarColor[0]};width: {$tweenBossHpBar}%;" />
        {/if}
        {#if bossCurrentBars <= 1}
            <div class="absolute -z-20 h-7 w-full bg-zinc-900" />
        {:else}
            <div class="absolute -z-20 h-7 w-full" style="background-color: {bossBarColor[1]};" />
        {/if}
        {#if $settings.meter.splitBossHpBar}
            <div class="absolute h-7 w-full">
                <div class="grid h-7 grid-cols-4 divide-x-2 divide-zinc-800/60">
                    <div />
                    <div />
                    <div />
                    <div />
                </div>
            </div>
        {/if}
    {:else}
        <div class="absolute -z-10 h-7 w-full bg-zinc-900" />
        {#if bossShield > 0}
            <div class="absolute z-0 h-7 bg-neutral-400" style="width: 100%;" />
        {:else}
            <div class="absolute z-0 h-7 bg-red-800" style="width: {$tweenBossHpBar}%;" />
        {/if}
    {/if}
    <div class="relative tracking-tighter">
        <div
            class="box-border flex h-7 items-center justify-center space-x-1 px-12 pb-px"
            use:menuTooltip={{ content: boss.name }}>
            <div class="truncate">
                {boss.name}
            </div>
            <!-- BossName 0k/0k(+0k) (0x)-->
            <div class="flex items-baseline">
                {bossCurrentHp[0]}<span class="text-xs">{bossCurrentHp[1]}</span>/{bossMaxHp[0]}<span class="text-xs"
                    >{bossMaxHp[1]}</span>
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
