<script lang="ts">
    import type { StatusEffect } from "$lib/types";
    import BuffTooltip from "./BuffTooltip.svelte";

    interface Props {
        synergies: Map<number, StatusEffect>;
    }

    let { synergies }: Props = $props();

    let width = $derived.by(() => {
        if (synergies.size > 1) {
            return `${synergies.size * 1.5 + 1}rem`;
        }
        return "3.5rem";
    });
</script>

<th class="" style="width: {width}">
    <div class="flex justify-center space-x-1">
        {#each [...synergies] as [id, synergy] (id)}
            <BuffTooltip {synergy} />
        {/each}
    </div>
</th>
