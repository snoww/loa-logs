/* eslint-disable @typescript-eslint/no-explicit-any */
import { classColors } from "$lib/constants/colors";
import { EntityType, type Entity, BossHpLog, type Skill, OpenerSkill, MiniSkill } from "$lib/types";
import Heap from "heap-js";
import { defaultOptions } from "./charts";
import { abbreviateNumber, formatDurationFromS, resampleData, round2 } from "./numbers";
import { getSkillIcon, isValidName } from "./strings";

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
                    return e.class + "(" + count[i] + ")";
                }
            });
    }
    return chartablePlayers
        .filter((e) => e.entityType === EntityType.PLAYER)
        .map((e) => (isValidName(e.name) ? e.name : e.class));
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

export function getAveragePlayerSeries(chartablePlayers: Entity[], legendNames: string[], fightStart: number) {
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

            return {
                name: legendNames[i],
                color: classColors[player.class].color,
                type: "line",
                data: player.damageStats.dpsAverage,
                showSymbol: false,
                smooth: 0.1,
                markPoint: markPoint,
                yAxisIndex: 0
            };
        });
}

export function getRollingPlayerSeries(chartablePlayers: Entity[], legendNames: string[], fightStart: number) {
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
            return {
                name: legendNames[i],
                color: classColors[player.class].color,
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
    chartBosses: any[]
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
            }
        },
        tooltip: {
            trigger: "axis",
            formatter: function (params: any[]) {
                let tooltipStr = `<div>${params[0].name}</div><div style="min-width: 10rem">`;
                const playerTooltips: string[] = [];
                const bossTooltips: string[] = [];
                params.forEach((param) => {
                    let label = param.seriesName;
                    let value = param.value;
                    if (param.seriesIndex >= Object.keys(chartablePlayers).length) {
                        value = value[1] + "%";
                        bossTooltips.push(
                            `<div style="display:flex; justify-content: space-between;"><div style="padding-right: 1rem;font-weight: 600;">${label}</div><div style="font-weight: 600;">${value}</div></div>`
                        );
                    } else {
                        value = abbreviateNumber(value);
                        label =
                            `<span style="display:inline-block;margin-right:5px;border-radius:10px;width:10px;height:10px;background-color:${param.color}"></span>` +
                            label;
                        playerTooltips.push(
                            `<div style="display:flex; justify-content: space-between;"><div style="padding-right: 1rem">${label}</div><div style="">${value}</div></div>`
                        );
                    }
                });
                tooltipStr += bossTooltips.join("") + playerTooltips.join("") + "</div>";
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
    chartBosses: any[]
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
            }
        },
        tooltip: {
            trigger: "axis",
            formatter: function (params: any[]) {
                let tooltipStr = `<div>${params[0].name}</div><div style="min-width: 10rem">`;
                const playerTooltips: string[] = [];
                const bossTooltips: string[] = [];
                params.forEach((param) => {
                    let label = param.seriesName;
                    let value = param.value;
                    if (param.seriesIndex >= Object.keys(chartablePlayers).length) {
                        value = value[1] + "%";
                        bossTooltips.push(
                            `<div style="display:flex; justify-content: space-between;"><div style="padding-right: 1rem;font-weight: 600;">${label}</div><div style="font-weight: 600;">${value}</div></div>`
                        );
                    } else {
                        value = abbreviateNumber(value);
                        label =
                            `<span style="display:inline-block;margin-right:5px;border-radius:10px;width:10px;height:10px;background-color:${param.color}"></span>` +
                            label;
                        playerTooltips.push(
                            `<div style="display:flex; justify-content: space-between;"><div style="padding-right: 1rem">${label}</div><div style="">${value}</div></div>`
                        );
                    }
                });
                tooltipStr += bossTooltips.join("") + playerTooltips.join("") + "</div>";
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

export function getSkillLogChart(player: Entity, skillIconPath: string, lastCombatPacket: number, fightStart: number) {
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
                let output = `<span style="font-weight: 800">${params[0].name}</span>`;
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
            itemHeight: 20
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
                data: skill.castLog.map((cast) => {
                    return [formatDurationFromS(cast), skill.name];
                })
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
