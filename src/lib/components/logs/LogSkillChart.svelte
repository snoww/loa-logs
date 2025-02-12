<script lang="ts">
    import type {
        EncounterDamageStats,
        Entity,
        Skill,
        SkillCast,
        SkillChartModInfo,
        SkillChartSupportDamage, SkillHit,
        StatusEffectWithId
    } from "$lib/types";
    import { getSkillCastBuffs } from "$lib/utils/buffs";
    import { chartable } from "$lib/utils/charts";
    import { abbreviateNumber, formatDurationFromMs, round } from "$lib/utils/numbers";
    import { settings, skillIcon } from "$lib/utils/settings";
    import { focusedSkillCast } from "$lib/utils/stores";
    import { getSkillIcon } from "$lib/utils/strings";
    import { menuTooltip, tooltip } from "$lib/utils/tooltip";
    import { onDestroy, onMount } from "svelte";
    import BuffTooltip from "../shared/BuffTooltip.svelte";
    import { writable } from "svelte/store";

    interface Props {
        chartOptions: any;
        player: Entity | null;
        encounterDamageStats: EncounterDamageStats;
    }

    let { chartOptions, player, encounterDamageStats }: Props = $props();

    onMount(() => {
        focusedSkillCast.set({ skillId: 0, cast: 0 });
    });

    onDestroy(() => {
        focusedSkillCast.set({ skillId: 0, cast: 0 });
    });

    let buffType = writable("party");

    let skill: Skill = $state({ skillCastLog: Array<SkillCast>() } as Skill);
    let skillCast: SkillCast = $state({ hits: Array<SkillHit>() } as SkillCast);
    let totalDamage: number = $state(0);

    let supportBuffs: SkillChartSupportDamage = $state({ buff: 0, brand: 0, identity: 0 });
    let modInfo: SkillChartModInfo = $state({ crit: 0, critDamage: 0, ba: 0, fa: 0 });

    let allGroupedBuffs: Map<string, Array<StatusEffectWithId>>[] = $state([]);

    $effect(() => {
        if ($focusedSkillCast.skillId > 0 && player) {
            skill = player.skills[$focusedSkillCast.skillId];
            skillCast = skill.skillCastLog[$focusedSkillCast.cast];
            totalDamage = skillCast.hits.map((hit) => hit.damage).reduce((a, b) => a + b, 0);
        }
    });

    $effect(() => {
        if ($focusedSkillCast.skillId > 0 && player) {
            let groupedBuffs = [];
            let tempModInfo = { crit: 0, critDamage: 0, ba: 0, fa: 0 };
            let tempSupportBuffs = { buff: 0, brand: 0, identity: 0 };
            for (const [, hit] of skillCast.hits.entries()) {
                getSkillCastBuffs(hit.damage, hit.buffedBy, hit.debuffedBy, encounterDamageStats, tempSupportBuffs);

                groupedBuffs.push(
                    getSkillCastBuffs(
                        hit.damage,
                        hit.buffedBy,
                        hit.debuffedBy,
                        encounterDamageStats,
                        { buff: 0, brand: 0, identity: 0 },
                        player.classId,
                        $buffType,
                        $settings.buffs.default
                    )
                );
                if (hit.crit) {
                    tempModInfo.crit++;
                    tempModInfo.critDamage += hit.damage;
                }
                if (hit.backAttack) {
                    tempModInfo.ba++;
                }
                if (hit.frontAttack) {
                    tempModInfo.fa++;
                }
            }

            allGroupedBuffs = groupedBuffs;
            modInfo = tempModInfo;
            supportBuffs = tempSupportBuffs;
        }
    });

    function getHighestDamageCastIndex(): number {
        let highestDamage = 0;
        let highestIndex = 0;
        for (const [i, cast] of skill.skillCastLog.entries()) {
            const totalDamage = cast.hits.map((hit) => hit.damage).reduce((a, b) => a + b, 0);
            if (totalDamage > highestDamage) {
                highestDamage = totalDamage;
                highestIndex = i;
            }
        }
        return highestIndex;
    }
