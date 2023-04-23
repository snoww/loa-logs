<script lang="ts">
    import type { Entity, IdentityStats } from "$lib/types";
    import Arcana from "./Arcana.svelte";
    import ArtistBard from "./ArtistBard.svelte";
    import GenericClass from "./GenericClass.svelte";

    export let localPlayer: Entity;    

    let identityStats: IdentityStats = JSON.parse(localPlayer.skillStats.identityStats!);
    
    
</script>
{#if localPlayer.class === "Arcanist"}
    <Arcana {identityStats} />
{:else if localPlayer.class === "Artist" || localPlayer.class === "Bard" }
<div>
    <ArtistBard className={localPlayer.class} {identityStats} />
</div>
{:else}
<div>
    <GenericClass className={localPlayer.class} {identityStats} />
</div>
{/if}