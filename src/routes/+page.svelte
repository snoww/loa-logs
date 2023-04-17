
<script lang="ts">
    import DamageMeter from "$lib/components/DamageMeter.svelte";
    import { defaultSettings, registerShortcuts, settings } from "$lib/utils/settings";
    import { appWindow } from '@tauri-apps/api/window';
    import { onMount } from 'svelte';
    import merge from 'lodash-es/merge';

    onMount(() => {
        settings.set(merge(defaultSettings, $settings));
        
        (async () => {
            await appWindow.setAlwaysOnTop(true);
            registerShortcuts($settings.shortcuts);       
        })();
	});
    
    
</script>

<div class="h-screen" id="live-meter">
    <DamageMeter />
</div>
