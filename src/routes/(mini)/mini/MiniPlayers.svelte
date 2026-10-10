<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte";
  import { flip } from "svelte/animate";

  import MiniPlayer from "./MiniPlayer.svelte";

  let { enc }: { enc: EncounterState } = $props();
  let topPlayerDamage = $derived(enc.playersOnly[0]?.damageStats.damageDealt ?? 0);
</script>

<div class="mx-auto flex h-12 w-full max-w-[70rem] items-center justify-center gap-1 px-2 py-1 text-xs tracking-tight">
  {#each enc.playersOnly as player, i (player.name)}
    <div
      class="relative flex w-32 flex-col justify-center gap-0.5"
      animate:flip={{ duration: 200 }}
      class:hidden={i >= 8}
    >
      <MiniPlayer
        entity={player}
        {enc}
        width={topPlayerDamage > 0 ? (player.damageStats.damageDealt / topPlayerDamage) * 100 : 0}
      />
    </div>
  {/each}
</div>
