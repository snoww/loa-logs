<script module lang="ts">
  import type { LogColumn } from "$lib/column";
  import { EntityState } from "$lib/entity.svelte.js";
  import { SkillState } from "$lib/skill.svelte.js";
  import { settings } from "$lib/stores.svelte.js";
  import { hyperAwakeningIds } from "$lib/utils/buffs";
  import { abbreviateNumberSplit, customRound } from "$lib/utils";
  import { damageValue, percentValue } from "./Snippets.svelte";

  export const playerBreakdownColumns: LogColumn<EntityState, SkillState>[] = [
    // Damage
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.damage;
      },
      headerText: "DMG",
      headerTooltip: "Damage Dealt",
      value: damage,
      valueTooltip: damageTooltip
    },

    // DPS
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.dps;
      },
      headerText: "DPS",
      headerTooltip: "Damage per second",
      value: dps,
      valueTooltip: dpsTooltip
    },

    // Damage %
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.damagePercent;
      },
      headerText: "D%",
      headerTooltip: "Damage %",
      value: damagePct,
      valueTooltip: null
    },

    // Crit %
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.critRate;
      },
      headerText: "CRIT",
      headerTooltip: "Crit %",
      value: critPct,
      valueTooltip: null
    },

    // Adjusted Crit %
    {
      show(enc) {
        return !enc.encounter.live && settings.app.logs.breakdown.adjustedCritRate;
      },
      headerText: "aCRIT",
      headerTooltip: "Adjusted Crit %",
      value: adjCritPct,
      valueTooltip: null,
      width: "w-14"
    },

    // Crit Damage %
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.critDmg;
      },
      headerText: "CDMG",
      headerTooltip: "% Damage that Crit",
      value: critDmgPct,
      valueTooltip: null,
      width: "w-14"
    },

    // Front Attack %
    {
      show(enc) {
        return (
          !enc.encounter.curSettings.positionalDmgPercent &&
          enc.encounter.curSettings.breakdown.frontAtk &&
          enc.anyFrontAttacks
        );
      },
      headerText: "F.A",
      headerTooltip: "Front Attack %",
      value: faPct,
      valueTooltip: null
    },

    // Front Attack Damage %
    {
      show(enc) {
        return (
          enc.encounter.curSettings.positionalDmgPercent &&
          enc.encounter.curSettings.breakdown.frontAtk &&
          enc.anyFrontAttacks
        );
      },
      headerText: "F.AD%",
      headerTooltip: "Front Attack Damage %",
      value: fadPct,
      valueTooltip: fadTooltip,
      width: "w-14"
    },

    // Back Attack %
    {
      show(enc) {
        return (
          !enc.encounter.curSettings.positionalDmgPercent &&
          enc.encounter.curSettings.breakdown.backAtk &&
          enc.anyBackAttacks
        );
      },
      headerText: "B.A",
      headerTooltip: "Back Attack %",
      value: baPct,
      valueTooltip: null
    },

    // Back Attack Damage %
    {
      show(enc) {
        return (
          enc.encounter.curSettings.positionalDmgPercent &&
          enc.encounter.curSettings.breakdown.backAtk &&
          enc.anyBackAttacks
        );
      },
      headerText: "B.AD%",
      headerTooltip: "Back Attack Damage %",
      value: badPct,
      valueTooltip: badTooltip,
      width: "w-14"
    },

    // Support Buff %
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.percentBuffBySup && enc.anySupportBuff;
      },
      headerText: "Buff%",
      headerTooltip: "% Damage buffed by Support Atk. Power Buff",
      value: buffPct,
      valueTooltip: null
    },

    // Brand %
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.percentBrand && enc.anySupportBrand;
      },
      headerText: "B%",
      headerTooltip: "% Damage buffed by Brand",
      value: brandPct,
      valueTooltip: null
    },

    // Iden %
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.percentIdentityBySup && enc.anySupportIdentity;
      },
      headerText: "Iden%",
      headerTooltip: "% Damage buffed by Support Identity",
      value: idenPct,
      valueTooltip: null
    },

    // T%
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.percentHatBySup && enc.anySupportHat;
      },
      headerText: "T%",
      headerTooltip: "% Damage buffed by Support Hyper Awakening Skill (T Skill)",
      value: tSkillPct,
      valueTooltip: null
    },

    // APH
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.avgDamage;
      },
      headerText: "APH",
      headerTooltip: "Skill Average Damage per Hit",
      value: avgPerHit,
      valueTooltip: avgPerHitTooltip
    },

    // APC
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.avgDamage;
      },
      headerText: "APC",
      headerTooltip: "Skill Average Damage per Cast",
      value: avgPerCast,
      valueTooltip: avgPerCastTooltip
    },

    // Max Hit
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.maxDamage;
      },
      headerText: "MaxH",
      headerTooltip: "Skill Max Hit Damage",
      value: maxHit,
      valueTooltip: maxHitTooltip
    },

    // Max Cast
    {
      show(enc) {
        return !enc.encounter.live && enc.encounter.curSettings.breakdown.maxDamage;
      },
      headerText: "MaxC",
      headerTooltip: "Skill Max Cast Damage",
      value: maxCast,
      valueTooltip: maxCastTooltip
    },

    // Casts
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.casts;
      },
      headerText: "Casts",
      headerTooltip: "Number of casts",
      value: casts,
      valueTooltip: null
    },

    // CPM
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.cpm;
      },
      headerText: "CPM",
      headerTooltip: "Casts per minute",
      value: castsPerMinute,
      valueTooltip: null
    },

    // Hits
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.hits;
      },
      headerText: "Hits",
      headerTooltip: "Number of hits",
      value: hits,
      valueTooltip: null,
      width: "w-10"
    },

    // HPM
    {
      show(enc) {
        return enc.encounter.curSettings.breakdown.hpm;
      },
      headerText: "HPM",
      headerTooltip: "Hits per minute",
      value: hitsPerMinute,
      valueTooltip: null
    },

    // Cooldown Ratio %
    {
      show(enc) {
        return enc.anyCooldownRatio;
      },
      headerText: "CDR%",
      headerTooltip: "Cooldown Ratio % - Percentage of time a skill was on cooldown",
      value: cooldownEfficiency,
      valueTooltip: cooldownEfficiencyTooltip
    }
  ];
