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
    :global(.tippy-box[data-theme='buff']) {
        @apply inline-block px-2 py-1 font-medium rounded-lg shadow-sm;
        @apply text-gray-300 bg-zinc-900;
    }
    :global(.tippy-box[data-theme='menu']) {
        @apply inline-block px-2 py-1 text-sm font-medium rounded-lg shadow-sm;
        @apply text-gray-200 bg-zinc-700;
    }
    :global(.tippy-box[data-theme='buff'] > .tippy-svg-arrow) {
        @apply fill-zinc-900;
    }
    :global(.tippy-box[data-theme='menu'] > .tippy-svg-arrow) {
        @apply fill-zinc-700;
    }
</style>