<script lang="ts">

    import { updateAvailable, updateDismissed, updateManifest } from "$lib/utils/stores";
    import SvelteMarkdown from 'svelte-markdown';
    import UpdateLink from "$lib/components/shared/UpdateLink.svelte";
    import { installUpdate } from "@tauri-apps/api/updater";
    import { writable } from "svelte/store";

    let updateText = writable("Update Now");
</script>
{#if $updateAvailable && $updateManifest && !$updateDismissed}
    <div class="fixed inset-0 z-50 bg-zinc-900 bg-opacity-80" />
    <div class="fixed left-0 right-0 top-0 z-50 h-modal w-full items-center justify-center p-4">
        <div class="relative top-[10%] mx-auto flex max-h-[95%] w-full xl:max-w-3xl lg:max-w-2xl md:max-w-lg max-w-md">
            <div class="relative mx-auto flex flex-col rounded-lg border-gray-700 bg-zinc-800 text-gray-400 shadow-md">
                <button
                    type="button"
                    class="absolute right-2.5 top-3 ml-auto whitespace-normal rounded-lg p-1.5 hover:bg-zinc-600 focus:outline-none"
                    aria-label="Close modal"
                    on:click={() => ($updateDismissed = true)}>
                    <span class="sr-only">Close modal</span>
                    <svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                            clip-rule="evenodd" />
                    </svg>
                </button>
                <div id="modal" class="flex-1 space-y-6 overflow-y-auto overscroll-contain px-6 py-4">
                    <div class="">
                        <div class="flex justify-center items-center space-x-2 mb-1">
                            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"
                                class="fill-gray-200 h-12 w-12">
                                <path
                                    d="M281.5-165v-57.5H679v57.5H281.5Zm170-165v-356L329-563.5 289-604l191-191 191.5 191-40.5 40.5L509-686v356h-57.5Z" />
                            </svg>
                            <div class="text-lg font-semibold text-gray-200">
                                New Update Available!
                            </div>
                        </div>
                        <div class="mb-5 text-gray-300" id="notes">
                            <SvelteMarkdown source={$updateManifest.body} renderers={{link: UpdateLink}} />
                        </div>
                        <div class="flex justify-center">
                            <button
                                type="button"
                                class="mr-2 inline-flex items-center justify-center rounded-lg bg-accent-900 px-5 py-2.5 text-center text-sm text-white hover:bg-accent-800 focus:outline-none"
                                on:click={async () => {
                                    $updateText = "Updating...";
                                    await installUpdate();
                                }}>
                                {$updateText}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    #notes > :global(h2) {
        @apply text-lg font-semibold;
    }
    #notes > :global(h3) {
        @apply text-lg font-semibold;
    }
    #notes > :global(h4) {
        @apply font-semibold;
    }
    #notes > :global(p) {
        @apply py-0.5;
    }
    #notes > :global(* > a) {
        @apply text-accent-500;
    }
    #notes > :global(* > a:hover) {
        @apply underline;
    }
    #notes > :global(ul) {
        @apply py-1;
        @apply list-disc list-inside;
    }
    #notes > :global(* > strong) {
        @apply font-semibold;
    }
    #notes > :global(* > em) {
        @apply italic;
    }
    #notes > :global(* > img) {
        @apply my-2;
    }
</style>