</script>

{#snippet damage(state: SkillState)}
  {@render damageValue(state.skillDamageString)}
{/snippet}

{#snippet damageTooltip(state: SkillState)}
  {state.skill.totalDamage.toLocaleString()}
{/snippet}

{#snippet dps(state: SkillState)}
  {@render damageValue(state.skillDpsString)}
{/snippet}

{#snippet dpsTooltip(state: SkillState)}
  {state.skillDps.toLocaleString()}
{/snippet}

{#snippet damagePct(state: SkillState)}
  {@render percentValue(customRound((state.skill.totalDamage / state.entity.damageDealt) * 100))}
{/snippet}

{#snippet critPct(state: SkillState)}
  {#if state.skill.special || state.skill.isHyperAwakening || hyperAwakeningIds.has(state.skill.id)}
    -
  {:else}
    {@render percentValue(state.critPercentage)}
  {/if}
{/snippet}

{#snippet adjCritPct(state: SkillState)}
  {#if state.adjustedCrit && !state.skill.special && !state.skill.isHyperAwakening && !hyperAwakeningIds.has(state.skill.id)}
    {@render percentValue(state.adjustedCrit)}
  {:else}
    -
  {/if}
{/snippet}

{#snippet critDmgPct(state: SkillState)}
  {#if state.skill.special || state.skill.isHyperAwakening || hyperAwakeningIds.has(state.skill.id)}
    -
  {:else}
    {@render percentValue(state.critDmgPercentage)}
  {/if}
{/snippet}

{#snippet faPct(state: SkillState)}
  {#if state.skill.special || state.skill.isHyperAwakening || hyperAwakeningIds.has(state.skill.id)}
    -
  {:else}
    {@render percentValue(state.faPercentage)}
  {/if}
{/snippet}

{#snippet fadPct(state: SkillState)}
  {#if state.skill.special || state.skill.isHyperAwakening || hyperAwakeningIds.has(state.skill.id)}
    -
  {:else}
    {@render percentValue(state.fadPercentage)}
  {/if}
{/snippet}

{#snippet fadTooltip(state: SkillState)}
  <span>
    Raw Front Attack
    {@render percentValue(state.faPercentage)}
  </span>
{/snippet}

{#snippet baPct(state: SkillState)}
  {#if state.skill.special || state.skill.isHyperAwakening || hyperAwakeningIds.has(state.skill.id)}
    -
  {:else}
    {@render percentValue(state.baPercentage)}
  {/if}
{/snippet}

{#snippet badPct(state: SkillState)}
  {#if state.skill.special || state.skill.isHyperAwakening || hyperAwakeningIds.has(state.skill.id)}
    -
  {:else}
    {@render percentValue(state.badPercentage)}
  {/if}
{/snippet}

{#snippet badTooltip(state: SkillState)}
  <span>
    Raw Back Attack
    {@render percentValue(state.baPercentage)}
  </span>
{/snippet}

{#snippet buffPct(state: SkillState)}
  {#if state.skill.special || state.skill.isHyperAwakening || hyperAwakeningIds.has(state.skill.id)}
    -
  {:else if state.skill.totalDamage > 0}
    {@render percentValue(customRound((state.skill.buffedBySupport / state.skill.totalDamage) * 100))}
  {:else}
    {@render percentValue(0)}
  {/if}
{/snippet}

{#snippet brandPct(state: SkillState)}
  {#if state.skill.special || state.skill.isHyperAwakening || hyperAwakeningIds.has(state.skill.id)}
    -
  {:else if state.skill.totalDamage > 0}
    {@render percentValue(customRound((state.skill.debuffedBySupport / state.skill.totalDamage) * 100))}
  {:else}
    {@render percentValue(0)}
  {/if}
{/snippet}

{#snippet idenPct(state: SkillState)}
  {#if state.skill.special || state.skill.isHyperAwakening || hyperAwakeningIds.has(state.skill.id)}
    -
  {:else if state.skill.totalDamage > 0}
    {@render percentValue(customRound((state.skill.buffedByIdentity / state.skill.totalDamage) * 100))}
  {:else}
    {@render percentValue(0)}
  {/if}
{/snippet}

{#snippet tSkillPct(state: SkillState)}
  {#if state.skill.special}
    -
  {:else if state.skill.totalDamage > 0}
    {@render percentValue(customRound(((state.skill.buffedByHat ?? 0) / state.skill.totalDamage) * 100))}
  {:else}
    {@render percentValue(0)}
  {/if}
{/snippet}

{#snippet avgPerHit(state: SkillState)}
  {@const averagePerHit = state.skill.hits > 0 ? state.skill.totalDamage / state.skill.hits : 0}
  {@render damageValue(abbreviateNumberSplit(averagePerHit))}
{/snippet}

{#snippet avgPerHitTooltip(state: SkillState)}
  {@const averagePerHit = state.skill.hits > 0 ? state.skill.totalDamage / state.skill.hits : 0}
  {Math.round(averagePerHit).toLocaleString()}
{/snippet}

{#snippet avgPerCast(state: SkillState)}
  {@render damageValue(abbreviateNumberSplit(state.averagePerCast))}
{/snippet}

{#snippet avgPerCastTooltip(state: SkillState)}
  {Math.round(state.averagePerCast).toLocaleString()}
{/snippet}

{#snippet maxHit(state: SkillState)}
  {@render damageValue(abbreviateNumberSplit(state.skill.maxDamage))}
{/snippet}

{#snippet maxHitTooltip(state: SkillState)}
  {state.skill.maxDamage.toLocaleString()}
{/snippet}

{#snippet maxCast(state: SkillState)}
  {#if state.skill.maxDamageCast > 0}
    {@render damageValue(abbreviateNumberSplit(state.skill.maxDamageCast))}
  {:else}
    -
  {/if}
{/snippet}

{#snippet maxCastTooltip(state: SkillState)}
  {state.skill.maxDamageCast.toLocaleString()}
{/snippet}

{#snippet casts(state: SkillState)}
  {state.skill.casts}
{/snippet}

{#snippet castsPerMinute(state: SkillState)}
  {customRound(state.skill.casts / (state.entity.encounter.duration / 1000 / 60))}
{/snippet}

{#snippet hits(state: SkillState)}
  {state.skill.hits}
{/snippet}

{#snippet hitsPerMinute(state: SkillState)}
  {#if state.skill.hits > 0}
    {customRound(state.skill.hits / (state.entity.encounter.duration / 1000 / 60))}
  {:else}
    0
  {/if}
{/snippet}

{#snippet cooldownEfficiency(state: SkillState)}
  {#if state.cooldownRatio}
    {@render percentValue(state.cooldownRatio)}
  {:else}
    -
  {/if}
{/snippet}

{#snippet cooldownEfficiencyTooltip(state: SkillState)}
  {#if state.cooldownRatio}
    <span> {state.skill.name} was on cooldown for {state.cooldownRatio}% of the fight</span>
  {:else}
    <span>N/A</span>
  {/if}
{/snippet}
