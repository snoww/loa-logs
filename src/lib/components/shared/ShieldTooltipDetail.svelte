<script lang="ts">
    import { ShieldDetails } from "$lib/types";
    import { skillIcon } from "$lib/utils/settings";
    import { generateShieldTooltipContent, tooltip } from "$lib/utils/tooltip";
    import { abbreviateNumberSplit } from "$lib/utils/numbers";

    interface Props {
        shieldDetails: ShieldDetails;
    }

    let { shieldDetails }: Props = $props();
    let shield: (string | number)[] = $derived(abbreviateNumberSplit(shieldDetails.total));
</script>

<div class="relative z-20" use:tooltip={{ content: generateShieldTooltipContent(shieldDetails, $skillIcon.path) }}>
    {shield[0]}<span class="text-3xs text-gray-300">{shield[1]}</span>
</div>
