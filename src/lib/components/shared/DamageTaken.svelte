<script lang="ts">
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import { flip } from "svelte/animate";

    import DamageTakenRow from "./DamageTakenRow.svelte";
    import type { EncounterState } from "$lib/encounter.svelte";

    interface Props {
        enc: EncounterState;
    }

    let { enc }: Props = $props();
</script>

<table class="relative w-full table-fixed">
    <thead class="sticky top-0 z-40 h-6">
        <tr class="bg-zinc-900 tracking-tight">
            <th class="w-7 px-2 font-normal"></th>
            <th class="w-14 px-2 text-left font-normal"></th>
            <th class="w-full"></th>
            <th class="w-28 font-normal" use:tooltip={{ content: "Total Damage Taken" }}>Damage Taken</th>
        </tr>
    </thead>
    <tbody class="relative z-10">
        {#each enc.playerDamageTakenSorted as player, i (player.name)}
            <tr
                class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                animate:flip={{ duration: 200 }}>
                <DamageTakenRow {enc} {player} width={enc.playerDamageTakenPercentages[i]} />
            </tr>
        {/each}
    </tbody>
</table>
