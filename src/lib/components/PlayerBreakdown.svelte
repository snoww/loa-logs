<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte";
  import { EntityState } from "$lib/entity.svelte";
  import { type Entity } from "$lib/types";
  import { settings } from "$lib/utils/settings";
  import { flip } from "svelte/animate";
  import PlayerBreakdownHeader from "./shared/PlayerBreakdownHeader.svelte";
  import PlayerBreakdownRow from "./shared/PlayerBreakdownRow.svelte";

  interface Props {
    entity: Entity;
    enc: EncounterState;
    handleRightClick: () => void;
  }

  let { entity, enc, handleRightClick }: Props = $props();

  let entityState = $derived(new EntityState(entity, enc));
</script>

<thead class="sticky top-0 z-40 h-6">
  <tr class="bg-zinc-900 tracking-tighter">
    <PlayerBreakdownHeader {entityState} />
  </tr>
</thead>
<tbody oncontextmenu={handleRightClick} class="relative z-10">
  {#each entityState.skills as skill, i (skill.id)}
    <tr
      class="text-3xs h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
      animate:flip={{ duration: 200 }}
    >
      <PlayerBreakdownRow {skill} {entityState} index={i} width={entityState.skillDamagePercentages[i]} />
    </tr>
  {/each}
</tbody>
