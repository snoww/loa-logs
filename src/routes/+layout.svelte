<script lang="ts">
    import "../app.css";
    import { onDestroy, onMount } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import NProgress from "nprogress";
    import "nprogress/nprogress.css";
    import { afterNavigate, goto, invalidateAll, onNavigate } from "$app/navigation";
    import { settings, updateSettings } from "$lib/utils/settings";
    import { appWindow } from "@tauri-apps/api/window";
    import { checkUpdate } from "@tauri-apps/api/updater";
    import { invoke } from "@tauri-apps/api";
    import UpdateAvailable from "$lib/components/shared/UpdateAvailable.svelte";
    import LogSidebar from "$lib/components/logs/LogSidebar.svelte";
    interface Props {
        children?: import("svelte").Snippet;
    }

    let { children }: Props = $props();

    let events: Set<UnlistenFn> = new Set();

    NProgress.configure({
        template: '<div class="bar bg-gray-500!" role="bar"><div class="peg shadow-gray-500!"></div></div>'
    });

    onMount(() => {
        onNavigate(() => {
            NProgress.start();
        });
        afterNavigate(() => {
            NProgress.done();
        });

        if (location.pathname !== "/") {
            (async () => {
                await checkForUpdate();

                if ($updateSettings.available) {
                    await showWindow();
                }

                let encounterUpdateEvent = await listen("show-latest-encounter", async (event) => {
                    await goto("/logs/encounter/" + event.payload);
                    await showWindow();
                });
                let openUrlEvent = await listen("redirect-url", async (event) => {
                    await invalidateAll();
                    await goto("/" + event.payload);
                    await showWindow();
                });

                events.add(encounterUpdateEvent);
                events.add(openUrlEvent);

                setInterval(checkForUpdate, 60 * 15 * 1000);
            })();
        }
    });

    onDestroy(() => {
        events.forEach((unlisten) => unlisten());
    });

    async function showWindow() {
        await appWindow.show();
        await appWindow.unminimize();
        await appWindow.setFocus();
    }

    async function checkForUpdate() {
        try {
            const { shouldUpdate, manifest } = await checkUpdate();
            if (shouldUpdate) {
                $updateSettings.available = true;
                const oldManifest = $updateSettings.manifest;
                $updateSettings.manifest = manifest;
                if (oldManifest?.version !== $updateSettings.manifest?.version) {
                    $updateSettings.dismissed = false;
                }
                $updateSettings.isNotice = !!manifest?.version.includes("2025");
            }
        } catch (e) {
            await invoke("write_log", { message: e });
        }
    }

    $effect.pre(() => {
        if ($settings.general.logScale === "1") {
            document.documentElement.style.setProperty("font-size", "medium");
        } else if ($settings.general.logScale === "2") {
            document.documentElement.style.setProperty("font-size", "large");
        } else if ($settings.general.logScale === "3") {
            document.documentElement.style.setProperty("font-size", "x-large");
        } else if ($settings.general.logScale === "0") {
            document.documentElement.style.setProperty("font-size", "small");
        }
    });
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />
<div class="{$settings.general.accentColor} text-sm">
    {@render children?.()}
    {#if location.pathname !== "/"}
        <UpdateAvailable />
    {/if}
</div>
