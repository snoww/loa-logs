<script lang="ts">
    import type { BuffDetails } from "$lib/types";
    import { skillIcon } from "$lib/utils/settings";
    import { generateTooltipContent, tooltip } from "$lib/utils/tooltip";

    interface Props {
        synergy: BuffDetails;
    }

    let { synergy }: Props = $props();
</script>

<div class="relative z-20" use:tooltip={{ content: generateTooltipContent(synergy, $skillIcon.path) }}>
    {synergy.percentage}<span class="text-3xs text-gray-300">%</span>
</div>
