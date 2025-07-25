<script module lang="ts">
  import type { LogColumn } from "$lib/column";
  import type { EncounterState } from "$lib/encounter.svelte.js";
  import { EntityState } from "$lib/entity.svelte.js";
  import { customRound } from "$lib/utils";
  import { badTooltip, damageValue, fadTooltip, percentValue } from "./Snippets.svelte";

  export const logColumns: LogColumn<EncounterState, EntityState>[] = [
    // Dead for
    {
      show(enc) {
        if (!enc.curSettings.deathTime) return false;
        return enc.anyDead;
      },
      headerText: "Dead",
      headerTooltip: "Dead for",
      value: deadFor,
      valueTooltip: null
    },

    // Number of deaths
    {
      show(enc) {
        if (!enc.curSettings.deathTime) return false;
        return enc.multipleDeaths;
      },
      headerText: "Deaths",
      headerTooltip: "Death Count",
      value: deaths,
      valueTooltip: null,
      width: "w-14"
    },

    // Incapacitation time
    {
      show(enc) {
        if (!enc.curSettings.incapacitatedTime) return false;
        return enc.anyPlayerIncapacitated;
      },
      headerText: "INCAP",
      headerTooltip: "Time spent in the air, on the floor, or affected by crowd control effects.",
      value: incap,
      valueTooltip: incapTooltip
    },

    // Damage dealt
    {
      show(enc) {
        if (!enc.curSettings.damage) return false;
        return true;
      },
      headerText: "DMG",
      headerTooltip: "Damage Dealt",
      value: damage,
      valueTooltip: damageTooltip,
      width: "w-14"
    },

    // Damage per second
    {
      show(enc) {
        if (!enc.curSettings.dps) return false;
        return true;
      },
      headerText: "DPS",
      headerTooltip: "Damage per second",
      value: dps,
      valueTooltip: dpsTooltip,
      width: "w-14"
    },

    // Damage percentage
    {
      show(enc) {
        if (!enc.curSettings.damagePercent) return false;
        return !enc.isSolo;
      },
      headerText: "D%",
      headerTooltip: "Damage %",
      value: damagePct,
      valueTooltip: null
    },

    // Crit rate
    {
      show(enc) {
        if (!enc.curSettings.critRate) return false;
        return true;
      },
      headerText: "CRIT",
      headerTooltip: "Crit %",
      value: critPct,
      valueTooltip: null
    },

    // Crit damage percentage
    {
      show(enc) {
        if (!enc.curSettings.critDmg) return false;
        return true;
      },
      headerText: "CDMG",
      headerTooltip: "% Damage that Crit",
      value: critDmgPct,
      valueTooltip: null,
      width: "w-14"
    },

    // Front attack percentage
    {
      show(enc) {
        if (!enc.curSettings.frontAtk) return false;
        return !enc.curSettings.positionalDmgPercent && enc.anyFrontAtk;
      },
      headerText: "F.A",
      headerTooltip: "Front Attack %",
      value: faPct,
      valueTooltip: null
    },

    // Front attack damage percentage
    {
      show(enc) {
        if (!enc.curSettings.frontAtk) return false;
        return enc.curSettings.positionalDmgPercent && enc.anyFrontAtk;
      },
      headerText: "F.AD%",
      headerTooltip: "Front Attack Damage %",
      value: fadPct,
      valueTooltip: fadTooltip,
      width: "w-14"
    },

    // Back attack percentage
    {
      show(enc) {
        if (!enc.curSettings.backAtk) return false;
        return !enc.curSettings.positionalDmgPercent && enc.anyBackAtk;
      },
      headerText: "B.A",
      headerTooltip: "Back Attack %",
      value: baPct,
      valueTooltip: null
    },

    // Back attack damage percentage
    {
      show(enc) {
        if (!enc.curSettings.backAtk) return false;
        return enc.curSettings.positionalDmgPercent && enc.anyBackAtk;
      },
      headerText: "B.AD%",
      headerTooltip: "Back Attack Damage %",
      value: badPct,
      valueTooltip: badTooltip,
      width: "w-14"
    },

    // Support buff percentage
    {
      show(enc) {
        if (!enc.curSettings.percentBuffBySup) return false;
        return enc.anySupportBuff;
      },
      headerText: "Buff%",
      headerTooltip: "% Damage buffed by Support Atk. Power buff",
      value: buffPct,
      valueTooltip: null
    },

    // Brand percentage
    {
      show(enc) {
        if (!enc.curSettings.percentBrand) return false;
        return enc.anySupportBrand;
      },
      headerText: "B%",
      headerTooltip: "% Damage buffed by Brand",
      value: brandPct,
      valueTooltip: null
    },

    // Identity percentage
    {
      show(enc) {
        if (!enc.curSettings.percentIdentityBySup) return false;
        return enc.anySupportIdentity;
      },
      headerText: "Iden%",
      headerTooltip: "% Damage buffed by Support Identity",
      value: identityPct,
      valueTooltip: null
    },

    // Hat percentage
    {
      show(enc) {
        if (!enc.curSettings.percentHatBySup) return false;
        return enc.anySupportHat;
      },
      headerText: "T%",
      headerTooltip: "% Damage buffed by Support Hyper Awakening Skill (T Skill)",
      value: hatPct,
      valueTooltip: null
    },

    // Counters
    {
      show(enc) {
        if (!enc.curSettings.counters) return false;
        return true;
      },
      headerText: "CTR",
      headerTooltip: "Counters",
      value: counters,
      valueTooltip: null,
      width: "w-10"
    }
  ];
