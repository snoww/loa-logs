<script lang="ts">
    import { classNameToClassId } from "$lib/constants/classes";
    import { classColors } from "$lib/constants/colors";
    import { classIconCache, colors } from "$lib/utils/settings";

    
    const resetDefaultColor = (className: string) => {
        $colors[className].color = classColors[className].defaultColor;
    };
    
</script>

<div class="mt-4 flex flex-col space-y-1 px-2">
    {#each Object.entries($colors) as classColor (classColor[0])}
        <div class="flex items-center justify-between">
            <div>
                <div class="flex items-center space-x-1">
                    <img class="size-8" src={$classIconCache[classNameToClassId[classColor[0]]]} alt={classColor[0]} />
                    <div class="text-gray-100">{classColor[0]}</div>
                </div>
            </div>
            <div class="flex items-center space-x-2">
                <input
                    class="cursor-pointer bg-zinc-800"
                    type="color"
                    id={classColor[0]}
                    bind:value={classColor[1].color}
                    on:change={(event) => {
                        if (event) $colors[classColor[0]].color = event.currentTarget.value;
                    }} />
                <button
                    class="rounded-md bg-zinc-600 p-1 text-xs hover:bg-zinc-700"
                    on:click={() => resetDefaultColor(classColor[0])}>Reset</button>
            </div>
        </div>
    {/each}
</div>
