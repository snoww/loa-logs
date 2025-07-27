/* eslint-disable @typescript-eslint/no-explicit-any */
import { focusedCast, settings } from "$lib/stores.svelte";
import {
  BossHpLog,
  EntityType,
  MiniSkill,
  OpenerSkill,
  type DamageStats,
  type EncounterDamageStats,
  type Entity,
  type Skill,
  type SkillCast,
  type SkillChartModInfo,
  type SkillChartSupportDamage
} from "$lib/types";
import Heap from "heap-js";
import BTree from "sorted-btree";
import { defaultOptions } from "../charts";
import { getFormattedBuffString, getSkillCastBuffs, getSkillCastSupportBuffs } from "./buffs";
import { bossHpMap } from "$lib/constants/encounters";
import {
  abbreviateNumber,
  customRound,
  getSkillIcon,
  isNameValid,
  normalize,
  resampleData,
  timestampToMinutesAndSeconds,
  timeToSeconds
} from "$lib/utils";

const colors = ["#cc338b", "#A020F0", "#FFA500", "#800000"];

export function getLegendNames(players: Entity[]) {
  if (!settings.app.general.showNames) {
    return players.map((player, i) => `${player.class} #${i + 1}`);
  }

  return players.map((player) => (isNameValid(player.name) ? player.name : player.class));
}

export function getDeathTimes(chartablePlayers: Entity[], legendNames: string[], fightStart: number) {
  const deathTimes: Record<string, number> = {};
  chartablePlayers.forEach((player, i) => {
    if (player.isDead) {
      deathTimes[legendNames[i]!] = (player.damageStats.deathTime - fightStart) / 1000;
    }
  });
  return deathTimes;
}

export function getPlayerSeries(
  chartablePlayers: Entity[],
  legendNames: string[],
  fightStart: number,
  field: keyof DamageStats
) {
  return chartablePlayers
    .filter((e) => e.entityType === EntityType.PLAYER)
    .map((player: Entity, i: number) => {
      let markPoint = {};
      if (player.isDead) {
        const rounded = Math.ceil((player.damageStats.deathTime - fightStart) / 1000 / 5) * 5;
        const index =
          field === "dpsRolling10sAvg"
            ? Math.ceil((player.damageStats.deathTime - fightStart) / 1000)
            : Math.floor(rounded / 5);

        markPoint = {
          data: [
            {
              name: "Death",
              value: "💀",
              coord: [index, player.damageStats[field][index]]
            }
          ]
        };
      }

      return {
        name: legendNames[i],
        color: settings.classColors[player.class] || "gray",
        type: "line",
        data: player.damageStats[field],
        showSymbol: false,
        smooth: 0.1,
        markPoint: markPoint,
        yAxisIndex: 0
      };
    });
}

export function getAveragePlayerSeries(chartablePlayers: Entity[], legendNames: string[], fightStart: number) {
  return getPlayerSeries(chartablePlayers, legendNames, fightStart, "dpsAverage");
}

export function getRollingPlayerSeries(chartablePlayers: Entity[], legendNames: string[], fightStart: number) {
  return getPlayerSeries(chartablePlayers, legendNames, fightStart, "dpsRolling10sAvg");
}

export function getAverageDpsChart(
  chartablePlayers: Entity[],
  legendNames: string[],
  chartPlayers: any[],
  chartBosses: any[],
  deathTimes: { [key: string]: number }
) {
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
      formatter(params: any[]) {
        const time = params[0].name;
        let tooltipStr = `<div>${time}</div><div style="min-width: 10rem">`;
        const bossTooltips: string[] = [];
        const tree = new BTree(undefined, (a, b) => b - a);
        const length = Object.keys(chartablePlayers).length;
        const totalDps = { value: 0 };
        params.forEach((param) => generateTooltip(param, bossTooltips, totalDps, tree, deathTimes, time, length));
        const totalDpsString = `<div style="display:flex; justify-content: space-between;font-weight: 600;"><div style="padding-right: 1rem">Total DPS</div><div>${abbreviateNumber(totalDps.value, 2)}</div></div>`;
        tooltipStr += bossTooltips.join("") + totalDpsString + tree.valuesArray().join("") + "</div>";
        return tooltipStr;
      }
    },
    xAxis: {
      type: "category",
      splitLine: {
        show: false
      },
      data: Array.from({ length: chartablePlayers[0].damageStats.dpsAverage.length }, (_, i) =>
        secondsToMinutesAndSeconds(i * 5)
      ),
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
          formatter(value: number) {
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
    series: [...chartPlayers, ...chartBosses]
  };
}

