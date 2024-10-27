<script lang="ts">
    import type { BuffDetails } from "$lib/types";
    import { skillIcon } from "$lib/utils/settings";
    import { generateTooltipContent, tooltip } from "$lib/utils/tooltip";

    let { synergy }: { synergy: BuffDetails } = $props();
</script>

<div class="relative z-20" use:tooltip={{ content: generateTooltipContent(synergy, $skillIcon.path) }}>
    {synergy.percentage}<span class="text-3xs text-gray-300">%</span>
</div>
