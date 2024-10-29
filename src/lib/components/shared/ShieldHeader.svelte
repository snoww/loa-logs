<script lang="ts">
    import type { StatusEffect } from "$lib/types";
    import ShieldTooltip from "$lib/components/shared/ShieldTooltip.svelte";

    let { shields }: { shields: Map<number, StatusEffect> } = $props();

    let width = $state("3.5rem");

    $effect(() => {
        if (shields.size > 1) {
            width = `${shields.size * 1.5 + 1}rem`;
        }
    });
</script>

<th class="" style="width: {width}">
    <div class="flex justify-center space-x-1">
        {#each [...shields] as [id, synergy] (id)}
            <ShieldTooltip {synergy} />
        {/each}
    </div>
</th>
