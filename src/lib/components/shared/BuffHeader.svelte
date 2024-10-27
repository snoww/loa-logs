<script lang="ts">
    import type { StatusEffect } from "$lib/types";
    import BuffTooltip from "./BuffTooltip.svelte";

    let { synergies }: { synergies: Map<number, StatusEffect> } = $props();

    let width = $state("3.5rem");

    $effect(() => {
        if (synergies.size > 1) {
            width = `${synergies.size * 1.5 + 1}rem`;
        }
    });
</script>

<th class="" style="width: {width}">
    <div class="flex justify-center space-x-1">
        {#each [...synergies] as [id, synergy] (id)}
            <BuffTooltip {synergy} />
        {/each}
    </div>
</th>
