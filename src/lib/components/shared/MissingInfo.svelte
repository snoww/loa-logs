<script lang="ts">
    import { missingInfo } from "$lib/utils/stores";
    import { tooltip } from "$lib/utils/tooltip";

</script>

{#if $missingInfo}
    <div
        class="text-left font-normal text-red-400 whitespace-nowrap"
        use:tooltip={{
            content:
                "MISSING INFO FOR LOCAL PLAYER, DATA WILL BE INCORRECT. Please go to CHARACTER SELECT or SWAP CHARACTERS."
        }}>
        WARNING: INVALID DATA
    </div>
{/if}
