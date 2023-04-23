<script lang="ts">
    import { MeterState, MeterTab, type Entity, type Encounter, ChartType } from "$lib/types";
    import { abbreviateNumber, formatDurationFromS, millisToMinutesAndSeconds } from "$lib/utils/numbers";
    import { invoke } from "@tauri-apps/api/tauri";
    import LogDamageMeterRow from "./LogDamageMeterRow.svelte";
    import LogPlayerBreakdown from "./LogPlayerBreakdown.svelte";
    import LogEncounterInfo from "./LogEncounterInfo.svelte";
    import LogBuffs from "./LogBuffs.svelte";
    import { page } from "$app/stores";
    import { chartable, defaultOptions, type EChartsOptions } from "$lib/utils/charts";
    import { classColors } from "$lib/constants/colors";
    import { settings, skillIcon } from "$lib/utils/settings";
    import { goto } from "$app/navigation";
    import html2canvas from 'html2canvas';
    import { screenshotAlert, screenshotError, takingScreenshot } from "$lib/utils/stores";
    import { getSkillIcon } from "$lib/utils/strings";
    import LogIdentity from "./identity/LogIdentity.svelte";

    export let id: string;
    export let encounter: Encounter;

    let players: Array<Entity> = [];
    let player: Entity | null = null;
    let playerDamagePercentages: Array<number> = [];
    let topDamageDealt = 0;
    let localPlayer: Entity | null = null;
    
    let anyDead: boolean;


    let state = MeterState.PARTY;
    let tab = MeterTab.DAMAGE;
    let chartType = ChartType.AVERAGE_DPS;
    let playerName = "";

    let deleteConfirm = false;

    let avgDpsOptions: EChartsOptions = {};
    let rollingDpsOptions: EChartsOptions = {};
    let skillLogOptions: EChartsOptions = {};

    $: {        
        if (encounter) {           
            players = Object.values(encounter.entities)
                .filter((players) => players.damageStats.damageDealt > 0)
                .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
            topDamageDealt = encounter.encounterDamageStats.topDamageDealt;
            playerDamagePercentages = players.map(player => (player.damageStats.damageDealt / topDamageDealt) * 100);
            anyDead = players.some(player => player.isDead);

            if (encounter.localPlayer) {
                localPlayer = encounter.entities[encounter.localPlayer];
            }
 
            if (playerName) {
                player = encounter.entities[playerName];
                state = MeterState.PLAYER;
            } else {
                player = null;
                state = MeterState.PARTY;
            }

            if (players.length > 0 && players[0].damageStats && players[0].damageStats.dpsAverage.length > 0 && players[0].damageStats.dpsRolling10sAvg.length > 0)
            {
                let legendNames: Array<string> = [];
                if (!$settings.general.showNames) {
                    let map: {[key: string]: number} = {}
                    let count = players.map(e => {
                        return map[e.class] = (typeof map[e.class] === "undefined") ? 1 : map[e.class] + 1;
                    })
                    legendNames = players.map((e, i) => {
                        if (map[e.class] === 1) {
                            return e.class;
                        } else {
                            return e.class + "(" + count[i] + ")";
                        }
                    })
                } else {
                    legendNames = players.map((e) => e.name);
                }

                if (chartType === ChartType.AVERAGE_DPS) {
                    avgDpsOptions = {
                        ...defaultOptions,
                        legend: {
                            data: legendNames,
                            textStyle: {
                                color: 'white'
                            },
                            type: 'scroll',
                            width: '90%',
                            pageIconInactiveColor: "#313131",
                            pageIconColor: "#aaa",
                            pageTextStyle: {
                                color: "#aaa"
                            },
                        },
                        xAxis: { 
                            type: 'category',
                            splitLine: {
                                show: false
                            },
                            data: Array.from({length: players[0].damageStats.dpsAverage.length}, (_, i) => formatDurationFromS(i * 5)),
                            boundaryGap: false,
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
                                color: 'white',
                                formatter: function(value: number) {
                                    return abbreviateNumber(value);
                                }
                            }
                        },
                        series: players.map((player, i) => {
                            return {
                                name: legendNames[i],
                                color: classColors[player.class].color,
                                type: 'line',
                                data: player.damageStats.dpsAverage,
                                showSymbol: false,
                                smooth: 0.1
                            }
                        })
                    };
                } else if (chartType === ChartType.ROLLING_DPS) {
                    rollingDpsOptions = {
                        ...defaultOptions,
                        legend: {
                            data: legendNames,
                            textStyle: {
                                    color: 'white'
                                },
                                type: 'scroll',
                                width: '90%',
                                pageIconInactiveColor: "#313131",
                                pageIconColor: "#aaa",
                                pageTextStyle: {
                                    color: "#aaa"
                                },
                        },
                        xAxis: { 
                            type: 'category',
                            splitLine: {
                                show: false
                            },
                            data: Array.from({length: players[0].damageStats.dpsRolling10sAvg.length}, (_, i) => formatDurationFromS(i)),
                            boundaryGap: false,
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
                                color: 'white',
                                formatter: function(value: number) {
                                    return abbreviateNumber(value);
                                }
                            }
                        },
                        series: players.map((player, i) => {
                            return {
                                name: legendNames[i],
                                color: classColors[player.class].color,
                                type: 'line',
                                data: player.damageStats.dpsRolling10sAvg,
                                showSymbol: false,
                                smooth: 0.1
                            }
                        })
                    }
                } else if (chartType === ChartType.SKILL_LOG && player) {
                    let sortedSkills = Object.values(player.skills).filter(skill => skill.castLog.length > 0).sort((a, b) => a.totalDamage - b.totalDamage)
                    let skills = sortedSkills.map(skill => skill.name);
                    skillLogOptions = {
                        ...defaultOptions,
                        grid: {
                            left: '2%',
                            right: '5%',
                            bottom: '18%',
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
                                }
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
                        tooltip: {
                            trigger: "axis",
                            formatter: function (params: any[]) {
                                let output = `<span style="font-weight: 800">${params[0].name}</span>`
                                params.forEach(p => {
                                    output += `<br/>${p.seriesName}`
                                })
                                
                                return output;
                            }
                        } as any,
                        legend: {
                            data: [...skills].reverse(),
                            textStyle: {
                                color: 'white'
                            },
                            type: 'scroll',
                            width: '90%',
                            pageIconInactiveColor: "#313131",
                            pageIconColor: "#aaa",
                            pageTextStyle: {
                                color: "#aaa"
                            },
                            itemWidth: 20,
                            itemHeight: 20,
                        },
                        xAxis: { 
                            type: 'category',
                            splitLine: {
                                show: false
                            },
                            data: Array.from({length: (encounter.lastCombatPacket - encounter.fightStart) / 1000}, (_, i) => formatDurationFromS(i)),
                            boundaryGap: false,
                            axisLabel: {
                                color: 'white'
                            },
                            
                        },
                        yAxis: {
                            type: 'category',
                            splitLine: {
                                show: true,
                                lineStyle: {
                                    color: '#333'
                                }
                            },
                            axisLabel: {
                                show: false,
                            },
                            data: skills.map((skill) => {
                                return {
                                    value: skill
                                }
                            }),
                        },
                        series: sortedSkills.map((skill) => {
                            return {
                                name: skill.name,
                                type: 'scatter',
                                symbol: 'image://' + $skillIcon.path + getSkillIcon(skill.icon),
                                symbolSize: [20, 20],
                                symbolKeepAspect: true,
                                data: skill.castLog.map((cast) => {
                                    return [formatDurationFromS(cast), skill.name]
                                })
                            }
                        })
                    }
                }
            }
        }
    }

    function inspectPlayer(name: string) {
        state = MeterState.PLAYER;
        playerName = name;
        chartType = ChartType.SKILL_LOG;
    }

    function damageTab() {
        tab = MeterTab.DAMAGE;
        setChartView();
    }

    function partySynergyTab() {
        tab = MeterTab.PARTY_BUFFS;
        setChartView();
    }

    function selfSynergyTab() {
        tab = MeterTab.SELF_BUFFS;
        setChartView();
    }

    function identityTab() {
        if (!localPlayer) return
        tab = MeterTab.IDENTITY;
        chartType = ChartType.IDENTITY
    }

    function setChartView() {
        if (state === MeterState.PARTY) {
            chartType = ChartType.AVERAGE_DPS;
        } else if (state === MeterState.PLAYER) {
            chartType = ChartType.SKILL_LOG;
        }
    }

    function handleRightClick() {
        if (state === MeterState.PLAYER) {
            state = MeterState.PARTY;
            player = null;
            playerName = "";
            chartType = ChartType.AVERAGE_DPS;
        }
    }

    async function deleteEncounter() {
        await invoke("delete_encounter", { id: id });
        if ($page.url.searchParams.has('page')) {
            let currentPage = parseInt($page.url.searchParams.get('page')!);
            goto(`/logs?page=${currentPage}`);
        } else {
            goto("/logs");
        }
    }

    let dropdownOpen = false;

    const handleDropdownClick = () => {
        dropdownOpen = !dropdownOpen
    }

    const handleDropdownFocusLoss = (event: FocusEvent) => {
        const relatedTarget = event.relatedTarget as HTMLElement;
        const currentTarget = event.currentTarget as HTMLElement;

        if (currentTarget.contains(relatedTarget)) return;
        
        dropdownOpen = false;
    };

    let targetDiv: HTMLElement;

    async function captureScreenshot() {
        takingScreenshot.set(true);
        setTimeout(async () => {
            const canvas = await html2canvas(targetDiv, {
                useCORS: true,
                backgroundColor: "#27272A",
            });

            canvas.toBlob(async (blob) => {
                if (!blob) return;
                try {
                    const item = new ClipboardItem({ 'image/png': blob });
                    await navigator.clipboard.write([item]);
                    takingScreenshot.set(false);
                    $screenshotAlert = true;
                    setTimeout(() => {
                        $screenshotAlert = false;
                    }, 2000);
                } catch (error) {
                    takingScreenshot.set(false);
                    $screenshotError = true;
                    setTimeout(() => {
                        $screenshotError = false;
                    }, 2000);
                }
            });
        }, 100);

    }
    
