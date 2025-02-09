<script lang="ts">
    import { cardIds, cardMap } from "$lib/constants/cards";
    import type { Entity, IdentityStats } from "$lib/types";
    import { chartable, defaultOptions, type EChartsOptions } from "$lib/utils/charts";
    import { fillMissingElapsedTimes, formatDurationFromS } from "$lib/utils/numbers";
    import { colors } from "$lib/utils/settings";

    interface Props {
        identityStats: IdentityStats;
        player: Entity;
        duration: number;
    }

    let { identityStats, player, duration }: Props = $props();

    let cards = Object.values(player.skills)
        .sort((a, b) => b.casts - a.casts)
        .filter((skill) => cardIds.includes(skill.id) || skill.id === 19282);
    let totalDraws = cards.reduce((acc, skill) => acc + skill.casts, 0);

    let data = fillMissingElapsedTimes(identityStats.log);

    let identityLogOptions: EChartsOptions = {
        ...defaultOptions,
        grid: {
            left: "2%",
            right: "5%",
            bottom: "23%",
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
        xAxis: {
            type: "category",
            splitLine: {
                show: false
            },
            boundaryGap: false,
            data: data.map((item) => formatDurationFromS(item[0])),
            axisLabel: {
                color: "white"
            }
        },
        yAxis: {
            type: "value",
            splitLine: {
                show: true,
                lineStyle: {
                    color: "#333"
                }
            }
        },
        tooltip: {
            trigger: "axis",
            formatter: function (params: any[]) {
                let output = `<div style="min-width: 70px"><span style="font-weight: 800">${params[0].name}</span>`;
                output += `<br/>${params[0].value.toFixed(0)}%<br/><span style="font-weight: 800">${
                    params[1].value
                }</span></div>`;

                return output;
            }
        },
        series: [
            {
                color: $colors["Arcanist"].color,
                type: "line",
                data: data.map((item) => (item as [number, [number, number, number]])[1][0]),
                showSymbol: false,
                smooth: 0.1
            },
            {
                type: "scatter",
                symbol: "none",
                data: data.map((item) => {
                    let cards = (item as [number, [number, number, number]])[1].slice(1);
                    let str = "";
                    if (cards[0] !== 0) {
                        str += cardMap[cards[0]];
                    }
                    if (cards[1] !== 0) {
                        str += " | " + cardMap[cards[1]];
                    } else {
                        str += " |";
                    }
                    return str;
                })
            }
        ]
    };
</script>

<div class="px relative top-0" id="buff-table">
    {#if identityStats.average}
        <div class="mt-4">
            <div class="mb-2 text-lg font-medium tracking-tight">Arcana Identity Stats</div>
            <div>
                Total Cards Drawn: <span class="font-medium">{totalDraws.toLocaleString()}</span>
            </div>
            <div>
                Draws per min: <span class="font-medium"
                    >{(totalDraws / (duration / 1000 / 60)).toFixed(1)} cards/min</span>
            </div>
        </div>
    {/if}
    <div class="mt-4">
        <div class="text-lg font-medium tracking-tight">Identity Log</div>
        <div class="mt-2 h-[220px]" use:chartable={identityLogOptions} style="width: calc(100vw - 4.5rem);"></div>
    </div>
</div>
