<script lang="ts">
    import { Alert } from "flowbite-svelte";
    import { fade } from "svelte/transition";

    export let showAlert;
    export let text = "";
    export let dismissable = true;
    export let width: string;
    export let isError = false;
    export let fixed = false;
</script>

<div transition:fade|global>
    <Alert
        color="none"
        class="{isError ? 'bg-red-800' : 'bg-accent-800'} {fixed
            ? 'fixed bottom-20'
            : 'absolute bottom-8'} inset-x-0 z-50 mx-auto h-10 py-2"
        style={`width: ${width};`}
        {dismissable}
        on:close={() => (showAlert = false)}>
        <span slot="icon">
            <svg
                aria-hidden="true"
                class="size-5"
                fill="currentColor"
                viewBox="0 0 20 20"
                xmlns="http://www.w3.org/2000/svg"
                ><path
                    fill-rule="evenodd"
                    d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                    clip-rule="evenodd" /></svg>
        </span>
        {text}
    </Alert>
</div>
