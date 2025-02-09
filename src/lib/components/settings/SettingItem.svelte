<script lang="ts">
    interface Props {
        name: string;
        description: string;
        setting: boolean;
    }

    let { name, description, setting = $bindable() }: Props = $props();
</script>

<div class="">
    <label class="flex items-center">
        <input
            type="checkbox"
            bind:checked={setting}
            class="text-accent-500 size-5 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
        <div class="ml-5">
            <div class="text-gray-100">{name}</div>
            {#if description}
                <div class="text-xs text-gray-300">{description}</div>
            {/if}
        </div>
    </label>
</div>