export function getRollingDpsChart(
  chartablePlayers: Entity[],
  legendNames: string[],
  chartPlayers: any[],
  chartBosses: any[],
  deathTimes: { [key: string]: number }
) {
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
      formatter(params: any[]) {
        const time = params[0].name;
        let tooltipStr = `<div>${time}</div><div style="min-width: 10rem">`;
        const bossTooltips: string[] = [];
        const length = Object.keys(chartablePlayers).length;
        const tree = new BTree(undefined, (a, b) => b - a);
        const totalDps = { value: 0 };
        params.forEach((param) => generateTooltip(param, bossTooltips, totalDps, tree, deathTimes, time, length));
        const totalDpsString = `<div style="display:flex; justify-content: space-between;font-weight: 600;"><div style="padding-right: 1rem">Total DPS</div><div>${abbreviateNumber(totalDps.value, 2)}</div></div>`;
        tooltipStr += bossTooltips.join("") + totalDpsString + tree.valuesArray().join("") + "</div>";
        return tooltipStr;
      }
    },
    xAxis: {
      type: "category",
      splitLine: {
        show: false
      },
      data: Array.from({ length: chartablePlayers[0].damageStats.dpsRolling10sAvg.length }, (_, i) =>
        secondsToMinutesAndSeconds(i)
      ),
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
          formatter(value: number) {
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
    series: [...chartPlayers, ...chartBosses]
  };
}

export function getBossHpSeries(bosses: [string, BossHpLog[]][], legendNames: string[], len: number, interval: number) {
  return bosses
    .filter((e) => e[1].length > 1)
    .sort((a, b) => {
      return a[1][0].time - b[1][0].time;
    })
    .map((entry, i) => {
      legendNames.push(entry[0]);
      const max = Math.max(...entry[1].slice(0, 5).map((e) => e.p));
      // if boss starts off with more than 100% hp (e.g. mordum g3 hm 0x phase), we normalize the hp percent to 0-1
      // boss hp bar x will be inaccurate
      const log = max > 1 ? entry[1].map((e) => new BossHpLog(e.time, e.hp, normalize(e.p, 0, max))) : entry[1];
      const resample = resampleData(log, interval, len);
      const data = resample.map((e) => {
        return [secondsToMinutesAndSeconds(e.time), customRound(e.p * 100, 1)];
      });
      return {
        name: entry[0],
        color: colors[i % colors.length],
        type: "line",
        data: data,
        showSymbol: false,
        smooth: 0.1,
        yAxisIndex: 1,
        lineStyle: {
          type: "dotted"
        }
      };
    });
}

