<script lang="ts">
    enum ShieldTab {
        GIVEN,
        RECEIVED,
        E_GIVEN,
        E_RECEIVED
    }

    let tab = ShieldTab.GIVEN;

</script>

<div class="flex divide-x divide-gray-600 items-center">
    <button
        class="rounded-sm px-2 py-1 border-t border-t-gray-600"
        class:bg-accent-900={tab === ShieldTab.GIVEN}
        class:bg-gray-700={tab !== ShieldTab.GIVEN}
        on:click={() => {tab = ShieldTab.GIVEN;}}>
        Shields Given
    </button>
    <button
        class="rounded-sm px-2 py-1 border-t border-t-gray-600"
        class:bg-accent-900={tab === ShieldTab.RECEIVED}
        class:bg-gray-700={tab !== ShieldTab.RECEIVED}
        on:click={() => {tab = ShieldTab.RECEIVED;}}>
        Shields Received
    </button>
    <button
        class="rounded-sm px-2 py-1 border-t border-t-gray-600"
        class:bg-accent-900={tab === ShieldTab.E_GIVEN}
        class:bg-gray-700={tab !== ShieldTab.E_GIVEN}
        on:click={() => {tab = ShieldTab.E_GIVEN;}}>
        Damage Absorbed
    </button>
    <button
        class="rounded-sm px-2 py-1 border-t border-t-gray-600"
        class:bg-accent-900={tab === ShieldTab.E_RECEIVED}
        class:bg-gray-700={tab !== ShieldTab.E_RECEIVED}
        on:click={() => {tab = ShieldTab.E_RECEIVED;}}>
        Damage Absorbed
    </button>
</div>
