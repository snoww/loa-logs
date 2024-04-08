<script lang="ts">
    import { MeterTab } from "$lib/types";
    import { settings } from "$lib/utils/settings";
    import { getVersion } from "@tauri-apps/api/app";
    import { onMount } from "svelte";
    export let tab: MeterTab;

    function setTab(newTab: MeterTab) {
        tab = newTab;
    }

    // https://stackoverflow.com/a/70226036
    function horizontalWheel(container: HTMLElement) {
        /** Max `scrollLeft` value */
        let scrollWidth: number;

        /** Desired scroll distance per animation frame */
        let getScrollStep = () => scrollWidth / 50; /* ADJUST TO YOUR WISH */

        /** Target value for `scrollLeft` */
        let targetLeft: number;

        function scrollLeft() {
            let beforeLeft = container.scrollLeft;
            let wantDx = getScrollStep();
            let diff = targetLeft - container.scrollLeft;
            let dX = wantDx >= Math.abs(diff) ? diff : Math.sign(diff) * wantDx;

            // Performing horizontal scroll
            container.scrollBy(dX, 0);

            // Break if smaller `diff` instead of `wantDx` was used
            if (dX === diff) return;

            // Break if can't scroll anymore or target reached
            if (beforeLeft === container.scrollLeft || container.scrollLeft === targetLeft) return;

            requestAnimationFrame(scrollLeft);
        }

        container.addEventListener("wheel", (e) => {
            e.preventDefault();

            scrollWidth = container.scrollWidth - container.clientWidth;
            targetLeft = Math.min(scrollWidth, Math.max(0, container.scrollLeft + e.deltaY));

            requestAnimationFrame(scrollLeft);
        });
    }

    let div: HTMLElement;
    onMount(() => {
        horizontalWheel(div);
    });
</script>

<div class="fixed bottom-0 z-30 h-6 w-full bg-zinc-800/[.8] text-gray-300" id="footer">
    <div class="">
        <div
            class="flex items-center !overflow-x-auto"
            style="width: calc(100vw - 7rem); -webkit-mask-image: linear-gradient(to right, black 90%, transparent 100%);"
            bind:this={div}>
            <button
                class="h-6 border-0 border-b-[3px] px-1.5 {tab === MeterTab.DAMAGE
                    ? 'border-zinc-500'
                    : 'border-zinc-800'}"
                on:click={() => setTab(MeterTab.DAMAGE)}>
                Damage
            </button>
            <button
                class="h-6 border-0 border-b-[3px] px-2 {tab === MeterTab.RDPS
                    ? 'border-zinc-500'
                    : 'border-zinc-800'}"
                on:click={() => setTab(MeterTab.RDPS)}>
                RDPS
            </button>
            <button
                class="h-6 flex-shrink-0 border-0 border-b-[3px] px-1.5 {tab === MeterTab.PARTY_BUFFS
                    ? 'border-zinc-500'
                    : 'border-zinc-800'}"
                on:click={() => setTab(MeterTab.PARTY_BUFFS)}>
                Party Buffs
            </button>
            <button
                class="h-6 flex-shrink-0 border-0 border-b-[3px] px-1.5 {tab === MeterTab.SELF_BUFFS
                    ? 'border-zinc-500'
                    : 'border-zinc-800'}"
                on:click={() => setTab(MeterTab.SELF_BUFFS)}>
                Self Buffs
            </button>
            {#if $settings.general.showTanked}
                <button
                    class="h-6 border-0 border-b-[3px] px-1.5 {tab === MeterTab.TANK
                        ? 'border-zinc-500'
                        : 'border-zinc-800'}"
                    on:click={() => setTab(MeterTab.TANK)}>
                    Tanked
                </button>
            {/if}
            {#if $settings.general.showBosses}
                <button
                    class="h-6 border-0 border-b-[3px] px-1.5 {tab === MeterTab.BOSS
                        ? 'border-zinc-500'
                        : 'border-zinc-800'}"
                    on:click={() => setTab(MeterTab.BOSS)}>
                    Bosses
                </button>
            {/if}
            {#if $settings.general.showDetails}
                <button
                    class="h-6 flex-shrink-0 border-0 border-b-[3px] px-1.5 {tab === MeterTab.DETAILS
                        ? 'border-zinc-500'
                        : 'border-zinc-800'}"
                    on:click={() => setTab(MeterTab.DETAILS)}>
                    Details
                </button>
            {/if}
            <div class="px-1">&nbsp;</div>
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
