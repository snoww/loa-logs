<script lang="ts">
    import type { BuffDetails } from "$lib/types";
    import { skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";


    export let buffDetails: BuffDetails;

</script>

<div class="font-normal text-xs flex flex-col space-y-1">
    {#each buffDetails.buffs as buff}
    {#if buff.sourceIcon}
    <div class="flex items-center">
        <img src={$skillIcon.path + getSkillIcon(buff.sourceIcon)} alt="buff_source_icon" class="w-5 h-5 mr-1"/>
        {buff.percentage}<span class="text-3xs text-gray-300">%</span>
    </div>
    {:else}
    <div class="flex items-center">
        <img src={$skillIcon.path + getSkillIcon(buff.icon)} alt="buff_icon" class="w-5 h-5 mr-1"/>
        {buff.percentage}<span class="text-3xs text-gray-300">%</span>
    </div>
    {/if}
    {/each}
</div>

