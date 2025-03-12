<script lang="ts">
    import type { EntityState } from "$lib/entity.svelte";
    import { tooltip } from "$lib/utils/tooltip";

    let { entityState }: { entityState: EntityState } = $props();

</script>

<th class="w-7 px-2 font-normal"></th>
<th class="w-14 px-2 text-left font-normal"></th>
<th class="w-full"></th>
{#if entityState.enc.curSettings.breakdown.damage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Damage Dealt" }}>DMG</th>
{/if}
{#if entityState.enc.curSettings.breakdown.dps}
    <th class="w-12 font-normal" use:tooltip={{ content: "Damage per second" }}>DPS</th>
{/if}
{#if entityState.enc.curSettings.breakdown.damagePercent}
    <th class="w-10 font-normal" use:tooltip={{ content: "Damage %" }}>D%</th>
{/if}
{#if entityState.enc.curSettings.breakdown.critRate}
    <th class="w-12 font-normal" use:tooltip={{ content: "Crit %" }}>CRIT</th>
{/if}
{#if !entityState.enc.live && entityState.enc.curSettings.breakdown.adjustedCritRate}
    <th class="w-12 font-normal" use:tooltip={{ content: "Adjusted Crit %" }}>aCRIT</th>
{/if}
{#if entityState.enc.curSettings.breakdown.critDmg}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage that Crit" }}>CDMG</th>
{/if}
{#if entityState.anyFrontAttacks && entityState.enc.curSettings.breakdown.frontAtk}
    {#if entityState.enc.curSettings.positionalDmgPercent}
        <th class="w-12 font-normal" use:tooltip={{ content: "% Damage from Front Attack" }}>F.A</th>
    {:else}
        <th class="w-12 font-normal" use:tooltip={{ content: "Front Attack %" }}>F.A</th>
    {/if}
{/if}
{#if entityState.anyBackAttacks && entityState.enc.curSettings.breakdown.backAtk}
    {#if entityState.enc.curSettings.positionalDmgPercent}
        <th class="w-12 font-normal" use:tooltip={{ content: "% Damage from Back Attack" }}>B.A</th>
    {:else}
        <th class="w-12 font-normal" use:tooltip={{ content: "Back Attack %" }}>B.A</th>
    {/if}
{/if}
{#if entityState.anySupportBuff && entityState.enc.curSettings.breakdown.percentBuffBySup}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support Atk. Power Buff" }}>Buff%</th>
{/if}
{#if entityState.anySupportBrand && entityState.enc.curSettings.breakdown.percentBrand}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Brand" }}>B%</th>
{/if}
{#if entityState.anySupportIdentity && entityState.enc.curSettings.breakdown.percentIdentityBySup}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support Identity" }}>Iden%</th>
{/if}
{#if entityState.anySupportHat && entityState.enc.curSettings.breakdown.percentHatBySup}
    <th
        class="w-12 font-normal"
        use:tooltip={{ content: "% Damage buffed by Support Hyper Awakening Technique (T Skill)" }}>T%</th>
{/if}
{#if entityState.enc.curSettings.breakdown.avgDamage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Skill Average Damage per Hit" }}>APH</th>
{/if}
{#if entityState.enc.curSettings.breakdown.avgDamage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Skill Average Damage per Cast" }}>APC</th>
{/if}
{#if entityState.enc.curSettings.breakdown.maxDamage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Skill Max Hit Damage" }}>MaxH</th>
{/if}
{#if !entityState.enc.live && entityState.enc.curSettings.breakdown.maxDamage}
    <th class="w-12 font-normal" use:tooltip={{ content: "Skill Max Cast Damage" }}>MaxC</th>
{/if}
{#if entityState.enc.curSettings.breakdown.casts}
    <th class="w-10 font-normal" use:tooltip={{ content: "Total Casts" }}>Casts</th>
{/if}
{#if entityState.enc.curSettings.breakdown.cpm}
    <th class="w-10 font-normal" use:tooltip={{ content: "Casts per minute" }}>CPM</th>
{/if}
{#if entityState.enc.curSettings.breakdown.hits}
    <th class="w-10 font-normal" use:tooltip={{ content: "Total Hits" }}>Hits</th>
{/if}
{#if entityState.enc.curSettings.breakdown.hpm}
    <th class="w-10 font-normal" use:tooltip={{ content: "Hits per minute" }}>HPM</th>
{/if}
