<script lang="ts">
    import { classNameToClassId } from "$lib/constants/classes";
    import { classColors } from "$lib/constants/colors";
    import { classIconCache, colors, settings } from "$lib/utils/settings";

    const resetDefaultColor = (className: string) => {
        $colors[className].color = classColors[className].defaultColor;
    };
</script>

<div class="mt-4 flex flex-col space-y-1 px-2">
    {#if $settings.general.constantLocalPlayerColor}
        <div class="flex items-center justify-between">
            <div>
                <div class="flex items-center pb-2">
                    <div class="font-medium text-gray-100">Local Player Color</div>
                </div>
            </div>
            <div class="flex items-center space-x-2">
                <input
                    class="cursor-pointer bg-zinc-800"
                    type="color"
                    id={"Local"}
                    bind:value={$colors["Local"].color}
                    onchange={(event) => {
                        if (event) $colors["Local"].color = event.currentTarget.value;
                    }} />
                <button
                    class="rounded-md bg-zinc-600 p-1 text-xs hover:bg-zinc-700"
                    onclick={() => resetDefaultColor("Local")}
                    >Reset
                </button>
            </div>
        </div>
    {/if}
    {#each Object.entries($colors) as [key, value]}
        {#if key !== "Local"}
            <div class="flex items-center justify-between">
                <div>
                    <div class="flex items-center space-x-1">
                        <img class="size-8" src={$classIconCache[classNameToClassId[key] || 0]} alt={key} />
                        <div class="text-gray-100">{key}</div>
                    </div>
                </div>
                <div class="flex items-center space-x-2">
                    <input
                        class="cursor-pointer bg-zinc-800"
                        type="color"
                        id={key}
                        bind:value={(value as any).color}
                        onchange={(event) => {
                            if (event) $colors[key].color = event.currentTarget.value;
                        }} />
                    <button
                        class="rounded-md bg-zinc-600 p-1 text-xs hover:bg-zinc-700"
                        onclick={() => resetDefaultColor(key)}>
                        Reset
                    </button>
                </div>
            </div>
        {/if}
    {/each}
</div>
