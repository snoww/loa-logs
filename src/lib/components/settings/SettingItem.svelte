<script lang="ts">
    export let name: string;
    export let description: string;
    export let setting: boolean;
</script>

<div class="">
    <label class="font-medium flex items-center">
        <input type="checkbox" bind:checked={setting} class="rounded h-5 w-5 text-accent-500 bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
        <div class="ml-5">
            <div class="text-gray-100">{name}</div>
            {#if description}
            <div class="text-xs text-gray-300">{description}</div>
            {/if}
        </div>
    </label>
</div>