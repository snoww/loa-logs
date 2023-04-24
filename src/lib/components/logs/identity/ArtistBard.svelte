<script lang="ts">
    import { classColors } from "$lib/constants/colors";
    import type { IdentityStats } from "$lib/types";
    import { chartable, defaultOptions, type EChartsOptions } from "$lib/utils/charts";
    import { fillMissingElapsedTimes, formatDurationFromS } from "$lib/utils/numbers";

    export let className: string
    export let identityStats: IdentityStats;

    let data = fillMissingElapsedTimes(identityStats.log);

    let identityLogOptions: EChartsOptions = {
        ...defaultOptions,
        grid: {
            left: '2%',
            right: '5%',
            bottom: '20%',
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
            splitNumber: 3,
            minInterval: 100, 
            axisLabel: {
                formatter: '{value}%',
            },
            max: 300
        },
        tooltip: {
            trigger: "axis",
            formatter: function (params: any[]) {                            
                let output = `<span style="font-weight: 800">${params[0].name}</span>`;
                if (params[1].value === "1") {
                    output += `<br/>${params[0].value.toFixed(0)}%<br/><span style="font-weight: 800">1 Bubble<span style="font-weight: 800">`;
                } else {
                    output += `<br/>${params[0].value.toFixed(0)}%<br/><span style="font-weight: 800">` + params[1].value + ` Bubbles<span style="font-weight: 800">`;
                }
                
                return output;
            }
        },
        series: [{
            color: classColors[className].color,
            type: 'line',
            data: data.map((item) => (item as [number, [number, number]])[1][0]),
            showSymbol: false,
            smooth: 0.1,
        },
        {
            type: 'scatter',
            symbol: "none",
            data: data.map((item) => {
                let bubbles = (item as [number, [number, number]])[1];
                return bubbles[1].toLocaleString(); 
            }),
        }
        ]
    };    

</script>

<div class="relative top-0 px">
    {#if identityStats.average}
    <div class="mt-4">
        <div class="font-bold text-lg mb-2">
            Stats
        </div>
        <div>
            Average Identity Gain: <span class="font-bold">{identityStats.average.toFixed(1)}%/s</span>
        </div>
    </div>
    {/if}
    <div class="mt-4">
        <div class="font-bold text-lg">
            Identity Log
        </div>
        <div class="h-[250px] mt-2" use:chartable={identityLogOptions} style="width: calc(100vw - 4.5rem);">
        </div>
    </div>
</div>
