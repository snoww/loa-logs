
<script lang="ts">
    import DamageMeter from "$lib/components/DamageMeter.svelte";
    import TitleBar from "$lib/components/TitleBar.svelte";
    import { appWindow } from '@tauri-apps/api/window';
    import { onMount } from 'svelte';

    onMount(() => {
        (async () => {
            await appWindow.setAlwaysOnTop(true);
        })();
	});
    
</script>

<!-- <TitleBar /> -->
<div class="h-screen">
    <DamageMeter />
</div>
