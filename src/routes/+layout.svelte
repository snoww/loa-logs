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

        if (location.pathname !== "/") {
            (async () => {
                let encounterUpdateEvent = await listen('show-latest-encounter', (event) => {                    
                    goto("/logs/encounter?id=" + event.payload);
                });
    
                events.add(encounterUpdateEvent);
            })();
        }

        return () => {
            unsubscribe();
        };
    });


    onDestroy(() => {
        events.forEach((unlisten) => unlisten());
    });
</script>

<div class="{$settings.general.accentColor}">
    <slot />
</div>