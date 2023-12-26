<script lang="ts">
    import "@fontsource-variable/inter";
    import "../app.css";
    import { onDestroy, onMount } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { navigating } from "$app/stores";
    import NProgress from "nprogress";
    import "nprogress/nprogress.css";
    import { goto, invalidateAll } from "$app/navigation";
    import { settings } from "$lib/utils/settings";
    import { appWindow } from "@tauri-apps/api/window";
    import { checkUpdate } from "@tauri-apps/api/updater";
    import { updateAvailable, updateManifest } from "$lib/utils/stores";
    import { invoke } from "@tauri-apps/api";
    import UpdateAvailable from "$lib/components/shared/UpdateAvailable.svelte";

    let events: Set<UnlistenFn> = new Set();

    NProgress.configure({
        template: '<div class="bar !bg-gray-500" role="bar"><div class="peg !shadow-gray-500"></div></div>'
    });

    onMount(() => {
        let unsubscribe = navigating.subscribe((navigating) => {
            if (navigating) {
                NProgress.start();
            } else {
                NProgress.done();
            }
        });

        if (location.pathname !== "/") {
            (async () => {
                await checkForUpdate();

                let encounterUpdateEvent = await listen("show-latest-encounter", async (event) => {
                    await goto("/logs/encounter?id=" + event.payload);
                    await showWindow();
                });
                let openUrlEvent = await listen("redirect-url", async (event) => {
                    await invalidateAll();
                    await goto("/" + event.payload);
                    await showWindow();
                });

                events.add(encounterUpdateEvent);
                events.add(openUrlEvent);

                setInterval(checkForUpdate, 600 * 1000);
            })();
        }

        return () => {
            unsubscribe();
        };
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
                $updateAvailable = true;
                $updateManifest = manifest;
            }
        } catch (e) {
            await invoke("write_log", { message: e });
        }
    }
    
    $: {
        if ($settings.general.logScale === "1") {
            document.documentElement.style.setProperty("font-size", "medium");
        } else if ($settings.general.logScale === "2") {
            document.documentElement.style.setProperty("font-size", "large");
        } else if ($settings.general.logScale === "3") {
            document.documentElement.style.setProperty("font-size", "x-large");
        } else if ($settings.general.logScale === "0") {
            document.documentElement.style.setProperty("font-size", "small");
        }
    }

</script>

<div class={$settings.general.accentColor}>
    <slot />
    {#if location.pathname !== "/"}
        <UpdateAvailable />
    {/if}
</div>
