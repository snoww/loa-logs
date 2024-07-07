<script lang="ts">
    import { missingInfo } from "$lib/utils/stores";
    import { tooltip } from "$lib/utils/tooltip";

</script>

{#if $missingInfo}
    <div
        class="text-left font-normal text-red-400 whitespace-nowrap"
        use:tooltip={{
            content:
                "Meter opened too late, missing local player info. Please go to Character Select or Swap Characters to reload info."
        }}>
        Warning: Missing Info
    </div>
{/if}