export function getDetailedSkillLogChart(
  player: Entity,
  lastCombatPacket: number,
  fightStart: number,
  encounterDamageStats: EncounterDamageStats
) {
  const sortedSkills = Object.values(player.skills)
    .filter((skill) => skill.skillCastLog.length > 0)
    .sort((a, b) => a.totalDamage - b.totalDamage);
  const skills = sortedSkills.map((skill) => skill.name);

  return {
    ...defaultOptions,
    grid: {
      left: "2%",
      right: "5%",
      bottom: "18%",
      top: "10%",
      containLabel: true
    },
    dataZoom: [
      {
        type: "slider",
        fillerColor: "rgba(80,80,80,.5)",
        borderColor: "rgba(80,80,80,.5)",
        handleStyle: {
          color: "rgba(80,80,80,.5)"
        },
        moveHandleStyle: {
          color: "rgba(136,136,136)"
        }
      },
      {
        type: "inside",
        xAxisIndex: [0],
        throttle: 50
      },
      {
        type: "inside",
        yAxisIndex: [0],
        throttle: 50,
        zoomOnMouseWheel: false
      }
    ],
    tooltip: {
      trigger: "item",
      triggerOn: "click",
      confine: true,
      transitionDuration: 0,
      extraCssText: "pointer-events: auto !important; padding: 0"
    } as any,
    legend: {
      data: [...skills].reverse(),
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
      itemWidth: 20,
      itemHeight: 20,
      selector: true
    },
    xAxis: {
      type: "category",
      splitLine: {
        show: false
      },
      data: Array.from(
        {
          length: (lastCombatPacket - fightStart) / 1000
        },
        (_, i) => secondsToMinutesAndSeconds(i)
      ),
      boundaryGap: false,
      axisLabel: {
        color: "white"
      }
    },
    yAxis: {
      type: "category",
      splitLine: {
        show: true,
        lineStyle: {
          color: "#333"
        }
      },
      axisLabel: {
        show: false
      },
      data: skills.map((skill) => {
        return {
          value: skill
        };
      })
    },
    series: sortedSkills.map((skill) => {
      return {
        name: skill.name,
        type: "scatter",
        symbol: "image://" + getSkillIcon(skill.icon),
        symbolSize: [20, 20],
        symbolKeepAspect: true,
        data: skill.skillCastLog.map((cast) => [
          timestampToMinutesAndSeconds(cast.timestamp),
          skill.name,
          cast,
          skill.icon
        ]),
        tooltip: {
          formatter: function (param: any) {
            focusedCast.skillId = skill.id;
            focusedCast.cast = param.dataIndex;
            let output = "<div class='overflow-y-auto max-h-56 pt-1 pb-2 px-2' style='scrollbar-width: thin;'>";
            output += `
                        <div class='flex justify-between'>
                        <div class='font-semibold mb-1'>${param.name}-${timestampToMinutesAndSeconds(param.value[2].last)} (${customRound((param.value[2].last - param.value[2].timestamp) / 1000)}s)</div>
                        <div class='tracking-tight text-sm'>Scroll Down for Details</div>
                        </div>
                        `;
            output += "<div>";
            output += "<div class='flex space-x-1'>";
            output += `<img class="size-5 rounded-xs" src='${getSkillIcon(param.value[3])}' alt='${param.seriesName}' />`;
            output += `<div class='font-semibold'>${param.seriesName + " #" + (param.dataIndex + 1)}</div>`;
            output += "</div>";
            if (param.value[2].hits.length > 0) {
              output += skillCastBreakdownTooltip(player, param.value[2], encounterDamageStats);
            }
            output += "</div>";
            output += "</div>";
            return output;
          },
          borderColor: "#fff",
          extraCssText: "pointer-events: auto !important; padding: 0"
        }
      };
    })
  };
}

