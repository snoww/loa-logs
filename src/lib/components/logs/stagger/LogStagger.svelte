<script lang="ts">
    import type { StaggerStats } from "$lib/types";
    import { chartable, defaultOptions, type EChartsOptions } from "$lib/utils/charts";
    import { fillMissingElapsedTimes, formatDurationFromS } from "$lib/utils/numbers";
    export let staggerStats: StaggerStats;

    let data = fillMissingElapsedTimes(staggerStats.log);

    let staggerLogOptions: EChartsOptions = {
        ...defaultOptions,
        grid: {
            left: '2%',
            right: '5%',
            bottom: '23%',
            top: '10%',
            containLabel: true
        },
        dataZoom: [
            {
                type: 'slider',
                fillerColor: 'rgba(80,80,80,.5)',
                borderColor: "rgba(80,80,80,.5)",
                handleStyle: {
                    color: 'rgba(80,80,80,.5)',
                },
                moveHandleStyle: {
                    color: 'rgba(136,136,136)',
                },
            },
            {
                type: 'inside',
                xAxisIndex: [0],
                throttle: 50,
            },
            {
                type: 'inside',
                yAxisIndex: [0],
                throttle: 50,
                zoomOnMouseWheel: false,
            },

        ],
        xAxis: { 
            type: 'category',
            splitLine: {
                show: false
            },
            boundaryGap: false,
            data: data.map((item) => formatDurationFromS(item[0])),
            axisLabel: {
                color: 'white'
            }
        },
        yAxis: {
            type: 'value',
            splitLine: {
                show: true,
                lineStyle: {
                    color: '#333'
                }
            },
            axisLabel: {
                formatter: '{value}%',
            },
        },
        tooltip: {
            trigger: "axis",
            formatter: function (params: any[]) {                                            
                return `<span style="font-weight: 800">${params[0].name}</span><br/>${params[0].value.toFixed(0)}%`;
            }
        },
        series: {
            color: "#8365c7",
            type: 'line',
            data: data.map((item) => (item as [number, number])[1]),
            showSymbol: false,
            smooth: 0.1,
        }
    }; 
    
</script>
<div class="relative top-0 px">
    {#if staggerStats.average}
    <div class="mt-4">
        <div class="font-bold text-lg mb-2">
            Stats
        </div>
        <div>
            Average Stagger: <span class="font-bold">{staggerStats.average.toFixed(1)}%/s</span>
        </div>
    </div>
    {/if}
    <div class="mt-4">
        <div class="font-bold text-lg">
            Stagger Log
        </div>
        <div class="h-[200px] mt-2" use:chartable={staggerLogOptions} style="width: calc(100vw - 4.5rem);">
        </div>
    </div>
</div>