<script lang="ts">
    import type { Entity, IdentityStats } from "$lib/types";
    import Arcana from "./Arcana.svelte";
    import ArtistBard from "./ArtistBard.svelte";
    import GenericClass from "./GenericClass.svelte";

    export let localPlayer: Entity;
    export let duration: number;

    let identityStats: IdentityStats = JSON.parse(localPlayer.skillStats.identityStats!);
</script>

{#if localPlayer.class === "Arcanist"}
    <Arcana {identityStats} {duration} player={localPlayer} />
{:else if localPlayer.class === "Artist" || localPlayer.class === "Bard"}
    <ArtistBard className={localPlayer.class} {identityStats} />
{:else}
    <GenericClass className={localPlayer.class} {identityStats} />
{/if}
