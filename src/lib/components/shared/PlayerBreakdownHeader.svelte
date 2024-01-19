<script lang="ts">
    import { tooltip } from "$lib/utils/tooltip";

    export let meterSettings: any;
    export let hasBackAttacks = true;
    export let hasFrontAttacks = true;
    export let anySupportBrand = false;
    export let anySupportBuff = false;
    export let anySupportIdentity = false;
</script>

<th class="w-7 px-2 font-normal" />
<th class="w-14 px-2 text-left font-normal" />
<th class="w-full" />
{#if meterSettings.breakdown.damage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Damage Dealt" }}>DMG</th>
{/if}
{#if meterSettings.breakdown.dps}
    <th class="w-12 font-normal" use:tooltip={{ content: "Damage per second" }}>DPS</th>
{/if}
{#if meterSettings.breakdown.damagePercent}
    <th class="w-10 font-normal" use:tooltip={{ content: "Damage %" }}>D%</th>
{/if}
{#if meterSettings.breakdown.critRate}
    <th class="w-12 font-normal" use:tooltip={{ content: "Crit %" }}>CRIT</th>
{/if}
{#if meterSettings.breakdown.critDmg}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage that Crit" }}>CDMG</th>
{/if}
{#if hasFrontAttacks && meterSettings.breakdown.frontAtk}
    {#if meterSettings.positionalDmgPercent}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage from Front Attack" }}>F.A</th>
    {:else}
    <th class="w-12 font-normal" use:tooltip={{ content: "Front Attack %" }}>F.A</th>
    {/if}
{/if}
{#if hasBackAttacks && meterSettings.breakdown.backAtk}
    {#if meterSettings.positionalDmgPercent}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage from Back Attack" }}>B.A</th>
    {:else}
    <th class="w-12 font-normal" use:tooltip={{ content: "Back Attack %" }}>B.A</th>
    {/if}
{/if}
{#if anySupportBuff && meterSettings.breakdown.percentBuffBySup}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support Atk. Power Buff" }}>Buff%</th>
{/if}
{#if anySupportIdentity && meterSettings.breakdown.percentIdentityBySup}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support Identity" }}>Iden%</th>
{/if}
{#if anySupportBrand && meterSettings.breakdown.percentBrand}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Brand" }}>B%</th>
{/if}
{#if meterSettings.breakdown.avgDamage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Skill Average Damage per Hit" }}>APH</th>
{/if}
{#if meterSettings.breakdown.avgDamage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Skill Average Damage per Cast" }}>APC</th>
{/if}
{#if meterSettings.breakdown.maxDamage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Skill Max Damage" }}>Max</th>
{/if}
{#if meterSettings.breakdown.casts}
    <th class="w-10 font-normal" use:tooltip={{ content: "Total Casts" }}>Casts</th>
{/if}
{#if meterSettings.breakdown.cpm}
    <th class="w-10 font-normal" use:tooltip={{ content: "Casts per minute" }}>CPM</th>
{/if}
{#if meterSettings.breakdown.hits}
    <th class="w-10 font-normal" use:tooltip={{ content: "Total Hits" }}>Hits</th>
{/if}
{#if meterSettings.breakdown.hpm}
    <th class="w-10 font-normal" use:tooltip={{ content: "Hits per minute" }}>HPM</th>
{/if}
