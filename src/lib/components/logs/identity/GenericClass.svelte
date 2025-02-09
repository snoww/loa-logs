<script lang="ts">
    import type { IdentityStats } from "$lib/types";
    import { chartable, defaultOptions, type EChartsOptions } from "$lib/utils/charts";
    import { fillMissingElapsedTimes, formatDurationFromS } from "$lib/utils/numbers";
    import { colors } from "$lib/utils/settings";

    interface Props {
        className: string;
        identityStats: IdentityStats;
    }

    let { className, identityStats }: Props = $props();

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
            },
            axisLabel: {
                formatter: "{value}%"
            }
        },
        tooltip: {
            trigger: "axis",
            formatter: function (params: any[]) {
                return `<span style="font-weight: 800">${params[0].name}</span><br/>${params[0].value.toFixed(0)}%`;
            }
        },
        series: {
            color: $colors[className].color,
            type: "line",
            data: data.map((item) => (item as [number, number])[1]),
            showSymbol: false,
            smooth: 0.1
        }
    };
</script>

<div class="px relative top-0">
    {#if identityStats.average}
        <div class="mt-4">
            <div class="mb-2 text-lg font-medium tracking-tight">{className} Identity Stats</div>
            <div>
                Average Identity Gain: <span class="font-medium">{identityStats.average.toFixed(1)}%/s</span>
            </div>
            <div>
                Identity per min: <span class="font-medium"
                    >{((identityStats.average / 100) * 60).toFixed(1)} per min</span>
            </div>
        </div>
    {/if}
    <div class="mt-4">
        <div class="text-lg font-medium">Identity Log</div>
        <div class="mt-2 h-[200px]" use:chartable={identityLogOptions} style="width: calc(100vw - 4.5rem);"></div>
    </div>
</div>
