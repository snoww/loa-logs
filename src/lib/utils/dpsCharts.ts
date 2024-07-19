/* eslint-disable @typescript-eslint/no-explicit-any */
import {
    EntityType,
    type Entity,
    BossHpLog,
    OpenerSkill,
    MiniSkill,
    type SkillCast,
    type EncounterDamageStats,
    type SkillHit,
    type SkillChartSupportDamage,
    type SkillChartModInfo
} from "$lib/types";
import Heap from "heap-js";
import { defaultOptions } from "./charts";
import {
    abbreviateNumber,
    formatDurationFromMs,
    formatDurationFromS,
    resampleData,
    round,
    round2,
    timeToSeconds
} from "./numbers";
import { getSkillIcon, isValidName } from "./strings";
import { bossHpMap } from "$lib/constants/bossHpBars";
import { classesMap } from "$lib/constants/classes";
import BTree from "sorted-btree";
import { getFormattedBuffString, getSkillCastBuffs } from "./buffs";
import { identity } from "lodash-es";
import { tooltip } from "./tooltip";
import { focusedSkillCast } from "./stores";

export function getLegendNames(chartablePlayers: Entity[], showNames: boolean) {
    if (!showNames) {
        const map: { [key: string]: number } = {};
        const count = chartablePlayers
            .filter((e) => e.entityType === EntityType.PLAYER)
            .map((e) => {
                return (map[e.class] = typeof map[e.class] === "undefined" ? 1 : map[e.class] + 1);
            });
        return chartablePlayers
            .filter((e) => e.entityType === EntityType.PLAYER)
            .map((e, i) => {
                if (map[e.class] === 1) {
                    return e.class;
                } else {
                    return e.class + " " + count[i];
                }
            });
    }
    return chartablePlayers
        .filter((e) => e.entityType === EntityType.PLAYER)
        .map((e) => (isValidName(e.name) ? e.name : e.class));
}

export function getDeathTimes(chartablePlayers: Entity[], legendNames: string[], fightStart: number) {
    const deathTimes: { [key: string]: number } = {};
    chartablePlayers.forEach((player, i) => {
        if (player.isDead) {
            deathTimes[legendNames[i]] = (player.damageStats.deathTime - fightStart) / 1000;
        }
    });
    return deathTimes;
}

const colors = ["#cc338b", "#A020F0", "#FFA500", "#800000"];

