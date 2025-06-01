/* eslint-disable @typescript-eslint/no-explicit-any */
import { bossHpMap } from "$lib/constants/encounters";
import type { EncounterState } from "$lib/encounter.svelte";
import { type SkillHit, type StatusEffect } from "$lib/types";
import { defaultOptions } from "../charts";
import {
  addBardBubbles,
  groupedSynergiesAdd,
  hyperAwakeningIds,
  isPartySynergy,
  isSupportBuff,
  makeSupportBuffKey,
  supportSkills
} from "./buffs";
import { abbreviateNumber, customRound, getSkillIcon, timestampToMinutesAndSeconds } from "$lib/utils";

const partyRegex = /^Party (\d)$/;
const partyColors = ["#54AD56", "#C3B1E1", "#A7C7E7", "#FDFD96"];
const bardSerenadeOfAmplification = "Serenade of Amplification";
const artistBlessingOfTheSun = "Blessing of the Sun";

class SupportSynergyDataPoint {
  totalDamage: number;
  buffs: Map<string, Array<SupportBuffPoint>>;

  constructor() {
    this.totalDamage = 0;
    this.buffs = new Map<string, Array<SupportBuffPoint>>();
  }

  add(hit: SkillHit, key: string, id: number, effect: StatusEffect) {
    const buffPoints = this.buffs.get(key) || [];
    const index = buffPoints.findIndex((b) => b.id === id);
    let buffPoint: SupportBuffPoint = {
      id: id,
      bonus: 0,
      buffedDamage: 0,
      totalDamage: hit.damage,
      icon: effect.source.icon,
      sourceIcon: effect.source.skill?.icon
    };
    if (index !== -1) {
      buffPoint = buffPoints[index];
      buffPoint.totalDamage += hit.damage;
    }
    addBardBubbles(key, buffPoint, effect);
    if (hit.buffedBy.includes(id) || hit.debuffedBy.includes(id)) {
      buffPoint.buffedDamage += hit.damage;
    }
    if (index !== -1) {
      buffPoints[index] = buffPoint;
    } else {
      buffPoints.push(buffPoint);
    }
    this.buffs.set(key, buffPoints);
  }

  merge(other: SupportSynergyDataPoint) {
    const keys = new Set([...this.buffs.keys(), ...other.buffs.keys()]);
    let totalDamage = 0;
    for (const key of keys) {
      const buffPointsA = this.buffs.get(key) || [];
      const buffPointsB = other.buffs.get(key) || [];
      const buffPointsIds = new Set([...buffPointsA.map((b) => b.id), ...buffPointsB.map((b) => b.id)]);
      const result = new Array<SupportBuffPoint>();
      for (const buffPointId of buffPointsIds) {
        const a = buffPointsA.find((bp) => bp.id === buffPointId) || ({} as SupportBuffPoint);
        const b = buffPointsB.find((bp) => bp.id === buffPointId) || ({} as SupportBuffPoint);
        totalDamage = (a.totalDamage || 0) + (b.totalDamage || 0);
        result.push({
          id: a.id || b.id,
          bonus: a.bonus || b.bonus,
          totalDamage: totalDamage,
          buffedDamage: (a.buffedDamage || 0) + (b.buffedDamage || 0),
          icon: a.icon || b.icon,
          sourceIcon: a.sourceIcon || b.sourceIcon
        });
      }
      this.buffs.set(key, result);
    }
    this.totalDamage = totalDamage;
  }
}

interface SupportBuffPoint {
  id: number;
  bonus?: number;
  buffedDamage: number;
  totalDamage: number;
  icon: string;
  sourceIcon?: string;
}

function addStatusEffect(map: Map<string, Map<number, StatusEffect>>, effect: [string, StatusEffect]) {
  const [id, statusEffect] = effect;
  if (
    !isPartySynergy(statusEffect) ||
    !isSupportBuff(statusEffect) ||
    statusEffect.source.name === bardSerenadeOfAmplification
  ) {
    return;
  }
  const key = makeSupportBuffKey(statusEffect);
  if (statusEffect.source.name === artistBlessingOfTheSun) {
    return;
  }
  const idNumber = Number(id);
  groupedSynergiesAdd(map, key, idNumber, statusEffect, undefined, true);
}

