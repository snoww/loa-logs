<script lang="ts">
    import { MeterTab } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import { getVersion } from "@tauri-apps/api/app";
    import { onMount } from "svelte";
    export let tab: MeterTab;

    function setTab(newTab: MeterTab) {
        tab = newTab;
    }

    let div: HTMLElement;
    onMount(() => {
        div.addEventListener("wheel", (evt) => {
            evt.preventDefault();
            div.scrollLeft += evt.deltaY;
        });
    });
</script>

<div class="fixed bottom-0 z-30 h-6 w-full bg-zinc-800/[.8] text-gray-300" id="footer">
    <div class="">
        <div class="flex items-center !overflow-x-auto" style="width: calc(100vw - 7rem);" bind:this={div}>
            <button
                class="h-6 border-0 border-b-[3px] px-2 {tab === MeterTab.DAMAGE
                    ? 'border-zinc-500'
                    : 'border-zinc-800'}"
                on:click={() => setTab(MeterTab.DAMAGE)}>
                Damage
            </button>
            <!-- idk if tank stats is useful or not, so not gonna include for now -->
            <!-- <button
                class="h-6 border-0 border-b-[3px] px-2 {tab === MeterTab.TANK ? 'border-zinc-500' : 'border-zinc-800'}"
                on:click={() => setTab(MeterTab.TANK)}>
                Tank
            </button> -->
            <button
                class="h-6 flex-shrink-0 border-0 border-b-[3px] px-2 {tab === MeterTab.PARTY_BUFFS
                    ? 'border-zinc-500'
                    : 'border-zinc-800'}"
                on:click={() => setTab(MeterTab.PARTY_BUFFS)}>
                Party Syn
            </button>
            <button
                class="h-6 flex-shrink-0 border-0 border-b-[3px] px-2 {tab === MeterTab.SELF_BUFFS
                    ? 'border-zinc-500'
                    : 'border-zinc-800'}"
                on:click={() => setTab(MeterTab.SELF_BUFFS)}>
                Self Syn
            </button>
            {#if $settings.general.showDetails}
                <button
                    class="h-6 flex-shrink-0 border-0 border-b-[3px] px-2 {tab === MeterTab.DETAILS
                        ? 'border-zinc-500'
                        : 'border-zinc-800'}"
                    on:click={() => setTab(MeterTab.DETAILS)}>
                    Details
                </button>
            {/if}
        </div>
        <div class="fixed bottom-0 right-0 flex items-center">
            <div class="h-6">LOA Logs</div>
            <div class="ml-1 mr-2 text-xs text-gray-500">
                {#await getVersion()}
                    v
                {:then version}
                    v{version}
                {/await}
            </div>
        </div>
    </div>
</div>
