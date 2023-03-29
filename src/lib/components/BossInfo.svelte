<script lang="ts">
    import { bosses } from "$lib/constants/bossmap";
    import { bossHpBarColors } from "$lib/constants/colors";
    import type { Entity } from "$lib/types";
    import { abbreviateNumber } from "$lib/utils/numbers";
    import { cubicInOut, cubicOut, linear } from "svelte/easing";
    import { tweened } from "svelte/motion";


    export let boss: Entity;

    let bossHPBars = 0;
    let bossCurrentBars = 0;
    let bossPreviousBars = bossCurrentBars;
    let bossCurrentPercentage = 0;
    let colorIndex = 0
    let bossBarColor = bossHpBarColors[colorIndex];
    const tweenBossHpBar = tweened(0, {
        duration: 200,
        easing: linear
    });

    $: {
        if (Object.hasOwn(bosses, boss.name)) {
            bossHPBars = bosses[boss.name];
        } else if (boss.maxHp === 1865513010) {
            // hard coding valtan ghost (hell)
            bossHPBars = 40;
        }
        bossCurrentPercentage = (boss.currentHp / boss.maxHp) * 100;
        if (bossHPBars !== 0 && !boss.isDead) {
            let bossHpPerBar = boss.maxHp / bossHPBars;
            bossCurrentBars = Math.floor((boss.currentHp / boss.maxHp) * bossHPBars);
            if (bossCurrentBars !== bossPreviousBars) {
                tweenBossHpBar.set(100);
                bossPreviousBars = bossCurrentBars;
                colorIndex = (colorIndex + 1) % bossHpBarColors.length ;
                bossBarColor = bossHpBarColors[colorIndex];
            }
            tweenBossHpBar.set(((boss.currentHp % bossHpPerBar) / bossHpPerBar) * 100);
        } else if (!boss.isDead) {
            tweenBossHpBar.set(bossCurrentPercentage);
        }

        if (boss.isDead) {
            boss.currentHp = 0;
            bossCurrentPercentage = 0;
            bossCurrentBars = 0;
            tweenBossHpBar.set(0);
        }
    }

</script>

<div class="bg-zinc-900/[.6] border border-black">
    {#if bossHPBars !== 0}
        <div class="absolute h-8 z-0 " style="background-color: {bossBarColor};width: {$tweenBossHpBar}%;"></div>
        {#if bossCurrentBars === 0}
            <div class="absolute h-8 -z-10 w-full bg-zinc-900"></div>
        {:else}
            <div class="absolute h-8 -z-10 w-full" style="background-color: {bossBarColor};"></div>
        {/if}
    {:else}
        <div class="absolute h-8 -z-10 w-full bg-zinc-900"></div>
        <div class="absolute h-8 z-0 bg-red-800" style="width: {$tweenBossHpBar}%;"></div>
    {/if}
    <div class="relative flex justify-center py-1">
        <div class="tracking-tighter">
            <div>
                {boss.name} <span>{abbreviateNumber(boss.currentHp)}/{abbreviateNumber(boss.maxHp)}<span class="pl-1">({bossCurrentPercentage.toFixed(1)}%)</span></span>
                <!-- {boss.name} <span>{boss.currentHp.toLocaleString()}/{boss.maxHp.toLocaleString()}<span class="pl-1">({bossCurrentPercentage.toFixed(1)}%)</span></span> -->
            </div>
        </div>
        {#if bossHPBars !== 0}
            {#if boss.currentHp === 0}
                <div class="absolute right-0 pr-2">Dead</div>
            {:else}
                <div class="absolute right-0 pr-2">{bossCurrentBars}x</div>
            {/if}
        {:else}
            <div class="absolute right-0 pr-2">{bossCurrentPercentage.toFixed(2)}%</div>
        {/if}
    </div>
</div>