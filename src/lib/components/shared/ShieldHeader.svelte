<script lang="ts">
    import { run } from "svelte/legacy";

    import type { StatusEffect } from "$lib/types";
    import ShieldTooltip from "$lib/components/shared/ShieldTooltip.svelte";

    interface Props {
        shields: Map<number, StatusEffect>;
    }

    let { shields }: Props = $props();

    let width = $state("3.5rem");

    $effect.pre(() => {
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
