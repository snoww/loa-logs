<script lang="ts">
    import type { StatusEffect } from "$lib/types";
    import { skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";
    import { generateHeaderTooltip, tooltip } from "$lib/utils/tooltip";

    export let synergy: StatusEffect;
    export let size = "size-5";
</script>

<div>
    <img
        src={$skillIcon.path + getSkillIcon(synergy.source.icon)}
        alt={synergy.name}
        class="table-cell {size}"
        use:tooltip={{ content: generateHeaderTooltip(synergy, $skillIcon.path) }} />
</div>
