<script lang="ts">
    import { searchFilter } from "$lib/utils/stores.js";
</script>

{#if $searchFilter.order === 2}
    <svg xmlns="http://www.w3.org/2000/svg" class="size-4 fill-gray-400 ml-1"
         viewBox="0 -960 960 960">
        <path d="M480-332 233-580h494L480-332Z" />
    </svg>
{:else}
    <svg xmlns="http://www.w3.org/2000/svg" class="size-4 fill-gray-400 ml-1"
         viewBox="0 -960 960 960">
        <path d="M480-580 727-332H233L480-580Z" />
    </svg>
{/if}