<script lang="ts">
    import type { StaggerStats } from "$lib/types";
    import { chartable, defaultOptions, type EChartsOptions } from "$lib/utils/charts";
    import { fillMissingElapsedTimes, formatDurationFromS, formatMinutes } from "$lib/utils/numbers";
    interface Props {
        staggerStats: StaggerStats;
    }

    let { staggerStats }: Props = $props();

    let data = fillMissingElapsedTimes(staggerStats.log);

    let staggerLogOptions: EChartsOptions = {
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
            color: "#8365c7",
            type: "line",
            data: data.map((item) => (item as [number, number])[1]),
            showSymbol: false,
            smooth: 0.1
        }
    };
</script>

<div class="px relative top-0">
    {#if staggerStats.average}
        <div class="mt-4">
            <div class="mb-2 text-lg font-medium tracking-tight">Encounter Stagger Stats</div>
            <div>
                Average Stagger: <span class="font-medium">{staggerStats.average.toFixed(1)}%/s</span>
            </div>
            {#if staggerStats.staggersPerMin}
                <div>
                    <span class="font-medium">{staggerStats.staggersPerMin.toFixed(1)}</span>
                    staggers/min, or
                    <span class="font-medium">{formatMinutes(1 / staggerStats.staggersPerMin)}</span> per stagger
                </div>
            {/if}
        </div>
    {/if}
    <div class="mt-4">
        <div class="text-lg font-medium tracking-tight">Stagger Log</div>
        <div class="mt-2 h-[200px]" use:chartable={staggerLogOptions} style="width: calc(100vw - 4.5rem);"></div>
    </div>
</div>
