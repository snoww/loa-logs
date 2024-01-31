<script lang="ts">
    import type { StatusEffect } from "$lib/types";
    import ShieldTooltip from "$lib/components/shared/ShieldTooltip.svelte";

    export let shields: Map<number, StatusEffect>;

    let width = "3.5rem";

    $: {
        if (shields.size > 1) {
            width = `${shields.size * 1.5 + 1}rem`;
        }
    }
</script>

<th class="" style="width: {width}">
    <div class="flex justify-center space-x-1">
        {#each [...shields] as [id, synergy] (id)}
            <ShieldTooltip {synergy} />
        {/each}
    </div>
</th>
