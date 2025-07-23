<script lang="ts">
  import type { Skill } from "$lib/types";

  const { skill }: { skill: Skill } = $props();
  function getColorFromTier(tier?: number) {
    if (tier === 4) {
      return "#d96dff";
    } else {
      return "#e08d14";
    }
  }

  function getColorFromLevel(level: number, tier?: number) {
    if (tier === 4) {
      if (level === 3 || level === 4) {
        return "#a23ac7";
      } else if (level === 5 || level === 6 || level === 7) {
        return "#e08d14";
      } else if (level === 8 || level === 9) {
        return "#ed691a";
      } else if (level === 10) {
        return "#e7c990";
      } else {
        return "#e5e7eb";
      }
    } else {
      if (level === 5 || level === 6) {
        return "#a23ac7";
      } else if (level === 7 || level === 8 || level === 9) {
        return "#e08d14";
      } else if (level === 10) {
        return "#ed691a";
      } else {
        return "#e5e7eb";
      }
    }
  }

  const TRIPOD_COLORS = ["bg-blue-800", "bg-lime-600", "bg-amber-600"];
</script>

{#snippet gem(tier: number, level: number, type: string)}
  <div class="flex gap-1">
    <span style="color: {getColorFromTier(tier)}">T{tier}</span>
    <span style="color: {getColorFromLevel(level, tier)}">Lv. {level}</span>
    <span>{type}</span>
  </div>
{/snippet}

{#snippet tripodRow(col: number, index: number, level?: number)}
  {#if index > 0}
    {@const opts = col < 3 ? [1, 2, 3] : [1, 2]}
    <div class="flex justify-center gap-1 py-0.5">
      {#each opts as i}
        {#if i === index}
          <div class="flex size-5 items-center justify-center rounded-full {TRIPOD_COLORS[col - 1]}">
            <p class="text-neutral-200">{level || 1}</p>
          </div>
        {:else}
          <div class="flex size-5 items-center justify-center rounded-full bg-neutral-700"></div>
        {/if}
      {/each}
    </div>
  {/if}
{/snippet}

{#snippet tripod(col: number, index: number)}
  <div class="flex size-5 items-center justify-center rounded-full {TRIPOD_COLORS[col - 1]}">
    <p class="text-neutral-200">{index}</p>
  </div>
{/snippet}

<div class="text-xs">
  <div class="py-0.5">{skill.name}</div>
  <div class="text-neutral-300">
    {#if skill.gemDamage}
      {@render gem(skill.gemTierDmg ?? skill.gemTier ?? 3, skill.gemDamage, "DMG")}
    {/if}
    {#if skill.gemCooldown}
      {@render gem(skill.gemTier ?? 3, skill.gemCooldown, "CD")}
    {/if}
  </div>
  <!-- logs before paradise update show tripod levels -->
  {#if skill.tripodIndex && skill.tripodLevel}
    <div class="w-16">
      {@render tripodRow(1, skill.tripodIndex.first, skill.tripodLevel?.first)}
      {@render tripodRow(2, skill.tripodIndex.second, skill.tripodLevel?.second)}
      {@render tripodRow(3, skill.tripodIndex.third, skill.tripodLevel?.third)}
    </div>
    <!-- only show tripod index -->
  {:else if skill.tripodIndex}
    <div class="flex items-center gap-0.5 py-1">
      {@render tripod(1, skill.tripodIndex.first)}
      {@render tripod(2, skill.tripodIndex.second)}
      {@render tripod(3, skill.tripodIndex.third)}
    </div>
  {/if}
</div>