export function getBasicSkillLogChart(player: Entity, lastCombatPacket: number, fightStart: number) {
  const sortedSkills = Object.values(player.skills)
    .filter((skill) => skill.castLog.length > 0)
    .sort((a, b) => a.totalDamage - b.totalDamage);
  const skills = sortedSkills.map((skill) => skill.name);

  return {
    ...defaultOptions,
    grid: {
      left: "2%",
      right: "5%",
      bottom: "18%",
      top: "10%",
      containLabel: true
    },
    dataZoom: [
      {
        type: "slider",
        fillerColor: "rgba(80,80,80,.5)",
        borderColor: "rgba(80,80,80,.5)",
        handleStyle: {
          color: "rgba(80,80,80,.5)"
        },
        moveHandleStyle: {
          color: "rgba(136,136,136)"
        }
      },
      {
        type: "inside",
        xAxisIndex: [0],
        throttle: 50
      },
      {
        type: "inside",
        yAxisIndex: [0],
        throttle: 50,
        zoomOnMouseWheel: false
      }
    ],
    tooltip: {
      trigger: "axis",
      formatter(params: any[]) {
        let output = `<span style="font-weight: 600">${params[0].name}</span>`;
        params.forEach((p) => {
          output += `<br/>${p.seriesName}`;
        });

        return output;
      }
    } as any,
    legend: {
      data: [...skills].reverse(),
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
      itemWidth: 20,
      itemHeight: 20,
      selector: true
    },
    xAxis: {
      type: "category",
      splitLine: {
        show: false
      },
      data: Array.from(
        {
          length: (lastCombatPacket - fightStart) / 1000
        },
        (_, i) => secondsToMinutesAndSeconds(i)
      ),
      boundaryGap: false,
      axisLabel: {
        color: "white"
      }
    },
    yAxis: {
      type: "category",
      splitLine: {
        show: true,
        lineStyle: {
          color: "#333"
        }
      },
      axisLabel: {
        show: false
      }
    },
    series: sortedSkills.map((skill) => {
      return {
        name: skill.name,
        type: "scatter",
        symbol: "image://" + getSkillIcon(skill.icon),
        symbolSize: [20, 20],
        symbolKeepAspect: true,
        data: skill.castLog.map((cast) => [secondsToMinutesAndSeconds(cast / 1000), skill.name])
      };
    })
  };
}

export function getOpenerSkills(skills: Skill[], x: number): OpenerSkill[] {
  const comparator = (a: MiniSkill, b: MiniSkill) => a.castLog[0] - b.castLog[0];

  const minHeap = new Heap(comparator);
  for (const skill of skills) {
    minHeap.push({
      name: skill.name,
      icon: skill.icon,
      castLog: skill.castLog.slice()
    });
  }

  const result: OpenerSkill[] = [];

  while (result.length < x && minHeap.size() > 0) {
    // Pop the object with the smallest cast_log[0]
    const skill = minHeap.pop();
    if (skill === undefined) {
      break;
    }

    // Push the object's name to the result array
    result.push(new OpenerSkill(skill.name, skill.icon));

    // Remove the first element from the object's cast_log
    skill.castLog.shift();

    // If the object's cast_log still has elements, re-insert it into the heap
    if (skill.castLog.length > 0) {
      minHeap.push(skill);
    }
  }

  // Return the first x names
  return result.slice(0, x);
}

function generateTooltip(
  param: any,
  bossTooltips: string[],
  totalDps: { value: number },
  tree: BTree,
  deathTimes: { [key: string]: number },
  time: string,
  chartablePlayersLength: number
) {
  let label = param.seriesName;
  let value = param.value;
  if (param.seriesIndex >= chartablePlayersLength) {
    value = value[1] + "%";
    if (Object.hasOwn(bossHpMap, label)) {
      const bossMaxHpBars = bossHpMap[label];
      const bossHpBars = Math.floor(bossMaxHpBars * (parseFloat(value) / 100));
      value = bossHpBars + "x (" + value + ")";
    }
    bossTooltips.push(
      `<div style="display:flex; justify-content: space-between;"><div style="padding-right: 1rem;font-weight: 600;">${label}</div><div style="font-weight: 600;">${value}</div></div>`
    );
  } else {
    if (deathTimes[label] && deathTimes[label] < timeToSeconds(time)) {
      label = "💀 " + label;
    }
    const dps = Number(value);
    totalDps.value += dps;
    value = abbreviateNumber(value);
    label =
      `<span style="display:inline-block;margin-right:5px;border-radius:10px;width:10px;height:10px;background-color:${param.color}"></span>` +
      label;
    tree.set(
      dps,
      `<div style="display:flex; justify-content: space-between;"><div style="padding-right: 1rem">${label}</div><div style="">${value}</div></div>`
    );
  }
}

