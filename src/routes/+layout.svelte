<script lang="ts">
    import "@fontsource/inter";
    import "../app.css";
    import { onDestroy, onMount } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { navigating } from "$app/stores";
    import NProgress from 'nprogress';
    import 'nprogress/nprogress.css';
    import { goto } from "$app/navigation";
    import { settings } from '$lib/utils/settings';
    import { appWindow } from "@tauri-apps/api/window";


    let events: Set<UnlistenFn> = new Set();
    
    NProgress.configure({ template: '<div class="bar !bg-gray-500" role="bar"><div class="peg !shadow-gray-500"></div></div>'});

    onMount(() => {
        let unsubscribe = navigating.subscribe((navigating) => {
            if (navigating) {
                NProgress.start();
            } else {
                NProgress.done();
            }
        });

        if ($settings.general.accentColor === undefined) {
            $settings.general.accentColor = "theme-pink";
        }

        if (location.pathname !== "/") {
            (async () => {
                let encounterUpdateEvent = await listen('show-latest-encounter', async (event) => {                    
                    await goto("/logs/encounter?id=" + event.payload);
                    await showWindow();
                });
                let openUrlEvent = await listen('redirect-url', async (event) => {                    
                    await goto("/" + event.payload);
                    await showWindow();
                });
    
                events.add(encounterUpdateEvent);
                events.add(openUrlEvent);
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
</script>

<div class="{$settings.general.accentColor}">
    <slot />
</div>