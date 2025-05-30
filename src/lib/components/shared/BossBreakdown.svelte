<script lang="ts">
    import type { EncounterState } from "$lib/encounter.svelte";
    import { EntityState } from "$lib/entity.svelte";
    import type { Entity } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import { flip } from "svelte/animate";
    import BossBreakdownRow from "./BossBreakdownRow.svelte";

    interface Props {
        enc: EncounterState;
        boss: Entity;
        handleRightClick: () => void;
    }

    let { enc, boss, handleRightClick }: Props = $props();
    let entityState = $derived(new EntityState(boss, enc));
</script>

<table class="relative w-full table-fixed">
    <thead class="sticky top-0 z-40 h-6">
        <tr class="bg-zinc-900 tracking-tighter">
            <th class="w-14 px-2 text-left font-normal"></th>
            <th class="w-full"></th>
            <th class="w-12 font-normal" use:tooltip={{ content: "Damage Dealt" }}>DMG</th>
            <th class="w-12 font-normal" use:tooltip={{ content: "Damage per second" }}>DPS</th>
            <th class="w-10 font-normal" use:tooltip={{ content: "Damage %" }}>D%</th>
            <th class="w-10 font-normal" use:tooltip={{ content: "Total Casts" }}>Casts</th>
            <th class="w-10 font-normal" use:tooltip={{ content: "Casts per minute" }}>CPM</th>
        </tr>
    </thead>
    <tbody oncontextmenu={handleRightClick} class="relative z-10">
        {#if boss}
            {#each entityState.skills as skill, i (skill.id)}
                <tr
                    class="text-3xs h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                    animate:flip={{ duration: 200 }}>
                    <BossBreakdownRow {entityState} {skill} width={entityState.skillDamagePercentages[i]} index={i} />
                </tr>
            {/each}
        {/if}
    </tbody>
</table>
