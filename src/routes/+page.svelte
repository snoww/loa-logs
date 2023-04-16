
<script lang="ts">
    import DamageMeter from "$lib/components/DamageMeter.svelte";
    import { registerShortcut, settings } from "$lib/utils/settings";
    import { appWindow } from '@tauri-apps/api/window';
    import { onMount } from 'svelte';

    onMount(() => {
        settings.set($settings);
        
        (async () => {
            await appWindow.setAlwaysOnTop(true);
            registerShortcut($settings.shortcuts.hideMeter.modifier, $settings.shortcuts.hideMeter.key);
        })();
	});
    
    
</script>

<div class="h-screen" id="live-meter">
    <DamageMeter />
</div>
