<script lang="ts">
    import type { StatusEffect } from "$lib/types";
    import { skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";
    import { generateHeaderTooltip, tooltip } from "$lib/utils/tooltip";

    interface Props {
        synergy: StatusEffect;
    }

    let { synergy }: Props = $props();

    let icon = $state(synergy.source.icon);
    if (synergy.source.skill) {
        icon = synergy.source.skill.icon;
    }
</script>

<div>
    <img
        src={$skillIcon.path + getSkillIcon(icon)}
        alt={synergy.name}
        class="table-cell size-5"
        use:tooltip={{ content: generateHeaderTooltip(synergy, $skillIcon.path) }} />
</div>
