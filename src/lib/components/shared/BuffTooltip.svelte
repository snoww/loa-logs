<script lang="ts">
    import type { StatusEffect } from "$lib/types";
    import { skillIcon } from "$lib/utils/settings";
    import { getSkillIcon } from "$lib/utils/strings";
    import { generateHeaderTooltip, tooltip } from "$lib/utils/tooltip";

    export let synergy: StatusEffect

</script>
<div>
    <img src={$skillIcon.path + getSkillIcon(synergy.source.icon)} alt={synergy.name} class="w-5 h-5 table-cell"
        use:tooltip={{content: generateHeaderTooltip(synergy, $skillIcon.path)}}/>
</div>