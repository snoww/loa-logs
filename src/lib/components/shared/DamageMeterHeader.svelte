<script lang="ts">
  import type { EncounterState } from "$lib/encounter.svelte";
  import { tooltip } from "$lib/utils/tooltip";

  let { enc, anyDead = undefined }: { enc: EncounterState; anyDead?: boolean } = $props();
</script>

<th class="w-20 px-2 text-left font-normal"></th>
<th class="w-full"></th>
{#if anyDead !== undefined ? anyDead : enc.anyDead && enc.curSettings.deathTime}
  <th class="w-14 font-normal" use:tooltip={{ content: "Dead for" }}>Dead</th>
{/if}
{#if enc.multipleDeaths && enc.curSettings.deathTime}
  <th class="w-14 font-normal" use:tooltip={{ content: "Death Count" }}>Deaths</th>
{/if}
{#if enc.anyPlayerIncapacitated && enc.curSettings.incapacitatedTime}
  <th
    class="w-14 font-normal"
    use:tooltip={{
      content: "Time spent in the air, on the floor, or affected by crowd control effects."
    }}>INCAP</th
  >
{/if}
{#if enc.curSettings.damage}
  <th class="w-14 font-normal" use:tooltip={{ content: "Damage Dealt" }}>DMG</th>
{/if}
{#if enc.curSettings.dps}
  <th class="w-14 font-normal" use:tooltip={{ content: "Damage per second" }}>DPS</th>
{/if}
{#if !enc.isSolo && enc.curSettings.damagePercent}
  <th class="w-12 font-normal" use:tooltip={{ content: "Damage %" }}>D%</th>
{/if}
{#if enc.curSettings.critRate}
  <th class="w-12 font-normal" use:tooltip={{ content: "Crit %" }}>CRIT</th>
{/if}
{#if enc.curSettings.critDmg}
  <th class="w-12 font-normal" use:tooltip={{ content: "% Damage that Crit" }}>CDMG</th>
{/if}
{#if enc.anyFrontAtk && enc.curSettings.frontAtk}
  <th class="w-12 font-normal" use:tooltip={{ content: "Front Attack %" }}>F.A</th>
{/if}
{#if enc.anyBackAtk && enc.curSettings.backAtk}
  <th class="w-12 font-normal" use:tooltip={{ content: "Back Attack %" }}>B.A</th>
{/if}
{#if enc.anySupportBuff && enc.curSettings.percentBuffBySup}
  <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support Atk. Power buff" }}>Buff% </th>
{/if}
{#if enc.anySupportBrand && enc.curSettings.percentBrand}
  <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Brand" }}>B%</th>
{/if}
{#if enc.anySupportIdentity && enc.curSettings.percentIdentityBySup}
  <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support Identity" }}>Iden% </th>
{/if}
{#if enc.anySupportHat && enc.curSettings.percentHatBySup}
  <th
    class="w-12 font-normal"
    use:tooltip={{
      content: "% Damage buffed by Support Hyper Awakening Technique (T Skill)"
    }}
    >T%
  </th>
{/if}
{#if enc.curSettings.counters}
  <th class="w-12 font-normal" use:tooltip={{ content: "Counters" }}>CTR</th>
{/if}