export function getSupportSynergiesOverTime(
  state: EncounterState,
  fightStartMs: number,
  fightEndMs: number,
  intervalMs: number
) {
  const groupedSupportSynergies = new Map<string, Map<number, StatusEffect>>();
  Object.entries(state.encounter!.encounterDamageStats.buffs).forEach((effect) =>
    addStatusEffect(groupedSupportSynergies, effect)
  );
  Object.entries(state.encounter!.encounterDamageStats.debuffs).forEach((effect) =>
    addStatusEffect(groupedSupportSynergies, effect)
  );

  const partyBuffs = new Array<Map<number, SupportSynergyDataPoint>>();
  const partyGroupedSupportSynergies = new Map<string, Set<string>>();
  if (groupedSupportSynergies.size > 0 && state.parties.length >= 1) {
    state.parties.forEach((party, partyId) => {
      partyGroupedSupportSynergies.set(partyId.toString(), new Set<string>());
      const partySyns = new Set<string>();
      for (const player of party) {
        groupedSupportSynergies.forEach((synergies, key) => {
          synergies.forEach((_, id) => {
            if (player.damageStats.buffedBy[id] || player.damageStats.debuffedBy[id]) {
              partySyns.add(key);
            }
          });
        });
      }
      partyGroupedSupportSynergies.set(partyId.toString(), new Set([...partySyns].sort()));
    });

    state.parties.forEach((party, partyId) => {
      partyBuffs[partyId] = new Map<number, SupportSynergyDataPoint>();

      partyGroupedSupportSynergies.get(partyId.toString())?.forEach((key) => {
        const buffs = groupedSupportSynergies.get(key);

        if (!buffs) {
          return;
        }

        let isHat = false;

        buffs.forEach((syn, id) => {
          if (supportSkills.haTechnique.includes(id)) {
            isHat = true;
          }

          for (const player of party) {
            const skills = player.skills;
            for (const skill of Object.values(skills)) {
              if (!isHat && hyperAwakeningIds.has(skill.id)) {
                continue;
              }
              for (const skillCast of skill.skillCastLog) {
                for (const hit of skillCast.hits) {
                  const synergyPoint = partyBuffs[partyId].get(hit.timestamp) || new SupportSynergyDataPoint();
                  synergyPoint.add(hit, key, id, syn);
                  partyBuffs[partyId].set(hit.timestamp, synergyPoint);
                }
              }
            }
          }
        });
      });
    });
  }

  const partySupportSynergyTimeline = new Array<Array<[number, SupportSynergyDataPoint]>>();
  for (const map of partyBuffs) {
    partySupportSynergyTimeline.push([...map.entries()].sort((a, b) => a[0] - b[0]));
  }
  const supportSynergiesOverTime = new Array<Array<[string, SupportSynergyDataPoint]>>();
  partySupportSynergyTimeline.map((partyTimeline, partyId) => {
    const synergyPoint = new SupportSynergyDataPoint();
    supportSynergiesOverTime[partyId] = [];
    for (let t = 0, index = 0; t <= fightEndMs - fightStartMs; t += intervalMs) {
      while (index < partyTimeline.length && partyTimeline[index][0] <= t) {
        synergyPoint.merge(partyTimeline[index][1]);
        index++;
      }
      const copy = new SupportSynergyDataPoint();
      copy.merge(synergyPoint);
      supportSynergiesOverTime[partyId].push([timestampToMinutesAndSeconds(t), copy]);
    }
  });

  return supportSynergiesOverTime.map((data, partyId) => {
    return {
      name: `Party ${partyId + 1}`,
      color: partyColors[partyId],
      type: "line",
      data: data,
      showSymbol: false,
      smooth: 0.1,
      markPoint: {},
      yAxisIndex: 1
    };
  });
}

