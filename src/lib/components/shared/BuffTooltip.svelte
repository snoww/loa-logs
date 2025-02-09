<script lang="ts">
    import type { StatusEffect } from "$lib/types";
    import { skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";
    import { generateHeaderTooltip, tooltip } from "$lib/utils/tooltip";

    interface Props {
        synergy: StatusEffect;
        size?: string;
    }

    let { synergy, size = "size-5" }: Props = $props();
</script>

<div>
    <img
        src={$skillIcon.path + getSkillIcon(synergy.source.icon)}
        alt={synergy.name}
        class="table-cell {size}"
        use:tooltip={{ content: generateHeaderTooltip(synergy, $skillIcon.path) }} />
</div>