</script>

<div class="mt-2 h-[400px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);"></div>

<div class="mb-4 mt-2 min-h-[30rem]">
    <div class="flex justify-start text-lg font-medium">
        <div use:menuTooltip={{ content: "Details about a skill casted during the raid" }}>Skill Cast Details</div>
    </div>
    {#if $focusedSkillCast.skillId === 0}
        <div>Click on a skill cast to show details.</div>
    {:else if skill}
        <div class="px-1 pb-2">
            <div class="flex items-center pb-2 pt-1">
                <button
                    use:tooltip={{ content: "Go to highest damage cast." }}
                    class="bg-accent-500 hover:bg-accent-800 mr-4 rounded-md p-1 text-sm"
                    onclick={() => {
                        $focusedSkillCast.cast = getHighestDamageCastIndex();
                    }}>
                    Find Max Cast
                </button>
                <button
                    use:tooltip={{ content: "Previous Cast" }}
                    aria-label="Previous Cast"
                    class="pr-1"
                    onclick={() => {
                        if ($focusedSkillCast.cast > 0) {
                            $focusedSkillCast.cast -= 1;
                        }
                    }}>
                    <svg
                        class="size-7 fill-gray-400 {$focusedSkillCast.cast === 0
                            ? 'opacity-20'
                            : 'hover:fill-accent-800'}"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 96 960 960">
                        <path d="m560.5 837-262-262 262-262 65 65.5L429 575l196.5 196.5-65 65.5Z" />
                    </svg>
                </button>
                <button
                    use:tooltip={{ content: "Next Cast" }}
                    aria-label="Next Cast"
                    class="px-1"
                    onclick={() => {
                        if ($focusedSkillCast.cast < skill.skillCastLog.length - 1) $focusedSkillCast.cast += 1;
                    }}>
                    <svg
                        class="size-7 fill-gray-400 {$focusedSkillCast.cast === skill.skillCastLog.length - 1
                            ? 'opacity-20'
                            : 'hover:fill-accent-800'}"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 96 960 960">
                        <path d="m375.5 837-65-65.5L507 575 310.5 378.5l65-65.5 262 262-262 262Z" />
                    </svg>
                </button>
            </div>
            <div class="pb-2 font-medium">
                <span use:tooltip={{ content: "Skill duration, from cast to last tick of damage" }}>
                    {formatDurationFromMs(skillCast.timestamp)}-{formatDurationFromMs(skillCast.last)} ({round(
                        (skillCast.last - skillCast.timestamp) / 1000
                    )}s)
                </span>
            </div>
            <div class="flex items-center space-x-2">
                <img class="size-7 rounded-sm" src={$skillIcon.path + getSkillIcon(skill.icon)} alt={skill.name} />
                <div class="font-semibold">
                    {skill.name} #{$focusedSkillCast.cast + 1}
                </div>
            </div>
            {#if skillCast.hits.length === 0}
                <div class="py-2">No Hits</div>
            {:else}
                <div class="py-2">
                    Total Damage: <span class="font-semibold" use:tooltip={{ content: totalDamage.toLocaleString() }}
                        >{abbreviateNumber(totalDamage)}</span>
                </div>
                <div>
                    Crit: <span class="font-semibold">{round((modInfo.crit / skillCast.hits.length) * 100)}%</span>
                    | CDMG:
                    <span class="font-semibold"
                        >{round(totalDamage !== 0 ? (modInfo.critDamage / totalDamage) * 100 : 0)}%</span>
                    {#if modInfo.ba > 0}
                        | BA: <span class="font-semibold">{round((modInfo.ba / skillCast.hits.length) * 100)}%</span>
                    {/if}
                    {#if modInfo.fa > 0}
                        | FA: <span class="font-semibold">{round((modInfo.fa / skillCast.hits.length) * 100)}%</span>
                    {/if}
                </div>
                <div class="">
                    Buff: <span class="font-semibold"
                        >{round(totalDamage !== 0 ? (supportBuffs.buff / totalDamage) * 100 : 0)}%</span>
                    | Brand:
                    <span class="font-semibold"
                        >{round(totalDamage !== 0 ? (supportBuffs.brand / totalDamage) * 100 : 0)}%</span>
                    | Identity:
                    <span class="font-semibold"
                        >{round(totalDamage !== 0 ? (supportBuffs.identity / totalDamage) * 100 : 0)}%</span>
                </div>
                <table class="mt-2 w-[60rem] table-fixed">
                    <thead>
                        <tr>
                            <td class="w-12 font-semibold" use:tooltip={{ content: "Each damage tick" }}>Hits</td>
                            <td class="w-16 font-semibold" use:tooltip={{ content: "Time since previous damage tick" }}
                                >Ticks</td>
                            <td class="w-12 font-semibold" use:tooltip={{ content: "Hit modifiers, e.g. Crit, BA, FA" }}
                                >Mods</td>
                            <td class="w-16 font-semibold" use:tooltip={{ content: "Hit damage" }}>DMG</td>
                            <td class="w-full font-semibold">
                                <span use:tooltip={{ content: "Party Buffs" }}>
                                    <button
                                        class={$buffType === "party" ? "text-accent-500" : "hover:text-accent-500"}
                                        onclick={() => {
                                            $buffType = "party";
                                        }}>
                                        Party
                                    </button>
                                </span>
                                |
                                <span use:tooltip={{ content: "Self Buffs, including Relic Sets" }}>
                                    <button
                                        class={$buffType === "self" ? "text-accent-500" : "hover:text-accent-500"}
                                        onclick={() => {
                                            $buffType = "self";
                                        }}>
                                        Self
                                    </button>
                                </span>
                                |
                                <span use:tooltip={{ content: "All other buffs, e.g. Darks, Atros, etc." }}>
                                    <button
                                        class={$buffType === "misc" ? "text-accent-500" : "hover:text-accent-500"}
                                        onclick={() => {
                                            $buffType = "misc";
                                        }}>
                                        Misc.
                                    </button>
                                </span>
                                Buffs
                            </td>
                        </tr>
                    </thead>
                    <tbody>
                        {#each skillCast.hits as hit, i (i)}
                            <tr>
                                <td class="h-7 font-mono">#{i + 1}</td>
                                {#if i === 0}
                                    <td class="font-mono">
                                        <span use:tooltip={{ content: `${formatDurationFromMs(hit.timestamp)}s` }}>
                                            +{round((hit.timestamp - skillCast.timestamp) / 1000)}s
                                        </span>
                                    </td>
                                {:else}
                                    <td class="font-mono">
                                        <span use:tooltip={{ content: `${formatDurationFromMs(hit.timestamp)}s` }}>
                                            +{round((hit.timestamp - skillCast.hits[i - 1].timestamp) / 1000)}s
                                        </span>
                                    </td>
                                {/if}
                                <td class="font-mono">
                                    {#if hit.crit}
                                        <span use:tooltip={{ content: "Critical Hit" }}>C</span>
                                    {/if}
                                    {#if hit.backAttack}
                                        <span use:tooltip={{ content: "Back Attack" }}>B</span>
                                    {/if}
                                    {#if hit.frontAttack}
                                        <span use:tooltip={{ content: "Front Attack" }}>F</span>
                                    {/if}
                                    {#if !hit.crit && !hit.backAttack && !hit.frontAttack}
                                        -
                                    {/if}
                                </td>
                                <td class="font-mono">
                                    <span use:tooltip={{ content: hit.damage.toLocaleString() }}>
                                        {abbreviateNumber(hit.damage)}
                                    </span>
                                </td>
                                <td>
                                    <div class="flex">
                                        {#if allGroupedBuffs[i].size > 0}
                                            {#each allGroupedBuffs[i] as [_, groupedBuffs]}
                                                {#each groupedBuffs as buff}
                                                    <BuffTooltip synergy={buff.statusEffect} size={"size-6"} />
                                                {/each}
                                            {/each}
                                        {:else}
                                            -
                                        {/if}
                                    </div>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    {/if}
</div>
