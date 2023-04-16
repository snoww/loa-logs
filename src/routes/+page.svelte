
<script lang="ts">
    import DamageMeter from "$lib/components/DamageMeter.svelte";
    import { settings } from "$lib/utils/settings";
    import { appWindow } from '@tauri-apps/api/window';
    import { onMount } from 'svelte';

    onMount(() => {
        (async () => {
            await appWindow.setAlwaysOnTop(true);
        })();
        settings.set($settings);
	});
    
    
</script>

<div class="h-screen" id="live-meter">
    <DamageMeter />
</div>
