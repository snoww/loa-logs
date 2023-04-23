<script lang="ts">
    import { cardIconMap, cardMap } from "$lib/constants/cards";
    import { classColors } from "$lib/constants/colors";
    import type { IdentityStats } from "$lib/types";
    import { chartable, defaultOptions, type EChartsOptions } from "$lib/utils/charts";
    import { HexToRgba } from "$lib/utils/colors";
    import { fillMissingElapsedTimes, formatDurationFromS } from "$lib/utils/numbers";
    import { skillIcon } from "$lib/utils/settings";
    import { takingScreenshot } from "$lib/utils/stores";
    import { getSkillIcon } from "$lib/utils/strings";

    export let identityStats: IdentityStats;

    let cards = Object.entries(identityStats.cardDraws!).sort((a, b) => b[1] - a[1]);
    let totalDraws = cards.reduce((acc, [_, count]) => acc + count, 0);
    let maxDraws = cards[0][1];
    let relativeDrawPercentages = cards.map(([_, count]) => count / maxDraws * 100);    
    let drawPercentages = cards.map(([_, count]) => count / totalDraws * 100);

    let data = fillMissingElapsedTimes(identityStats.log);

    let identityLogOptions: EChartsOptions = {
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
                start: 0,
                endValue: "0:30"
            
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
        },
        tooltip: {
            trigger: "axis",
            formatter: function (params: any[]) {                            
                let output = `<div style="min-width: 70px"><span style="font-weight: 800">${params[0].name}</span>`;
                output += `<br/>${params[0].value.toFixed(0)}%<br/><span style="font-weight: 800">${params[1].value}</span></div>`;
                
                return output;
            }
        },
        series: [{
            color: classColors["Arcanist"].color,
            type: 'line',
            data: data.map((item) => (item as [number, [number, number, number]])[1][0]),
            showSymbol: false,
            smooth: 0.1,
        },
        {
            type: 'scatter',
            symbol: "none",
            data: data.map((item) => {
                let cards = (item as [number, [number, number, number]])[1].slice(1);                
                let str = ""
                if (cards[0] !== 0) {
                    str += cardMap[cards[0]]
                }
                if (cards[1] !== 0) {
                    if (cards[0] !== 0) {
                        str += " | "
                    }
                    str += cardMap[cards[1]]
                }
                return str;
                
            }),
        }]
    };    

</script>

<div class="relative top-0 px" id="buff-table">
    <table class="table-fixed w-full relative">
        <thead class="h-6 z-30" on:contextmenu|preventDefault={() => {console.log("titlebar clicked")}}>
            <tr class="bg-zinc-900">
                <th class="text-left px-2 font-normal w-full"></th>
                <th class="font-normal w-14">Draws</th>
                <th class="font-normal w-20">Draw %</th>
            </tr>
        </thead>
        <tbody class="relative z-10">
            {#each cards as [card, count], i}
            <tr class="h-6 px-2 py-1 text-3xs">
                <td class="px-1 relative z-10">
                    <div class="flex space-x-1 items-center">
                        <img class="h-5 w-5" src={$skillIcon.path + getSkillIcon(cardIconMap[card])} alt={cardMap[card]} />
                        <div class="truncate pl-px">
                            {cardMap[card]}
                        </div>
                    </div>
                </td>
                <td class="px-1 text-center">
                    {count}
                </td>
                <td class="px-1 text-center">
                    {drawPercentages[i].toFixed(1)}<span class="text-3xs text-gray-300">%</span>
                </td>
                <div class="absolute left-0 h-6 px-2 py-1 -z-10" class:shadow-md={!$takingScreenshot}
                    style="background-color: {HexToRgba(classColors["Arcanist"].color, 0.6)}; width: {relativeDrawPercentages[i]}%"
                ></div>
            </tr>
            {/each}
        </tbody>
    </table>
    {#if identityStats.average}
    <div class="mt-4">
        <div class="font-bold text-lg mb-2">
            Stats
        </div>
        <div>
            Total Cards Drawn: <span class="font-bold">{totalDraws.toLocaleString()}</span>
        </div>
        <div>
            Average Identity Gain: <span class="font-bold">{identityStats.average.toFixed(1)}%/s</span>
        </div>
    </div>
    {/if}
    <div class="mt-4">
        <div class="font-bold text-lg">
            Charts
        </div>
        <div class="w-full h-[220px] mt-2" use:chartable={identityLogOptions}>
        </div>
    </div>
</div>