export function getSupportSynergiesOverTimeChart(
  legendNames: string[],
  chartBuffs: any[],
  buffSubstring: string,
  chartBosses: any[]
) {
  const buffSeries = chartBuffs.map((chartOptions) => {
    return {
      ...chartOptions,
      data: chartOptions.data.map((dataPoint: [string, SupportSynergyDataPoint]) => {
        const time = dataPoint[0];
        const synergies: SupportSynergyDataPoint = dataPoint[1];
        for (const [key, buffs] of synergies.buffs) {
          if (key.includes(buffSubstring)) {
            const buffedDamage = buffs.reduce((sum, buff) => buff.buffedDamage + sum, 0);
            return [time, (buffedDamage / synergies.totalDamage) * 100];
          }
        }
        return [time, 0];
      })
    };
  });

  return {
    ...defaultOptions,
    legend: {
      data: legendNames,
      textStyle: {
        color: "white"
      },
      type: "scroll",
      width: "90%",
      pageIconInactiveColor: "#313131",
      pageIconColor: "#aaa",
      pageTextStyle: {
        color: "#aaa"
      },
      selector: true
    },
    tooltip: {
      trigger: "axis",
      formatter: function (params: any[]) {
        const time = params[0].name;
        const bossTooltips = new Array<string>();
        const buffToolTips: string[] = [];
        params.forEach((param) => {
          const partyLabel: string = param.seriesName;
          let value = param.value[1];
          const partyNumber = partyRegex.exec(partyLabel);
          if (partyNumber) {
            const partyId = parseInt(partyNumber[1]) - 1;
            const synergies: SupportSynergyDataPoint = chartBuffs[partyId].data[param.dataIndex][1];
            let buffBreakdown = "";
            synergies.buffs.forEach((buffPoint, key) => {
              if (key.includes(buffSubstring)) {
                for (const buff of buffPoint) {
                  const buffedDamage = customRound((buff.buffedDamage / buff.totalDamage) * 100);
                  if (buffedDamage === "0.0") {
                    continue;
                  }
                  let buffed = `<div class="min-w-[4.5rem] flex items-center">`;
                  if (buff.sourceIcon) {
                    buffed += `<img src=${getSkillIcon(buff.sourceIcon)} alt="buff_source_icon" class="size-5 rounded mr-1"/>`;
                    if (buff.bonus) {
                      buffed += `[${buff.bonus}%] `;
                    }
                  } else {
                    buffed += `<img src=${getSkillIcon(buff.icon)} alt="buff_icon" class="size-5 rounded mr-1"/>`;
                  }
                  buffBreakdown += `${buffed}${buffedDamage}%</div>`;
                }
              }
            });
            const finalBreakdown = `<div class="flex gap-1">${buffBreakdown}</div>`;
            const finalValue = customRound(value);
            const finalLabel =
              `<span class="inline-block mr-1 rounded-full size-2.5" style="background-color:${param.color}"></span>` +
              partyLabel;
            buffToolTips.push(`<div class="flex gap-3 w-full justify-between">
                                <div class="flex gap-3"><div class="w-14">${finalLabel}</div> ${finalBreakdown}</div> <div class="font-bold">${finalValue}%</div>
                            </div>`);
          } else {
            value += "%";
            if (Object.hasOwn(bossHpMap, partyLabel)) {
              const bossMaxHpBars = bossHpMap[partyLabel];
              const bossHpBars = Math.floor(bossMaxHpBars * (parseFloat(value) / 100));
              value = `${bossHpBars}x (${value})`;
            }
            bossTooltips.push(
              `<div class="flex"><div class="pr-1 font-bold">${partyLabel}</div><div class="font-bold">${value}</div></div>`
            );
          }
        });
        return `<div>${time}</div><div>${bossTooltips.join("")}${buffToolTips.join("")}</div>`;
      }
    },
    xAxis: {
      type: "category",
      splitLine: {
        show: false
      },
      data: chartBuffs[0].data.map((d: [number, SupportSynergyDataPoint]) => d[0]),
      boundaryGap: false,
      axisLabel: {
        color: "white"
      }
    },
    yAxis: [
      {
        type: "value",
        splitLine: {
          show: true,
          lineStyle: {
            color: "#333"
          }
        },
        axisLabel: {
          color: "white",
          formatter: function (value: number) {
            return abbreviateNumber(value);
          }
        }
      },
      {
        type: "value",
        splitLine: {
          show: false
        },
        axisLabel: {
          color: "white",
          formatter: "{value}%"
        }
      }
    ],
    series: [...buffSeries, ...chartBosses]
  };
}
