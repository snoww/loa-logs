<script lang="ts">
    import { EntityType, type Entity, type Skill } from "$lib/types";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";
    import { flip } from "svelte/animate";
    import { colors, settings } from "$lib/utils/settings";
    import PlayerBreakdownHeader from "./shared/PlayerBreakdownHeader.svelte";
    import { cardIds } from "$lib/constants/cards";
    import { localPlayer } from "$lib/utils/stores";
    import { EntityState } from "$lib/entity.svelte";
    import type { EncounterState } from "$lib/encounter.svelte";
    import PlayerBreakdownRow from "./shared/PlayerBreakdownRow.svelte";
    import { Tween } from "svelte/motion";
    import { cubicOut } from "svelte/easing";

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
            animate:flip={{ duration: 200 }}>
            <PlayerBreakdownRow {skill} {entityState} index={i} width={entityState.skillDamagePercentages[i]} />
        </tr>
    {/each}
</tbody>
