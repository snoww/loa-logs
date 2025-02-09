<script lang="ts">
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";

    interface Props {
        meterSettings: string;
        hasBackAttacks?: boolean;
        hasFrontAttacks?: boolean;
        anySupportBrand?: boolean;
        anySupportBuff?: boolean;
        anySupportIdentity?: boolean;
    }

    let {
        meterSettings,
        hasBackAttacks = true,
        hasFrontAttacks = true,
        anySupportBrand = false,
        anySupportBuff = false,
        anySupportIdentity = false
    }: Props = $props();

    let currentSettings = $state($settings.logs);

    if (meterSettings === "logs") {
        currentSettings = $settings.logs;
    } else {
        currentSettings = $settings.meter;
    }
</script>

<th class="w-7 px-2 font-normal"></th>
<th class="w-14 px-2 text-left font-normal"></th>
<th class="w-full"></th>
{#if currentSettings.breakdown.damage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Damage Dealt" }}>DMG</th>
{/if}
{#if currentSettings.breakdown.dps}
    <th class="w-12 font-normal" use:tooltip={{ content: "Damage per second" }}>DPS</th>
{/if}
{#if currentSettings.breakdown.damagePercent}
    <th class="w-10 font-normal" use:tooltip={{ content: "Damage %" }}>D%</th>
{/if}
{#if currentSettings.breakdown.critRate}
    <th class="w-12 font-normal" use:tooltip={{ content: "Crit %" }}>CRIT</th>
{/if}
{#if meterSettings === "logs" && currentSettings.breakdown.adjustedCritRate}
    <th class="w-12 font-normal" use:tooltip={{ content: "Adjusted Crit %" }}>aCRIT</th>
{/if}
{#if currentSettings.breakdown.critDmg}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage that Crit" }}>CDMG</th>
{/if}
{#if hasFrontAttacks && currentSettings.breakdown.frontAtk}
    {#if currentSettings.positionalDmgPercent}
        <th class="w-12 font-normal" use:tooltip={{ content: "% Damage from Front Attack" }}>F.A</th>
    {:else}
        <th class="w-12 font-normal" use:tooltip={{ content: "Front Attack %" }}>F.A</th>
    {/if}
{/if}
{#if hasBackAttacks && currentSettings.breakdown.backAtk}
    {#if currentSettings.positionalDmgPercent}
        <th class="w-12 font-normal" use:tooltip={{ content: "% Damage from Back Attack" }}>B.A</th>
    {:else}
        <th class="w-12 font-normal" use:tooltip={{ content: "Back Attack %" }}>B.A</th>
    {/if}
{/if}
{#if anySupportBuff && currentSettings.breakdown.percentBuffBySup}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support Atk. Power Buff" }}>Buff%</th>
{/if}
{#if anySupportBrand && currentSettings.breakdown.percentBrand}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Brand" }}>B%</th>
{/if}
{#if anySupportIdentity && currentSettings.breakdown.percentIdentityBySup}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support Identity" }}>Iden%</th>
{/if}
{#if currentSettings.breakdown.avgDamage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Skill Average Damage per Hit" }}>APH</th>
{/if}
{#if currentSettings.breakdown.avgDamage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Skill Average Damage per Cast" }}>APC</th>
{/if}
{#if currentSettings.breakdown.maxDamage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Skill Max Hit Damage" }}>MaxH</th>
{/if}
{#if meterSettings === "logs" && currentSettings.breakdown.maxDamage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Skill Max Cast Damage" }}>MaxC</th>
{/if}
{#if currentSettings.breakdown.casts}
    <th class="w-10 font-normal" use:tooltip={{ content: "Total Casts" }}>Casts</th>
{/if}
{#if currentSettings.breakdown.cpm}
    <th class="w-10 font-normal" use:tooltip={{ content: "Casts per minute" }}>CPM</th>
{/if}
{#if currentSettings.breakdown.hits}
    <th class="w-10 font-normal" use:tooltip={{ content: "Total Hits" }}>Hits</th>
{/if}
{#if currentSettings.breakdown.hpm}
    <th class="w-10 font-normal" use:tooltip={{ content: "Hits per minute" }}>HPM</th>
{/if}