export function getBossHpSeries(bosses: [string, BossHpLog[]][], legendNames: string[], len: number, interval: number) {
    return bosses
        .filter((e) => e[1].length > 1)
        .sort((a, b) => {
            return a[1][0].time - b[1][0].time;
        })
        .map((entry, i) => {
            legendNames.push(entry[0]);
            const resample = resampleData(entry[1], interval, len);
            const data = resample.map((e) => {
                return [formatDurationFromS(e.time), round2(e.p * 100, 1)];
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

export function getAveragePlayerSeries(
    chartablePlayers: Entity[],
    legendNames: string[],
    fightStart: number,
    classColors: any
) {
    return chartablePlayers
        .filter((e) => e.entityType === EntityType.PLAYER)
        .map((player: Entity, i: number) => {
            let markPoint = {};
            if (player.isDead) {
                const rounded = Math.ceil((player.damageStats.deathTime - fightStart) / 1000 / 5) * 5;
                const index = Math.floor(rounded / 5);

                markPoint = {
                    data: [
                        {
                            name: "Death",
                            value: "💀",
                            coord: [index, player.damageStats.dpsAverage[index]]
                        }
                    ]
                };
            }

            let color = "gray";
            if (classColors[classesMap[player.classId]]) {
                color = classColors[classesMap[player.classId]].color;
            }

            return {
                name: legendNames[i],
                color: color,
                type: "line",
                data: player.damageStats.dpsAverage,
                showSymbol: false,
                smooth: 0.1,
                markPoint: markPoint,
                yAxisIndex: 0
            };
        });
}

export function getRollingPlayerSeries(
    chartablePlayers: Entity[],
    legendNames: string[],
    fightStart: number,
    classColors: any
) {
    return chartablePlayers
        .filter((e) => e.entityType === EntityType.PLAYER)
        .map((player, i) => {
            let markPoint = {};
            if (player.isDead) {
                const index = Math.ceil((player.damageStats.deathTime - fightStart) / 1000);
                markPoint = {
                    data: [
                        {
                            name: "Death",
                            value: "💀",
                            coord: [index, player.damageStats.dpsRolling10sAvg[index]]
                        }
                    ]
                };
            }

            let color = "gray";
            if (classColors[classesMap[player.classId]]) {
                color = classColors[classesMap[player.classId]].color;
            }

            return {
                name: legendNames[i],
                color: color,
                type: "line",
                data: player.damageStats.dpsRolling10sAvg,
                showSymbol: false,
                smooth: 0.1,
                markPoint: markPoint
            };
        });
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
            formatter: function (params: any[]) {
                const time = params[0].name;
                let tooltipStr = `<div>${time}</div><div style="min-width: 10rem">`;
                const bossTooltips: string[] = [];
                const tree = new BTree(undefined, (a, b) => b - a);
                const length = Object.keys(chartablePlayers).length;
                params.forEach((param) => generateTooltip(param, bossTooltips, tree, deathTimes, time, length));
                tooltipStr += bossTooltips.join("") + tree.valuesArray().join("") + "</div>";
                return tooltipStr;
            }
        },
        xAxis: {
            type: "category",
            splitLine: {
                show: false
            },
            data: Array.from({ length: chartablePlayers[0].damageStats.dpsAverage.length }, (_, i) =>
                formatDurationFromS(i * 5)
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
            formatter: function (params: any[]) {
                const time = params[0].name;
                let tooltipStr = `<div>${time}</div><div style="min-width: 10rem">`;
                const bossTooltips: string[] = [];
                const length = Object.keys(chartablePlayers).length;
                const tree = new BTree(undefined, (a, b) => b - a);
                params.forEach((param) => generateTooltip(param, bossTooltips, tree, deathTimes, time, length));
                tooltipStr += bossTooltips.join("") + tree.valuesArray().join("") + "</div>";
                return tooltipStr;
            }
        },
        xAxis: {
            type: "category",
            splitLine: {
                show: false
            },
            data: Array.from({ length: chartablePlayers[0].damageStats.dpsRolling10sAvg.length }, (_, i) =>
                formatDurationFromS(i)
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
        series: [...chartPlayers, ...chartBosses]
    };
}

export function getSkillLogChart(
    player: Entity,
    skillIconPath: string,
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
                (_, i) => formatDurationFromS(i)
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
                symbol: "image://" + skillIconPath + getSkillIcon(skill.icon),
                symbolSize: [20, 20],
                symbolKeepAspect: true,
                data: skill.skillCastLog.map((cast) => [
                    formatDurationFromMs(cast.timestamp),
                    skill.name,
                    cast,
                    skill.icon
                ]),
                tooltip: {
                    formatter: function (param: any) {
                        focusedSkillCast.set({ skillId: skill.id, cast: param.dataIndex });
                        let output = "<div class='tooltip-scroll overflow-y-auto max-h-56 pt-1 pb-2 px-2'>";
                        output += `
                        <div class='flex justify-between'>
                        <div class='font-semibold mb-1'>${param.name}-${formatDurationFromMs(param.value[2].last)} (${round((param.value[2].last - param.value[2].timestamp) / 1000)}s)</div>
                        <div class='tracking-tight text-sm'>Scroll Down for Details</div>
                        </div>
                        `;
                        output += "<div>";
                        output += "<div class='flex space-x-1'>";
                        output += `<img class="size-5 rounded-sm" src='${skillIconPath + getSkillIcon(param.value[3])}' alt='${param.seriesName}' />`;
                        output += `<div class='font-semibold'>${param.seriesName}</div>`;
                        output += "</div>";
                        output += skillCastBreakdownTooltip(param.value[2], encounterDamageStats, skillIconPath);
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

export function getSkillLogChartOld(player: Entity, skillIconPath: string, lastCombatPacket: number, fightStart: number) {
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
            formatter: function (params: any[]) {
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
                (_, i) => formatDurationFromS(i)
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
                symbol: "image://" + skillIconPath + getSkillIcon(skill.icon),
                symbolSize: [20, 20],
                symbolKeepAspect: true,
                data: skill.castLog.map((cast) => [formatDurationFromMs(cast), skill.name])
            };
        })
    };
}

export function getOpenerSkills(skills: MiniSkill[], x: number): OpenerSkill[] {
    const comparator = (a: MiniSkill, b: MiniSkill) => a.castLog[0] - b.castLog[0];

    const minHeap = new Heap(comparator);
    for (const skill of skills) {
        minHeap.push(skill);
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
    skillCast: SkillCast,
    encounterDamageStats: EncounterDamageStats,
    iconPath: string
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
    const supportBuffs: SkillChartSupportDamage = { buff: 0, brand: 0, identity: 0 };
    const modInfo: SkillChartModInfo = { crit: 0, critDamage: 0, ba: 0, fa: 0 };
    for (const [i, hit] of skillCast.hits.entries()) {
        const groupedBuffs = getSkillCastBuffs(
            hit.damage,
            hit.buffedBy,
            hit.debuffedBy,
            encounterDamageStats,
            supportBuffs
        );

        const buffString = getFormattedBuffString(groupedBuffs, iconPath);

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
    Crit: <span class='font-semibold'>${round((modInfo.crit / skillCast.hits.length) * 100)}%</span>
    | CDMG: <span class='font-semibold'>${totalDamage !== 0 ? round((modInfo.critDamage / totalDamage) * 100) : 0}%</span>`;
    if (modInfo.ba > 0) {
        output += ` | BA: <span class='font-semibold'>${round((modInfo.ba / skillCast.hits.length) * 100)}%</span>`;
    }
    if (modInfo.fa > 0) {
        output += ` | FA: <span class='font-semibold'>${round((modInfo.fa / skillCast.hits.length) * 100)}%</span>`;
    }
    output += "</div>";
    output += `<div>
    Buff: <span class='font-semibold'>${supportBuffs.buff > 0 ? round((supportBuffs.buff / totalDamage) * 100) : 0}%</span>`;
    output += ` | B: <span class='font-semibold'>${supportBuffs.brand > 0 ? round((supportBuffs.brand / totalDamage) * 100) : 0}%</span>`;
    output += ` | Iden: <span class='font-semibold'>${supportBuffs.identity > 0 ? round((supportBuffs.identity / totalDamage) * 100) : 0}%</span>
    </div>`;
    output += table;
    output += "</div></div>";

    return output;
}
