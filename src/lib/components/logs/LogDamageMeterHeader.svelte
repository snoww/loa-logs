<script lang="ts">
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";

    interface Props {
        anyDead: boolean;
        multipleDeaths: boolean;
        anyFrontAtk: boolean;
        anyBackAtk: boolean;
        anySupportBuff: boolean;
        anySupportIdentity: boolean;
        anySupportBrand: boolean;
        anySupportHat: boolean;
        anyRdpsData: boolean;
        anyPlayerIncapacitated: boolean;
        isSolo: boolean;
    }

    let {
        anyDead,
        multipleDeaths,
        anyFrontAtk,
        anyBackAtk,
        anySupportBuff,
        anySupportIdentity,
        anySupportBrand,
        anySupportHat,
        anyRdpsData,
        anyPlayerIncapacitated,
        isSolo
    }: Props = $props();
</script>

{#if anyDead && $settings.logs.deathTime}
    <th class="w-16 font-normal" use:tooltip={{ content: "Dead for" }}>Dead for</th>
{/if}
{#if multipleDeaths && $settings.logs.deathTime}
    <th class="w-14 font-normal" use:tooltip={{ content: "Death Count" }}>Deaths</th>
{/if}
{#if anyPlayerIncapacitated && $settings.logs.incapacitatedTime}
    <th
        class="w-14 font-normal"
        use:tooltip={{
            content: "Time spent in the air, on the floor, or affected by crowd control effects."
        }}>INCAP</th>
{/if}
{#if $settings.logs.damage}
    <th class="w-14 font-normal" use:tooltip={{ content: "Damage Dealt" }}>DMG</th>
{/if}
{#if $settings.logs.dps}
    <th class="w-14 font-normal" use:tooltip={{ content: "Damage per second" }}>DPS</th>
{/if}
{#if !isSolo && $settings.logs.damagePercent}
    <th class="w-12 font-normal" use:tooltip={{ content: "Damage %" }}>D%</th>
{/if}
{#if $settings.logs.critRate}
    <th class="w-12 font-normal" use:tooltip={{ content: "Crit %" }}>CRIT</th>
{/if}
{#if $settings.logs.critDmg}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage that Crit" }}>CDMG </th>
{/if}
{#if anyFrontAtk && $settings.logs.frontAtk}
    <th class="w-12 font-normal" use:tooltip={{ content: "Front Attack %" }}>F.A</th>
{/if}
{#if anyBackAtk && $settings.logs.backAtk}
    <th class="w-12 font-normal" use:tooltip={{ content: "Back Attack %" }}>B.A</th>
{/if}
{#if anySupportBuff && $settings.logs.percentBuffBySup}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support" }}>Buff% </th>
{/if}
{#if anySupportBrand && $settings.logs.percentBrand}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Brand" }}>B% </th>
{/if}
{#if anySupportIdentity && $settings.logs.percentIdentityBySup}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support Identity" }}>Iden% </th>
{/if}
{#if anySupportHat && $settings.logs.percentHatBySup}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support Hyper Awakening Technique" }}>HAT% </th>
{/if}
{#if anyRdpsData && $settings.logs.ssyn}
    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage gained from Support" }}>sSyn% </th>
{/if}
{#if $settings.logs.counters}
    <th class="w-12 font-normal" use:tooltip={{ content: "Counters" }}>CTR</th>
{/if}
