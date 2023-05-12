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
<style>
    :global(.tippy-box[data-theme='buff']) {
        @apply inline-block px-2 py-1 font-medium rounded-lg shadow-sm text-xs;
        @apply text-gray-200 bg-zinc-900;
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