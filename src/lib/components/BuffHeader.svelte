<script lang="ts">
    import type { StatusEffect } from "$lib/types";
    import BuffTooltip from "./shared/BuffTooltip.svelte";

    export let synergies: Map<number, StatusEffect>;

    let width = "3.5rem";

    $: {
        if (synergies.size > 1) {
            width = `${synergies.size * 1.5 + 1}rem`;
        }
    }

</script>

<th class="" style="width: {width}">
    <div class="flex justify-center space-x-1">
        {#each [...synergies] as [id, synergy] (id)}
        <BuffTooltip {synergy} />
        {/each}
    </div>
</th>