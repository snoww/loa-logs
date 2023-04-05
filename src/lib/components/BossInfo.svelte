<script lang="ts">
    import { bosses } from "$lib/constants/bossMap";
    import { bossHpBarColors } from "$lib/constants/colors";
    import type { Entity } from "$lib/types";
    import { abbreviateNumber } from "$lib/utils/numbers";
    import { linear } from "svelte/easing";
    import { tweened } from "svelte/motion";


    export let boss: Entity;

    let bossHPBars = 0;
    let bossCurrentBars = 0;
    let bossPreviousBars = 0;
    let bossCurrentPercentage = 0;
    let colorIndex = 0
    let bossBarColor = [bossHpBarColors[colorIndex], bossHpBarColors[colorIndex + 1]];
    const tweenBossHpBar = tweened(100, {
        duration: 200,
        easing: linear
    });

    $: {
        if (Object.hasOwn(bosses, boss.name)) {           
            bossHPBars = bosses[boss.name];
        } else if (boss.maxHp === 1865513010 || boss.maxHp === 529402339 || boss.maxHp === 285632921) {
            // hard coding valtan ghost (hell, hard, normal)
            bossHPBars = 40;
        } else {
            bossHPBars = 0;
        }

        bossCurrentPercentage = (boss.currentHp / boss.maxHp) * 100;
        if (bossHPBars !== 0 && !boss.isDead) {
            if (boss.currentHp === boss.maxHp) {
                bossCurrentBars = bossHPBars;
                bossPreviousBars = bossCurrentBars;
            } else {
                bossCurrentBars = Math.ceil((boss.currentHp / boss.maxHp) * bossHPBars);
            }
            if (bossPreviousBars === 0) {
                bossPreviousBars = bossCurrentBars;
            }
            if (bossCurrentBars < bossPreviousBars) {
                bossPreviousBars = bossCurrentBars;
                colorIndex++;
                bossBarColor = [bossHpBarColors[(colorIndex) % bossHpBarColors.length], bossHpBarColors[(colorIndex + 1) % bossHpBarColors.length]];
            }
            if (boss.currentHp !== boss.maxHp) {
                let bossHpPerBar = boss.maxHp / bossHPBars;
                tweenBossHpBar.set(((boss.currentHp % bossHpPerBar) / bossHpPerBar) * 100);                
            }
        } else if (!boss.isDead) {
            tweenBossHpBar.set(bossCurrentPercentage);
        }

        if (boss.isDead || boss.currentHp < 0) {
            colorIndex = 0;
            boss.currentHp = 0;
            bossCurrentPercentage = 0;
            bossCurrentBars = 0;
            tweenBossHpBar.set(0);
        }
    }

</script>

<div class="bg-zinc-900/[.3] h-7 border border-black">
    {#if bossHPBars !== 0}
        <div class="absolute h-7 -z-10 " style="background-color: {bossBarColor[0]};width: {$tweenBossHpBar}%;"></div>
        {#if bossCurrentBars <= 1}
            <div class="absolute h-7 -z-20 w-full bg-zinc-900"></div>
        {:else}
            <div class="absolute h-7 -z-20 w-full" style="background-color: {bossBarColor[1]};"></div>
        {/if}
    {:else}
        <div class="absolute h-7 -z-10 w-full bg-zinc-900"></div>
        <div class="absolute h-7 z-0 bg-red-800" style="width: {$tweenBossHpBar}%;"></div>
    {/if}
    <div class="relative flex justify-center py-1">
        <div class="tracking-tighter">
            <div>
                {boss.name} <span>{abbreviateNumber(boss.currentHp)}/{abbreviateNumber(boss.maxHp)}<span class="pl-1">({bossCurrentPercentage.toFixed(1)}%)</span></span>
                <!-- {boss.name} <span>{boss.currentHp.toLocaleString()}/{boss.maxHp.toLocaleString()}<span class="pl-1">({bossCurrentPercentage.toFixed(1)}%)</span></span> -->
            </div>
        </div>
        {#if bossHPBars !== 0}
            {#if boss.currentHp <= 0}
                <div class="absolute right-0 pr-2">Dead</div>
            {:else if bossCurrentBars > 1}
                <div class="absolute right-0 pr-2">{bossCurrentBars}x</div>
            {/if}
        {/if}
    </div>
</div>