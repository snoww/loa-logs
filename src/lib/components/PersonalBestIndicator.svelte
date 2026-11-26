<script lang="ts">
  import QuickTooltip from "#lib/components/QuickTooltip.svelte";
  import type { EncounterState } from "#lib/encounter.svelte.ts";
  import type { EntityState } from "#lib/entity.svelte.ts";
  import { abbreviateNumber, getBossHpBars } from "#lib/utils.ts";
  import {
    evaluatePersonalBest,
    formatPbLogMeta,
    formatSignedAbbreviated,
    formatSignedFixed
  } from "#lib/utils/personalBest.ts";

  interface Props {
    enc: EncounterState;
    entityState: EntityState;
    compact?: boolean;
    previewDelta?: number;
  }

  let { enc, entityState, compact = false, previewDelta }: Props = $props();

  let comparison = $derived.by(() => {
    if (!enc.live || !enc.personalBest || !enc.encounter) return null;
    const misc = enc.encounter.encounterDamageStats.misc;
    return evaluatePersonalBest({
      pb: enc.personalBest,
      entity: entityState.entity,
      damageDealt: entityState.damageDealt,
      baseDamage: entityState.baseDamage,
      durationMs: enc.duration,
      fightStart: enc.encounter.fightStart,
      intermissionStart: misc?.intermissionStart,
      intermissionEnd: misc?.intermissionEnd,
      rdpsValid: misc?.rdpsValid,
      currentBoss: enc.encounter.currentBoss,
      localSupportBuffs: enc.encounter.localSupportBuffs
    });
  });
  let delta = $derived(previewDelta ?? (comparison?.kind === "dps" ? comparison.deltaPercent : comparison?.deltaScore));
  let deltaColor = $derived((delta ?? 0) >= 0 ? "text-lime-400" : "text-red-300");
</script>

{#snippet pbDelta(value: number)}
  <span
    class={[
      "tracking-[-0.01em] whitespace-nowrap tabular-nums",
      compact ? "w-11 shrink-0 text-right text-[10px]" : "text-xs",
      deltaColor
    ]}
    style="text-shadow: 0 1px 2px rgba(0, 0, 0, 0.85)"
  >
    {formatSignedFixed(value, 1, "%")}
  </span>
{/snippet}

{#if compact && delta !== undefined}
  {@render pbDelta(delta)}
{:else if comparison && delta !== undefined}
  {#snippet pbTooltip()}
    <span class="flex min-w-55 flex-col leading-[1.55] tracking-tighter tabular-nums">
      {#if comparison.kind === "dps"}
        {@const boss = enc.encounter?.currentBoss}
        {@const totalBars = boss ? getBossHpBars(boss) : 1}
        {@const bars = (comparison.bossHpPercent / 100) * totalBars}
        {@const remainingBars = (boss?.currentShield ?? 0) > 0 ? Math.round(bars) : Math.ceil(bars)}
        <span>
          <span class="text-neutral-400">{comparison.metric}</span>
          {abbreviateNumber(comparison.liveDps)}
          <span class="text-neutral-400">now</span>
          &middot;
          {abbreviateNumber(comparison.pbDps)}
          <span class="text-neutral-400">
            PB at {totalBars > 1 ? `${remainingBars}x` : `${comparison.bossHpPercent.toFixed(1)}% boss HP`}
          </span>
        </span>
        <span class={deltaColor}>
          {formatSignedAbbreviated(comparison.deltaDps, abbreviateNumber)} ({formatSignedFixed(
            comparison.deltaPercent,
            1,
            "%"
          )})
        </span>
      {:else}
        <span>
          <span class="text-neutral-400">Buff performance</span>
          {comparison.liveScore.toFixed(1)}%
          <span class="text-neutral-400">now</span>
          &middot;
          {comparison.pbScore.toFixed(1)}%
          <span class="text-neutral-400">PB</span>
        </span>
        <span>
          <span class="text-red-300">AP {comparison.liveAp.toFixed(1)}%</span>
          &middot;
          <span class="text-green-300">Brand {comparison.liveBrand.toFixed(1)}%</span>
          &middot;
          <span class="text-yellow-300">Identity {comparison.liveIdentity.toFixed(1)}%</span>
        </span>
      {/if}
      <span class="text-neutral-400">
        {formatPbLogMeta(comparison.logId, comparison.fightStart)}
      </span>
    </span>
  {/snippet}

  <QuickTooltip tooltip={pbTooltip} class="w-14 shrink-0 text-right">
    {@render pbDelta(delta)}
  </QuickTooltip>
{/if}
