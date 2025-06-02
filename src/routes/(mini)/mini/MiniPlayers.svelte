<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte";
  import { flip } from "svelte/animate";
  import MiniPlayer from "./MiniPlayer.svelte";

  let { enc }: { enc: EncounterState } = $props();

  // const dummyPlayers: Array<any> = [
  //   {
  //     entity: {
  //       classId: 205,
  //       class: "Sorceress"
  //     },
  //     name: "Snowving",
  //     width: 80
  //   },
  //   {
  //     entity: {
  //       classId: 603,
  //       class: "Aeromancer"
  //     },
  //     name: "playerwithlongname",
  //     width: 40
  //   }
  // ];
</script>

<div class="flex gap-1 h-12 w-full items-center justify-center text-xs tracking-tight px-2 py-1 max-w-[70rem] mx-auto">
    {#each enc.playersOnly as player, i (i)}
      <div class="relative flex w-32 flex-col justify-center gap-0.5" animate:flip={{ duration: 200 }}>
        <MiniPlayer entity={player} {enc} width={enc.playerDamagePercentages[i]}/>
      </div>
    {/each}
</div>
