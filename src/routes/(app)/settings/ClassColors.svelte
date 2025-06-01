<script lang="ts">
  import { classNameToClassId } from "$lib/constants/classes";
  import { defaultClassColors, settings } from "$lib/stores.svelte";

  import { getClassIcon } from "$lib/utils";
</script>

{#snippet colorOption(key: string)}
  <div class="flex min-w-80 items-center justify-between gap-2 rounded px-2 hover:bg-neutral-700/60">
    <div class="flex items-center gap-1">
      {#if Object.hasOwn(classNameToClassId, key)}
        <img class="size-6" src={getClassIcon(classNameToClassId[key])} alt={key} />
      {/if}
      {#if key === "Local"}
        <p class="font-semibold">Local Player (Enable in Accessibility)</p>
      {:else}
        <p>{key}</p>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      {#if settings.classColors[key] !== defaultClassColors[key]}
        <button
          class="rounded-md bg-neutral-700 px-1 text-xs hover:bg-neutral-700/80"
          onclick={() => {
            settings.classColors[key] = defaultClassColors[key];
          }}
        >
          reset
        </button>
      {/if}
      <input
        type="color"
        class="cursor-pointer"
        value={settings.classColors[key]}
        onchange={(event) => {
          if (event) settings.classColors[key] = event.currentTarget.value;
        }}
      />
    </div>
  </div>
{/snippet}

<div class="flex flex-col gap-2">
  {#each Object.keys(settings.classColors) as key, i}
    {@render colorOption(key)}
  {/each}
</div>
