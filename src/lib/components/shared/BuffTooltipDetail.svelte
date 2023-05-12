<script lang="ts">
    import type { BuffDetails } from "$lib/types";
    import { skillIcon } from "$lib/utils/settings";
    import { generateTooltipContent, tooltip } from "$lib/utils/tooltip";

    export let synergy: BuffDetails

</script>
<div class="relative z-20" use:tooltip={{content: generateTooltipContent(synergy, $skillIcon.path)}}>
    {synergy.percentage}<span class="text-3xs text-gray-300">%</span>
</div>
<style>
    :global(.tippy-box[data-theme='test']) {
        @apply inline-block py-1 px-2.5 text-xs font-medium rounded-lg shadow-sm;
        @apply text-white bg-zinc-900;
    }
    :global(.tippy-svg-arrow) {
        @apply fill-zinc-900;
    }
</style>