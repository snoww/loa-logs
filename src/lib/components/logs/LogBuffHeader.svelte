<script lang="ts">
    import type { StatusEffect } from "$lib/types";
    import { Tooltip } from 'flowbite-svelte';
    import LogBuffTooltip from "../shared/BuffTooltip.svelte";
    import { skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";

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
        <div>
            <img src={$skillIcon.path + getSkillIcon(synergy.source.icon)} alt={synergy.name} class="w-5 h-5 table-cell"/>
            <Tooltip placement="bottom" defaultClass="bg-zinc-900 p-2 text-gray-300">
                <LogBuffTooltip buff={synergy} />
            </Tooltip>
        </div>
        {/each}
    </div>
</th>