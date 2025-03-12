<script lang="ts">
    import type { EncounterState } from "$lib/encounter.svelte";
    import { EntityState } from "$lib/entity.svelte";
    import type { Entity } from "$lib/types";
    import { HexToRgba, RGBLinearShade } from "$lib/utils/colors";
    import { settings } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import { tooltip } from "$lib/utils/tooltip";
    import { cubicOut } from "svelte/easing";
    import { Tween } from "svelte/motion";

    interface Props {
        enc: EncounterState;
        boss: Entity;
        width: number;
        index: number;
    }

    let { enc, boss, width, index }: Props = $props();
    let entityState = $derived(new EntityState(boss, enc));

    const tweenedValue = new Tween(enc.live ? 0 : width, {
        duration: 400,
        easing: cubicOut
    });

    $effect(() => {
        tweenedValue.set(width ?? 0);
    });

    let color = "#164e63";
</script>

<td colspan="2" class="px-2">
    <div class="truncate">
        <span use:tooltip={{ content: boss.name }}>
            {boss.name}
        </span>
    </div>
</td>
<td class="px-1 text-center" use:tooltip={{ content: boss.damageStats.damageDealt.toLocaleString() }}>
    {entityState.damageDealtString[0]}<span class="text-3xs text-gray-300">{entityState.damageDealtString[1]}</span>
</td>
<td class="px-1 text-center">
    {entityState.dpsString[0]}<span class="text-3xs text-gray-300">{entityState.dpsString[1]}</span>
</td>
<td
    class="absolute left-0 -z-10 h-7 px-2 py-1"
    class:shadow-md={!takingScreenshot}
    style="background-color: {index % 2 === 1 && $settings.general.splitLines
        ? RGBLinearShade(HexToRgba(color, 0.6))
        : HexToRgba(color, 0.6)}; width: {tweenedValue.current}%"></td>
