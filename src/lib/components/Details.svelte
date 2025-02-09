<script lang="ts">
    import type { IdentityEvent, StaggerEvent } from "$lib/types";
    import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";

    let identity: IdentityEvent = $state({ gauge1: 0, gauge2: 0, gauge3: 0 });
    let stagger: StaggerEvent = $state({ current: 0, max: 0 });
    let staggerPercent = $state("");

    $effect(() => {
        if (stagger.max > 0) {
            staggerPercent = "(" + (((stagger.max - stagger.current) / stagger.max) * 100).toFixed(1) + "%)";
        }
    });

    let events: Array<UnlistenFn> = [];
    onMount(() => {
        (async () => {
            await emit("emit-details-request");
            let staggerEvent = await listen("stagger-update", (event: any) => {
                // console.log(+Date.now(), event.payload);
                stagger = event.payload;
            });
            let identityEvent = await listen("identity-update", (event: any) => {
                // console.log(+Date.now(), event.payload);
                identity = event.payload;
            });
            events.push(staggerEvent, identityEvent);
        })();
    });
    onDestroy(() => {
        emit("emit-details-request");
        events.forEach((unlisten) => unlisten());
    });
</script>

<div class="flex flex-col space-y-4 p-4">
    <div>
        <div class="font-medium">Raw Identity</div>
        <div class="font-mono text-lg">
            {identity.gauge1} / {identity.gauge2} / {identity.gauge3}
        </div>
    </div>
    <div>
        <div class="font-medium">Raw Stagger</div>
        <div class="font-mono text-lg">
            {stagger.max - stagger.current} / {stagger.max}
            {staggerPercent}
        </div>
    </div>
</div>
