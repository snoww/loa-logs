<script lang="ts">
    import type { Entity } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import { flip } from "svelte/animate";
    import BossRow from "./BossRow.svelte";

    interface Props {
        bosses: Array<Entity>;
        tween?: boolean;
        duration: number;
        inspectBoss: (boss: string) => void;
    }

    let { bosses, tween = true, duration, inspectBoss }: Props = $props();

    let bossDamageDealtPercentages: Array<number> = $derived(
        bosses.map((boss) => (boss.damageStats.damageDealt / bosses[0].damageStats.damageDealt!) * 100)
    );
</script>

<table class="relative w-full table-fixed">
    <thead class="sticky top-0 z-40 h-6">
        <tr class="bg-zinc-900 tracking-tight">
            <th class="w-14 px-2 text-left font-normal"></th>
            <th class="w-full"></th>
            <th class="w-14 font-normal" use:tooltip={{ content: "Damage Dealt" }}>DMG</th>
            <th class="w-14 font-normal" use:tooltip={{ content: "Damage per second" }}>DPS</th>
        </tr>
    </thead>
    <tbody class="relative z-10">
        {#each bosses as boss, i (boss.name)}
            <tr
                class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                animate:flip={{ duration: 200 }}
                onclick={() => inspectBoss(boss.name)}>
                <BossRow {duration} {boss} width={bossDamageDealtPercentages[i]} {tween} index={i} />
            </tr>
        {/each}
    </tbody>
</table>
