<script lang="ts">
    import { pageStore, searchStore } from "$lib/utils/stores";
</script>

<div class="flex items-center justify-between">
    <div>
        <div class="relative">
            <div class="absolute inset-y-0 left-0 flex cursor-default items-center pl-2">
                <svg
                    class="h-5 w-5 text-gray-400"
                    aria-hidden="true"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                    xmlns="http://www.w3.org/2000/svg"
                    ><path
                        fill-rule="evenodd"
                        d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
                        clip-rule="evenodd" /></svg>
            </div>
            <input
                type="text"
                bind:value={$searchStore}
                class="focus:border-accent-500 block w-80 rounded-lg border border-gray-600 bg-zinc-700 px-8 text-sm text-zinc-300 placeholder-gray-400 focus:ring-0"
                placeholder="Search encounters, names, or classes" />
            {#if $searchStore.length > 0}
                <button
                    class="absolute inset-y-0 right-0 flex items-center pr-2"
                    on:click={() => {
                        searchStore.set("");
                        pageStore.set(1);
                    }}>
                    <svg
                        class="h-5 w-5 fill-gray-400 hover:fill-gray-200"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 96 960 960"
                        ><path
                            d="m250.5 870-64-64.5 229-229.5-229-229.5 64-64.5L480 511.5 709.5 282l64 64.5-229 229.5 229 229.5-64 64.5L480 640.5 250.5 870Z" /></svg>
                </button>
            {/if}
        </div>
    </div>
</div>
