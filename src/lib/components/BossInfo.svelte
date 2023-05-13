<script lang="ts">
    import { bosses } from "$lib/constants/bossMap";
    import { bossHpBarColors } from "$lib/constants/colors";
    import type { Entity } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
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

    let bossCurrentHp: (string | number)[];
    let bossMaxHp: (string | number)[];

    $: {
        bossCurrentHp = abbreviateNumberSplit(boss.currentHp);
        bossMaxHp = abbreviateNumberSplit(boss.maxHp);

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

<div class="bg-zinc-900/[.3] h-7 border-y border-black">
    {#if bossHPBars !== 0}
        <div class="absolute h-7 -z-10 " style="background-color: {bossBarColor[0]};width: {$tweenBossHpBar}%;"></div>
        {#if bossCurrentBars <= 1}
            <div class="absolute h-7 -z-20 w-full bg-zinc-900"></div>
        {:else}
            <div class="absolute h-7 -z-20 w-full" style="background-color: {bossBarColor[1]};"></div>
        {/if}
        {#if $settings.meter.splitBossHpBar}
        <div class="absolute h-7 w-full">
            <div class="grid grid-cols-4 divide-x-2 divide-zinc-800/60 h-7">
                <div> </div>
                <div> </div>
                <div> </div>
                <div> </div>
            </div>
        </div>
        {/if}
    {:else}
        <div class="absolute h-7 -z-10 w-full bg-zinc-900"></div>
        <div class="absolute h-7 z-0 bg-red-800" style="width: {$tweenBossHpBar}%;"></div>
    {/if}
    <div class="relative tracking-tighter">
        <div class="flex justify-center items-center px-12 h-7 space-x-1 pb-px">
            <div class="truncate">
                {boss.name}
            </div>
            <div>
                {bossCurrentHp[0]}<span class="text-xs">{bossCurrentHp[1]}</span>/{bossMaxHp[0]}<span class="text-xs">{bossMaxHp[1]}</span><span class="pl-1">({bossCurrentPercentage.toFixed(1)}<span class="text-xs">%</span>)</span>
            </div>
        </div>
        {#if bossHPBars !== 0}
        {#if boss.currentHp <= 0}
            <div class="absolute inset-y-0 right-0 pr-2 h-7 pb-px">
                <div class="flex justify-center items-center h-7">
                    Dead
                </div>
            </div>
        {:else if bossCurrentBars > 1}
            <div class="absolute inset-y-0 right-0 pr-2 h-7 pb-px">
                <div class="flex justify-center items-center h-7">
                    {bossCurrentBars}x
                </div>
            </div>
        {/if}
        {/if}
    </div>
</div>