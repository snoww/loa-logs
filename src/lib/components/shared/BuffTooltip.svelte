<script lang="ts">
    import { classesMap } from "$lib/constants/classes";
    import type { StatusEffect } from "$lib/types";
    import { removeUnknownHtmlTags } from "$lib/utils/strings";


    export let buff: StatusEffect;
    console.log(buff);

</script>

<div class="font-normal text-sm">
    {#if buff.source.skill}
    <div class="flex">
        {classesMap[buff.source.skill.classId]}:
        <img src={buff.source.skill.icon} alt={buff.source.skill.name} class="w-5 h-5 mx-1"/>
        {buff.source.skill.name}
    </div>
    {:else if buff.source.name}
    <div class="flex">
        {#if buff.buffCategory === "set"}
        <div class="pr-1">
            {"[Set] " + buff.source.setName}:
        </div>
        {:else if buff.buffCategory === "bracelet"}
        <div class="pr-1">
            {"[Bracelet]"}
        </div>
        {:else if buff.buffCategory === "battleitem"}
        <div class="pr-1">
            {"[Battle Item]"}
        </div>
        {:else if buff.buffCategory === "dropsofether"}
        <div class="pr-1">
            {"[Drops of Ether]"}
        </div>
        {/if}
        <div>
            {@html removeUnknownHtmlTags(buff.source.name)}
        </div>
    </div>
    {/if}
    <div class="flex tracking-tight">
        <img src={buff.source.icon} alt={buff.name} class="w-5 h-5 mr-1"/>
        <div class="flex truncate">
            {@html removeUnknownHtmlTags(buff.source.desc)}
        </div>
    </div>    
</div>