<script lang="ts">
    import DamageMeter from "$lib/components/DamageMeter.svelte";
    import { classIconCache, colors, defaultSettings, registerShortcuts, settings, skillIcon } from "$lib/utils/settings";
    import { appWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";
    import merge from "lodash-es/merge";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { classesMap } from "$lib/constants/classes";
    import { estherMap } from "$lib/constants/esthers";
    import { invoke } from "@tauri-apps/api";
    import { classColors } from "$lib/constants/colors";

    onMount(() => {
        (async () => {
            await invoke("write_log", { message: "setting up live meter" });
            let data = await invoke("get_settings");
            if (data) {
                settings.set(merge(defaultSettings, $settings, data));
            } else {
                settings.set(merge(defaultSettings, $settings));
            }
            colors.set(merge(classColors, $colors));
            if ($settings.general.blur) {
                await invoke("enable_blur");
            } else {
                await invoke("disable_blur");
            }            
            await appWindow.setAlwaysOnTop(true);
            registerShortcuts($settings.shortcuts);
            skillIcon.set({
                path: convertFileSrc(await join(await resourceDir(), "images", "skills"))
            });
            Object.keys(classesMap).forEach(async (key) => {
                $classIconCache[key] = convertFileSrc(
                    await join(await resourceDir(), "images", "classes", key + ".png")
                );
            });
            for (const esther of estherMap) {
                $classIconCache[esther.name] = convertFileSrc(
                    await join(await resourceDir(), "images", "classes", esther.icon)
                );
            }

            await invoke("write_log", { message: "finished meter setup" });
        })();
    });
</script>

<div
    class="h-screen overflow-hidden {$settings.general.transparent ? 'bg-zinc-800/[.2]' : 'bg-zinc-800 opacity-95'}"
    id="live-meter">
    <DamageMeter />
</div>
