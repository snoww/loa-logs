
<script lang="ts">
    import DamageMeter from "$lib/components/DamageMeter.svelte";
    import { defaultSettings, settings, settingsStore } from "$lib/utils/settings";
    import { appWindow } from '@tauri-apps/api/window';
    import { onMount } from 'svelte';

    onMount(() => {
        (async () => {
            await appWindow.setAlwaysOnTop(true);
        })();

        if ($settings == undefined) {
            $settings = settingsStore("settings", defaultSettings);
        }
    
	});
    
</script>

<div class="h-screen" id="live-meter">
    <DamageMeter />
</div>
