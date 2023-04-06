<script lang="ts">
    import { MeterTab } from '$lib/types';
    import { getVersion } from '@tauri-apps/api/app';
    export let tab: MeterTab;

    function setTab(newTab: MeterTab) {
        tab = newTab;
    }

</script>

<div class="fixed bottom-0 h-6 bg-zinc-800/[.8] w-full text-gray-300" id="footer">
    <div class="flex justify-between">
        <div class="flex items-center">
            <button class="px-2 border-0 border-b-[3px] h-6 {tab === MeterTab.DAMAGE ? "border-zinc-500": "border-zinc-800"}"
                on:click={() => setTab(MeterTab.DAMAGE)}>
                Damage
            </button>
            <!-- idk if tank stats is useful or not, so not gonna include for now -->
            <!-- <button class="px-2 border-0 border-b-[3px] h-6 {tab === MeterTab.TANK ? "border-zinc-500": "border-zinc-800"}"
                on:click={() => setTab(MeterTab.TANK)}>
                Tank
            </button> -->
            <button class="px-2 border-0 border-b-[3px] h-6 {tab === MeterTab.PARTY_BUFFS ? "border-zinc-500": "border-zinc-800"} truncate"
                on:click={() => setTab(MeterTab.PARTY_BUFFS)}>
                Party Syn
            </button>
            <button class="px-2 border-0 border-b-[3px] h-6 {tab === MeterTab.SELF_BUFFS ? "border-zinc-500": "border-zinc-800"} truncate"
                on:click={() => setTab(MeterTab.SELF_BUFFS)}>
                Self Syn
            </button>
        </div>
        <div class="flex items-center">
            <div class="h-6">
                LOA Logs
            </div>
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