
<script lang="ts">
    import DamageMeter from "$lib/components/DamageMeter.svelte";
    import { classIconCache, defaultSettings, registerShortcuts, settings, skillIcon } from "$lib/utils/settings";
    import { appWindow } from '@tauri-apps/api/window';
    import { onMount } from 'svelte';
    import merge from 'lodash-es/merge';
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { classesMap } from "$lib/constants/classes";
    import { estherMap } from "$lib/constants/esthers";

    onMount(() => {
        settings.set(merge(defaultSettings, $settings));
        
        (async () => {
            await appWindow.setAlwaysOnTop(true);
            registerShortcuts($settings.shortcuts);
            skillIcon.set({ path: convertFileSrc(await join(await resourceDir(), 'images', 'skills'))})
            Object.keys(classesMap).forEach(async key => {
                $classIconCache[key] = convertFileSrc(await join(await resourceDir(), 'images', 'classes', key + ".png"));
            });
            for (const [key, value] of Object.entries(estherMap)) {
                $classIconCache[key] = convertFileSrc(await join(await resourceDir(), 'images', 'classes', value + ".png"));
            }
        })();
	});
    
    
</script>

<div class="h-screen overflow-hidden" id="live-meter">
    <DamageMeter />
</div>
