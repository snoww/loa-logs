<script lang="ts">
    import { tooltip } from "$lib/utils/tooltip";
    import { rdpsEventDetails } from "$lib/utils/stores";

    export let meterSettings: any;
</script>

<thead>
    <tr class="bg-zinc-900">
        {#if $rdpsEventDetails}
            <th class="w-full px-2 text-right font-normal text-red-400">
                {#if $rdpsEventDetails === "invalid_zone"}
                    <span>RDPS Unsupported in Current Content</span>
                {:else if $rdpsEventDetails === "missing_info"}
                    <span>Meter Opened Too Late, RDPS Data not Loaded</span>
                {:else if $rdpsEventDetails === "request_failed"}
                    <span>Failed to Fetch Character Stats</span>
                {:else if $rdpsEventDetails === "request_failed_retrying"}
                    <span>Failed to Fetch Character Stats, Retrying...</span>
                {:else if $rdpsEventDetails === "not_available"}
                    <span class="text-gray-200">RDPS Unavailable</span>
                {:else if $rdpsEventDetails === "requesting_stats"}
                    <span class="text-gray-200">Requesting RDPS Data...</span>
                {:else}
                    <span>Error: {$rdpsEventDetails}</span>
                {/if}
            </th>
        {:else}
            <th class="w-7 px-2 font-normal" />
            <th class="w-14 px-2 text-left font-normal" />
            <th class="w-full" />
            <th
                class="w-14 font-normal"
                use:tooltip={{ content: "Damage dealt without any synergies + Damage given from your synergies" }}
                >rDMG
            </th>
            <th class="w-14 font-normal" use:tooltip={{ content: "rDamage per second" }}>rDPS</th>
            <th class="w-14 font-normal" use:tooltip={{ content: "rDamage %" }}>rD%</th>
            {#if meterSettings.rdpsDamageReceived}
                <th class="w-14 font-normal" use:tooltip={{ content: "Total damage received from friendly synergies" }}
                    >Recv</th>
            {/if}
            {#if meterSettings.rdpsDamageGiven}
                <th class="w-14 font-normal" use:tooltip={{ content: "Total damage given with your synergies" }}
                    >Given</th>
            {/if}
            {#if meterSettings.rdpsContribution}
                <th class="w-14 font-normal" use:tooltip={{ content: "% of your Damage that is from all synergies" }}
                    >Con%</th>
            {/if}
            {#if meterSettings.rdpsSContribution}
                <th class="w-14 font-normal" use:tooltip={{ content: "% of your Damage that is from the Support" }}
                    >sCon%</th>
            {/if}
            {#if meterSettings.rdpsDContribution}
                <th class="w-14 font-normal" use:tooltip={{ content: "% of your Damage that is from Dealers" }}
                    >dCon%</th>
            {/if}
            {#if meterSettings.rdpsSyn}
                <th class="w-14 font-normal" use:tooltip={{ content: "% of Damage gained all synergies" }}>Syn%</th>
            {/if}
            {#if meterSettings.rdpsSSyn}
                <th class="w-14 font-normal" use:tooltip={{ content: "% of Damage gained from Support" }}>sSyn%</th>
            {/if}
            {#if meterSettings.rdpsDSyn}
                <th class="w-14 font-normal" use:tooltip={{ content: "% of Damage gained from Dealers" }}>dSyn%</th>
            {/if}
        {/if}
    </tr>
</thead>