function skillCastBreakdownTooltip(
  player: Entity,
  skillCast: SkillCast,
  encounterDamageStats: EncounterDamageStats
): string {
  const totalDamage = skillCast.hits.map((hit) => hit.damage).reduce((a, b) => a + b, 0);
  let output = "<div class='flex flex-col'>";
  output += `<div>Total Damage: <span class='font-semibold'>${abbreviateNumber(totalDamage)}</span></div>`;
  output += `<div>`;
  let table = `
    <table class='table-fixed'>
        <thead>
            <tr>
                <td class='w-10 font-semibold'>Hits</td>
                <td class='w-14 font-semibold'>Mods</td>
                <td class='w-16 font-semibold'>DMG</td>
                <td class='w-60 font-semibold overflow-auto'>Buffs</td>
            </tr>
        </thead>
        <tbody>
    `;
  const totalSupBuffs: SkillChartSupportDamage = { buff: 0, brand: 0, identity: 0 };
  const modInfo: SkillChartModInfo = { crit: 0, critDamage: 0, ba: 0, fa: 0 };
  for (const [i, hit] of skillCast.hits.entries()) {
    const groupedBuffs = getSkillCastBuffs(hit, encounterDamageStats, player);
    const supportBuffs = getSkillCastSupportBuffs(hit, encounterDamageStats);
    totalSupBuffs.buff += supportBuffs.buff;
    totalSupBuffs.brand += supportBuffs.brand;
    totalSupBuffs.identity += supportBuffs.identity;

    const buffString = getFormattedBuffString(groupedBuffs);

    table += "<tr>";
    table += `<td class="font-mono">#${i + 1}</td>`;
    let mods = "";
    if (hit.crit) {
      mods += "C ";
      modInfo.crit++;
      modInfo.critDamage += hit.damage;
    }
    if (hit.backAttack) {
      mods += "B ";
      modInfo.ba++;
    }
    if (hit.frontAttack) {
      mods += "F ";
      modInfo.fa++;
    }
    table += `<td class="font-mono">${mods.trim() ? mods : "-"}</td>`;
    table += `<td class="font-mono">${abbreviateNumber(hit.damage)}</td>`;
    table += `<td>${buffString}</td>`;
    table += "</tr>";
  }
  table += "</tbody></table>";
  output += `<div>
    Crit: <span class='font-semibold'>${customRound((modInfo.crit / skillCast.hits.length) * 100)}%</span>
    | CDMG: <span class='font-semibold'>${totalDamage !== 0 ? customRound((modInfo.critDamage / totalDamage) * 100) : 0}%</span>`;
  if (modInfo.ba > 0) {
    output += ` | BA: <span class='font-semibold'>${customRound((modInfo.ba / skillCast.hits.length) * 100)}%</span>`;
  }
  if (modInfo.fa > 0) {
    output += ` | FA: <span class='font-semibold'>${customRound((modInfo.fa / skillCast.hits.length) * 100)}%</span>`;
  }
  output += "</div>";
  output += `<div>
    Buff: <span class='font-semibold'>${totalSupBuffs.buff > 0 ? customRound((totalSupBuffs.buff / totalDamage) * 100) : 0}%</span>`;
  output += ` | B: <span class='font-semibold'>${totalSupBuffs.brand > 0 ? customRound((totalSupBuffs.brand / totalDamage) * 100) : 0}%</span>`;
  output += ` | Iden: <span class='font-semibold'>${totalSupBuffs.identity > 0 ? customRound((totalSupBuffs.identity / totalDamage) * 100) : 0}%</span>
    </div>`;
  output += table;
  output += "</div></div>";

  return output;
}

function secondsToMinutesAndSeconds(seconds: number): string {
  seconds = Math.round(seconds);
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes.toString().padStart(1, "0")}:${remainingSeconds.toString().padStart(2, "0")}`;
}

export function timeStringToSeconds(time: string): number {
  const split = time.split(":");
  const minutes = +split[0];
  const seconds = +split[1];
  return minutes * 60 + seconds;
}
