<script lang="ts">
    import { updateSettings } from "$lib/utils/settings";

    interface Props {
        hidden: boolean;
        text: string;
    }

    let { hidden = $bindable(), text }: Props = $props();
</script>

<div class="ml-2 flex space-x-2">
    <div class="relative">
        {#if $updateSettings.available}
            <span class="absolute top-0.5 -right-0.5 flex size-2">
                <span class="bg-accent-500 absolute inline-flex h-full w-full animate-ping rounded-full opacity-75"
                ></span>
                <span class="bg-accent-800 relative inline-flex size-2 rounded-full"></span>
            </span>
        {/if}
        <button onclick={() => (hidden = false)} class="mt-px block" aria-label="Show settings">
            <svg
                class="hover:fill-accent-500 size-6 fill-gray-300"
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 96 960 960">
                <path d="M107 841v-91.5h746.5V841H107Zm0-219.5V530h746.5v91.5H107Zm0-219V310h746.5v92.5H107Z" />
            </svg>
        </button>
    </div>
    <div class="pl-2 text-xl font-medium tracking-tight text-gray-300">{text}</div>
</div>