</script>

<svelte:window on:contextmenu|preventDefault/>
<div bind:this={targetDiv} class:p-4={$takingScreenshot}>
    <LogEncounterInfo bossName={encounter.currentBossName} encounterDuration={millisToMinutesAndSeconds(encounter.duration)} 
                        totalDamageDealt={encounter.encounterDamageStats.totalDamageDealt} 
                        dps={encounter.encounterDamageStats.dps}/>
    {#if !$takingScreenshot}
    <div class="mt-2 flex justify-between" style="width: calc(100vw - 4.5rem);">
        <div class="flex divide-x divide-gray-600">
            <button class="px-2 rounded-sm py-1" class:bg-accent-900={tab == MeterTab.DAMAGE} class:bg-gray-700={tab != MeterTab.DAMAGE} on:click={damageTab}>
                Damage
            </button>
            <button class="px-2 rounded-sm py-1" class:bg-accent-900={tab == MeterTab.PARTY_BUFFS} class:bg-gray-700={tab != MeterTab.PARTY_BUFFS} on:click={partySynergyTab}>
                Party Synergy
            </button>
            <button class="px-2 rounded-sm py-1" class:bg-accent-900={tab == MeterTab.SELF_BUFFS} class:bg-gray-700={tab != MeterTab.SELF_BUFFS} on:click={selfSynergyTab}>
                Self Synergy
            </button>
            {#if localPlayer && localPlayer.skillStats.identityStats}
            <button class="px-2 rounded-sm py-1" class:bg-accent-900={tab == MeterTab.IDENTITY} class:bg-gray-700={tab != MeterTab.IDENTITY} on:click={identityTab}>
                Identity
            </button>
            {/if}
            <div class="bg-gray-700 flex items-center relative rounded-sm" on:focusout={handleDropdownFocusLoss}>
                <button on:click={handleDropdownClick} class="px-2 h-full">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path></svg>
                </button>
                {#if dropdownOpen}
                    <div class="absolute top-0 left-9 z-50 bg-gray-700 rounded-md">
                        <div class="flex flex-col px-2 py-1 w-40 divide-y-2 divide-gray-600">
                            <button class="text-left p-1 hover:text-accent-500" on:click={() => {dropdownOpen = false; captureScreenshot();}}>
                                Take Screenshot
                            </button>
                            <button class="flex items-center bg-gray-700 justify-between p-1">
                                <span class="text-sm">Show Names</span>
                                <label class="relative inline-flex items-center cursor-pointer">
                                    <input type="checkbox" value="" class="sr-only peer" bind:checked={$settings.general.showNames}>
                                    <div class="w-9 h-5 peer-focus:outline-none rounded-full peer bg-gray-800 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all border-gray-600 peer-checked:bg-accent-800"></div>
                                </label>
                            </button>
                            <button class="text-left p-1 hover:text-red-600" on:click={() => {dropdownOpen = false; deleteConfirm = true}}>
                                Delete
                            </button>
                        </div>
                    </div>
                {/if}
            </div>
        </div>
    
        {#if deleteConfirm}
        <div class="fixed inset-0 z-50 bg-zinc-900 bg-opacity-80"></div>
        <div class="fixed top-0 left-0 right-0 h-modal z-50 w-full p-4 justify-center items-center">
            <div class="flex relative max-w-md w-full max-h-full mx-auto top-[25%]">
                <div class="bg-zinc-800 text-gray-400 rounded-lg border-gray-700 shadow-md relative flex flex-col mx-auto">
                    <button type="button" class="focus:outline-none whitespace-normal rounded-lg p-1.5 hover:bg-zinc-600 ml-auto absolute top-3 right-2.5" aria-label="Close modal" on:click={() => deleteConfirm = false}>
                        <span class="sr-only">Close modal</span> <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"></path></svg>
                    </button> 
                    <div id="modal" class="p-6 space-y-6 flex-1 overflow-y-auto overscroll-contain">
                        <div class="text-center">
                            <svg aria-hidden="true" class="mx-auto mb-4 w-14 h-14 text-gray-200" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" class="s-Qbr4I8QhaoSZ"></path></svg> 
                            <h3 class="mb-5 text-lg font-normal text-gray-400">Are you sure you want to delete this encounter?</h3> 
                            <button type="button" class="text-center font-medium focus:outline-none inline-flex items-center justify-center px-5 py-2.5 text-sm text-white bg-red-700 hover:bg-red-800 rounded-lg mr-2" on:click={deleteEncounter}>
                                Yes, I'm sure
                            </button> 
                            <button type="button" class="text-center font-medium focus:outline-none inline-flex items-center justify-center px-5 py-2.5 text-sm bg-gray-800 text-gray-400 focus:text-white hover:text-white hover:bg-zinc-700 bg-transparent rounded-lg" on:click={() => deleteConfirm = false}>
                                No, cancel
                            </button>
                        </div>
                    </div> 
                </div>
            </div>
        </div>
        {/if}
    </div>
    {/if}
    {#if tab !== MeterTab.IDENTITY}
    <div class="relative top-0 px" id="buff-table">
        <table class="table-fixed w-full relative">
            {#if tab === MeterTab.DAMAGE}
                {#if state === MeterState.PARTY}
                <thead class="h-6 z-30" on:contextmenu|preventDefault={() => {console.log("titlebar clicked")}}>
                    <tr class="bg-zinc-900">
                        <th class="text-left px-2 font-normal w-full"></th>
                        {#if anyDead && $settings.logs.deathTime}
                        <th class="font-normal w-20">Dead for</th>
                        {/if}
                        {#if $settings.logs.damage}
                        <th class="font-normal w-14">DMG</th>
                        {/if}
                        {#if $settings.logs.dps}
                        <th class="font-normal w-14">DPS</th>
                        {/if}
                        {#if players.length > 1 && $settings.logs.damagePercent}
                        <th class="font-normal w-14">D%</th>
                        {/if}
                        {#if $settings.logs.critRate}
                        <th class="font-normal w-14">CRIT</th>
                        {/if}
                        {#if $settings.logs.frontAtk}
                        <th class="font-normal w-14">F.A</th>
                        {/if}
                        {#if $settings.logs.backAtk}
                        <th class="font-normal w-14">B.A</th>
                        {/if}
                        {#if $settings.logs.counters}
                        <th class="font-normal w-[70px]">Counters</th>
                        {/if}
                    </tr>
                </thead>
                <tbody>
                    {#each players as player, i (player.name)}
                    <tr class="h-7 px-2 py-1" on:click={() => inspectPlayer(player.name)}>
                            <LogDamageMeterRow entity={player} 
                                                percentage={playerDamagePercentages[i]} 
                                                totalDamageDealt={encounter.encounterDamageStats.totalDamageDealt} 
                                                {anyDead} 
                                                end={encounter.lastCombatPacket}
                                               />
                    </tr>
                    {/each}
                </tbody>
                {:else if state === MeterState.PLAYER && player !== null}
                   <LogPlayerBreakdown {player} duration={encounter.duration} {handleRightClick}/>
                {/if}
            {:else if tab === MeterTab.PARTY_BUFFS}
                {#if state === MeterState.PARTY}
                    <LogBuffs {tab} encounterDamageStats={encounter.encounterDamageStats} {players} percentages={playerDamagePercentages} {handleRightClick} {inspectPlayer}/>
                {:else}
                    <LogBuffs {tab} encounterDamageStats={encounter.encounterDamageStats} {players} percentages={playerDamagePercentages} focusedPlayer={player} {handleRightClick} {inspectPlayer}/>
                {/if}
            {:else if tab === MeterTab.SELF_BUFFS}
                {#if state === MeterState.PARTY}
                    <LogBuffs {tab} encounterDamageStats={encounter.encounterDamageStats} {players} percentages={playerDamagePercentages} {handleRightClick} {inspectPlayer}/>
                {:else}
                    <LogBuffs {tab} encounterDamageStats={encounter.encounterDamageStats} {players} percentages={playerDamagePercentages} focusedPlayer={player} {handleRightClick} {inspectPlayer}/>
                {/if}
            {/if}
        </table>
    </div>
    {:else if tab === MeterTab.IDENTITY && localPlayer !== null}
        <LogIdentity localPlayer={localPlayer}/>
    {/if}
</div>
{#if tab !== MeterTab.IDENTITY}
<div class="mt-4">
    <div class="font-bold text-lg">
        Charts
    </div>
    <div class="flex divide-x divide-gray-600 mt-2">
        {#if playerName === "" && state === MeterState.PARTY}
        <button class="px-2 rounded-sm py-1" class:bg-accent-900={chartType == ChartType.AVERAGE_DPS} class:bg-gray-700={chartType != ChartType.AVERAGE_DPS} on:click={() => chartType = ChartType.AVERAGE_DPS}>
            Average DPS
        </button>
        <button class="px-2 rounded-sm py-1" class:bg-accent-900={chartType == ChartType.ROLLING_DPS} class:bg-gray-700={chartType != ChartType.ROLLING_DPS} on:click={() => chartType = ChartType.ROLLING_DPS}>
            10s DPS Window
        </button>
        {:else if playerName !== "" && state === MeterState.PLAYER}
        <button class="px-2 rounded-sm py-1" class:bg-accent-900={chartType == ChartType.SKILL_LOG} class:bg-gray-700={chartType != ChartType.SKILL_LOG} on:click={() => chartType = ChartType.SKILL_LOG}>
            Skill Casts
        </button>
        {:else if localPlayer}
        <button class="px-2 rounded-sm py-1" class:bg-accent-900={chartType === ChartType.IDENTITY} class:bg-gray-700={chartType != ChartType.IDENTITY} on:click={() => chartType = ChartType.IDENTITY}>
            Identity Gain
        </button>
        {/if}
    </div>
    {#if chartType === ChartType.AVERAGE_DPS}
        {#if !$settings.general.showNames}
        <div class="w-full h-[300px] mt-2" use:chartable={avgDpsOptions}>
        </div>
        {:else}
        <div class="w-full h-[300px] mt-2" use:chartable={avgDpsOptions}>
        </div>
        {/if}
    {:else if chartType === ChartType.ROLLING_DPS}
        {#if !$settings.general.showNames}
        <div class="w-full h-[300px] mt-2" use:chartable={rollingDpsOptions}>
        </div>
        {:else}
        <div class="w-full h-[300px] mt-2" use:chartable={rollingDpsOptions}>
        </div>
        {/if}
    {:else if chartType === ChartType.SKILL_LOG}
        {#if player}
        <div class="w-full h-[400px] mt-2" use:chartable={skillLogOptions}>
        </div>
        {/if}
    {/if}
</div>
{/if}