</script>

{#snippet deadFor(state: EntityState)}
  {#if state.entity.isDead}
    {state.deadFor}
  {/if}
{/snippet}

{#snippet deaths(state: EntityState)}
  {#if state.entity.damageStats.deaths > 0}
    {state.entity.damageStats.deaths}
  {:else}
    -
  {/if}
{/snippet}

{#snippet incap(state: EntityState)}
  {(state.incapacitatedTimeMs.total / 1000).toFixed(1)}s
{/snippet}

{#snippet incapTooltip(state: EntityState)}
  {@const { knockDown, cc } = state.incapacitatedTimeMs}
  <div class="-mx-px flex flex-col space-y-1 py-px text-xs font-normal">
    <span class="text-gray-300">Knockdowns: {(knockDown / 1000).toFixed(1)}s</span>
    <span class="text-gray-300">Crowd control: {(cc / 1000).toFixed(1)}s</span>
  </div>
{/snippet}

{#snippet damage(state: EntityState)}
  {@render damageValue(state.damageDealtString)}
{/snippet}

{#snippet damageTooltip(state: EntityState)}
  {state.damageDealt.toLocaleString()}
{/snippet}

{#snippet dps(state: EntityState)}
  {@render damageValue(state.dpsString)}
{/snippet}

{#snippet dpsTooltip(state: EntityState)}
  {state.dps.toLocaleString()}
{/snippet}

{#snippet damagePct(state: EntityState)}
  {@render percentValue(state.damagePercentage)}
{/snippet}

{#snippet critPct(state: EntityState)}
  {@render percentValue(state.critPercentage)}
{/snippet}

{#snippet critDmgPct(state: EntityState)}
  {@render percentValue(state.critDmgPercentage)}
{/snippet}

{#snippet faPct(state: EntityState)}
  {@render percentValue(state.faPercentage)}
{/snippet}

{#snippet fadPct(state: EntityState)}
  {@render percentValue(state.fadPercentage)}
{/snippet}

{#snippet baPct(state: EntityState)}
  {@render percentValue(state.baPercentage)}
{/snippet}

{#snippet badPct(state: EntityState)}
  {@render percentValue(state.badPercentage)}
{/snippet}

{#snippet buffPct(state: EntityState)}
  {@render percentValue(
    customRound((state.entity.damageStats.buffedBySupport / state.damageDealtWithoutSpecialOrHa) * 100)
  )}
{/snippet}

{#snippet brandPct(state: EntityState)}
  {@render percentValue(
    customRound((state.entity.damageStats.debuffedBySupport / state.damageDealtWithoutSpecialOrHa) * 100)
  )}
{/snippet}

{#snippet identityPct(state: EntityState)}
  {@render percentValue(
    customRound((state.entity.damageStats.buffedByIdentity / state.damageDealtWithoutSpecialOrHa) * 100)
  )}
{/snippet}

{#snippet hatPct(state: EntityState)}
  {@render percentValue(
    customRound(((state.entity.damageStats.buffedByHat ?? 0) / state.damageDealtWithoutSpecial) * 100)
  )}
{/snippet}

{#snippet counters(state: EntityState)}
  {state.entity.skillStats.counters}
{/snippet}
