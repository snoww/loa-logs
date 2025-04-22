<script lang="ts">
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import LogShields from "$lib/components/logs/LogShields.svelte";
    import Notification from "$lib/components/shared/Notification.svelte";
    import { EncounterState } from "$lib/encounter.svelte";
    import {
        ChartType,
        EntityType,
        MeterState,
        MeterTab,
        type Encounter,
        type Entity,
        type PartyInfo
    } from "$lib/types";
    import { chartable, type EChartsOptions } from "$lib/utils/charts";
    import {
        getAverageDpsChart,
        getAveragePlayerSeries,
        getBossHpSeries,
        getDeathTimes,
        getLegendNames,
        getRollingDpsChart,
        getRollingPlayerSeries,
        getSkillLogChart,
        getSkillLogChartOld
    } from "$lib/utils/dpsCharts";
    import { formatTimestampDate, millisToMinutesAndSeconds } from "$lib/utils/numbers";
    import { colors, settings, skillIcon } from "$lib/utils/settings";
    import {
        raidGates,
        screenshotAlert,
        screenshotError,
        takingScreenshot,
        uploadErrorMessage,
        uploadErrorStore
    } from "$lib/utils/stores";
    import { getSupportSynergiesOverTime, getSupportSynergiesOverTimeChart } from "$lib/utils/supportBuffCharts";
    import { LOG_SITE_URL, uploadLog } from "$lib/utils/sync";
    import { tooltip } from "$lib/utils/tooltip";
    import { invoke } from "@tauri-apps/api/tauri";
    import html2canvas from "html2canvas-pro";
    import ArcanistCardTable from "../shared/ArcanistCardTable.svelte";
    import BossBreakdown from "../shared/BossBreakdown.svelte";
    import BossTable from "../shared/BossTable.svelte";
    import Buffs from "../shared/Buffs.svelte";
    import DamageMeterHeader from "../shared/DamageMeterHeader.svelte";
    import DamageTaken from "../shared/DamageTaken.svelte";
    import PlayerRow from "../shared/PlayerRow.svelte";
    import LogIdentity from "./identity/LogIdentity.svelte";
    import LogDamageMeterPartySplit from "./LogDamageMeterPartySplit.svelte";
    import LogEncounterInfo from "./LogEncounterInfo.svelte";
    import LogPlayerBreakdown from "./LogPlayerBreakdown.svelte";
    import LogSkillChart from "./LogSkillChart.svelte";
    import OpenerSkills from "./OpenerSkills.svelte";
    import LogStagger from "./stagger/LogStagger.svelte";

    interface Props {
        id: string;
        encounter: Encounter;
    }

    let { id, encounter = $bindable() }: Props = $props();

    let enc = $derived(new EncounterState(encounter, $settings, false, $colors));

    let hasSkillCastLog = $state(false);

    let deleteConfirm = $state(false);

    let encounterPartyInfo: PartyInfo | undefined = $state(encounter.encounterDamageStats.misc?.partyInfo);

    let localPlayerEntity = $derived(encounter.entities[enc.localPlayer]);

    let meterState = $state(MeterState.PARTY);
    let tab = $state(MeterTab.DAMAGE);
    let chartType = $state(ChartType.AVERAGE_DPS);
    let playerName = $state("");
    let player: Entity | undefined = $derived.by(() => {
        if (playerName) {
            return encounter.entities[playerName];
        }
    });
    let focusedBoss = $state("");

    let chartOptions: EChartsOptions = $state({});
    let chartablePlayers = Object.values(encounter.entities)
        .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.PLAYER && e.classId != 0)
        .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);

    $effect.pre(() => {
        if (
            chartablePlayers.length > 0 &&
            chartablePlayers[0].damageStats &&
            chartablePlayers[0].damageStats.dpsAverage.length > 0 &&
            chartablePlayers[0].damageStats.dpsRolling10sAvg.length > 0
        ) {
            let legendNames = getLegendNames(chartablePlayers, $settings.general.showNames);
            let deathTimes = getDeathTimes(chartablePlayers, legendNames, encounter.fightStart);
            let bossHpLogs = Object.entries(encounter.encounterDamageStats.bossHpLog || {});

            if (chartType === ChartType.AVERAGE_DPS) {
                let chartPlayers = getAveragePlayerSeries(chartablePlayers, legendNames, encounter.fightStart, $colors);
                let bossChart = getBossHpSeries(
                    bossHpLogs,
                    legendNames,
                    chartablePlayers[0].damageStats.dpsAverage.length,
                    5
                );
                chartOptions = getAverageDpsChart(chartablePlayers, legendNames, chartPlayers, bossChart, deathTimes);
            } else if (chartType === ChartType.ROLLING_DPS) {
                let chartPlayers = getRollingPlayerSeries(chartablePlayers, legendNames, encounter.fightStart, $colors);
                let bossChart = getBossHpSeries(
                    bossHpLogs,
                    legendNames,
                    chartablePlayers[0].damageStats.dpsRolling10sAvg.length,
                    1
                );
                chartOptions = getRollingDpsChart(chartablePlayers, legendNames, chartPlayers, bossChart, deathTimes);
            } else if (chartType === ChartType.BRAND_BUFF) {
                const legendNames = new Array<string>();
                const intervalMs = 5000;
                let buffsSeries = getSupportSynergiesOverTime(
                    encounter,
                    chartablePlayers,
                    enc.partyInfo!,
                    encounter.fightStart,
                    encounter.lastCombatPacket,
                    intervalMs,
                    legendNames
                );
                let bossChart = getBossHpSeries(bossHpLogs, legendNames, buffsSeries[0].data.length, 5);
                chartOptions = getSupportSynergiesOverTimeChart(
                    legendNames,
                    buffsSeries,
                    "_1_",
                    bossChart,
                    $skillIcon.path
                );
            } else if (chartType === ChartType.AP_BUFF) {
                const legendNames = new Array<string>();
                const intervalMs = 5000;
                let buffsSeries = getSupportSynergiesOverTime(
                    encounter,
                    chartablePlayers,
                    enc.partyInfo!,
                    encounter.fightStart,
                    encounter.lastCombatPacket,
                    intervalMs,
                    legendNames
                );
                let bossChart = getBossHpSeries(bossHpLogs, legendNames, buffsSeries[0].data.length, 5);
                chartOptions = getSupportSynergiesOverTimeChart(
                    legendNames,
                    buffsSeries,
                    "_0_",
                    bossChart,
                    $skillIcon.path
                );
            } else if (chartType === ChartType.IDENTITY_BUFF) {
                const legendNames = new Array<string>();
                const intervalMs = 5000;
                let buffsSeries = getSupportSynergiesOverTime(
                    encounter,
                    chartablePlayers,
                    enc.partyInfo!,
                    encounter.fightStart,
                    encounter.lastCombatPacket,
                    intervalMs,
                    legendNames
                );
                let bossChart = getBossHpSeries(bossHpLogs, legendNames, buffsSeries[0].data.length, 5);
                chartOptions = getSupportSynergiesOverTimeChart(
                    legendNames,
                    buffsSeries,
                    "_2_",
                    bossChart,
                    $skillIcon.path
                );
            } else if (chartType === ChartType.HAT_BUFF) {
                const legendNames = new Array<string>();
                const intervalMs = 5000;
                let buffsSeries = getSupportSynergiesOverTime(
                    encounter,
                    chartablePlayers,
                    enc.partyInfo!,
                    encounter.fightStart,
                    encounter.lastCombatPacket,
                    intervalMs,
                    legendNames
                );
                let bossChart = getBossHpSeries(bossHpLogs, legendNames, buffsSeries[0].data.length, 5);
                chartOptions = getSupportSynergiesOverTimeChart(
                    legendNames,
                    buffsSeries,
                    "_3_",
                    bossChart,
                    $skillIcon.path
                );
            } else if (chartType === ChartType.SKILL_LOG && player && player.entityType === EntityType.PLAYER) {
                if (
                    Object.entries(player.skills).some(
                        ([, skill]) => skill.skillCastLog && skill.skillCastLog.length > 0
                    )
                ) {
                    hasSkillCastLog = true;
                    chartOptions = getSkillLogChart(
                        player,
                        $skillIcon.path,
                        encounter.lastCombatPacket,
                        encounter.fightStart,
                        encounter.encounterDamageStats
                    );
                } else {
                    chartOptions = getSkillLogChartOld(
                        player,
                        $skillIcon.path,
                        encounter.lastCombatPacket,
                        encounter.fightStart
                    );
                }
            } else if (chartType === ChartType.SKILL_LOG && focusedBoss) {
                let boss = enc.bosses.find((boss) => boss.name === focusedBoss);
                chartOptions = getSkillLogChartOld(
                    boss!,
                    $skillIcon.path,
                    encounter.lastCombatPacket,
                    encounter.fightStart
                );
            }
        }
    });

    function inspectPlayer(name: string) {
        meterState = MeterState.PLAYER;
        playerName = name;
        chartType = ChartType.SKILL_LOG;

        scrollToTop();
    }

    function inspectBoss(name: string) {
        meterState = MeterState.PLAYER;
        chartType = ChartType.SKILL_LOG;
        focusedBoss = name;
    }

    function updateTab(setTab: MeterTab) {
        if (tab === MeterTab.TANK || tab === MeterTab.SHIELDS || tab === MeterTab.BOSS) {
            handleRightClick();
        } else if (tab === MeterTab.IDENTITY) {
            if (!localPlayerEntity) return;
            chartType = ChartType.IDENTITY;
        } else if (tab === MeterTab.STAGGER) {
            chartType = ChartType.STAGGER;
        }
        tab = setTab;
        setChartView();
    }

    function setChartView() {
        if (meterState === MeterState.PARTY) {
            chartType = ChartType.AVERAGE_DPS;
        } else if (meterState === MeterState.PLAYER) {
            chartType = ChartType.SKILL_LOG;
        }
    }

    function handleRightClick(e: MouseEvent | undefined = undefined) {
        if (e) {
            e.preventDefault();
        }
        if (meterState === MeterState.PLAYER) {
            meterState = MeterState.PARTY;
            playerName = "";
            chartType = ChartType.AVERAGE_DPS;
            scrollToTop();
        }
    }

    function scrollToTop() {
        if (targetDiv) {
            targetDiv.scrollIntoView({ behavior: "smooth", block: "start", inline: "start" });
        }
    }

    async function deleteEncounter() {
        await invoke("delete_encounter", { id: id });
        if (page.url.searchParams.has("page")) {
            let currentPage = parseInt(page.url.searchParams.get("page")!);
            goto(`/logs?page=${currentPage}`);
        } else {
            goto("/logs");
        }
    }

    let dropdownOpen = $state(false);

    const handleDropdownClick = () => {
        dropdownOpen = !dropdownOpen;
    };

    const handleDropdownFocusLoss = (event: FocusEvent) => {
        const relatedTarget = event.relatedTarget as HTMLElement;
        const currentTarget = event.currentTarget as HTMLElement;

        if (currentTarget.contains(relatedTarget)) return;

        dropdownOpen = false;
    };

    let targetDiv: HTMLElement | undefined = $state();

    let uploading = $state(false);

    async function upload() {
        if (encounter.sync || uploading) {
            return;
        }

        if (!$settings.sync.enabled) {
            $uploadErrorStore = true;
            $uploadErrorMessage = "Upload not enabled";
            return;
        }

        if (!$settings.sync.accessToken || !$settings.sync.validToken) {
            $uploadErrorStore = true;
            $uploadErrorMessage = "Upload token is invalid";
            return;
        }

        uploading = true;
        let result = await uploadLog(id, encounter, $settings.sync);
        if (result.error) {
            $uploadErrorStore = true;
            $uploadErrorMessage = "upload error: " + result.error;
        } else {
            encounter.sync = result.id;
        }

        uploading = false;
    }

    async function captureScreenshot() {
        takingScreenshot.set(true);
        setTimeout(async () => {
            if (!targetDiv) {
                takingScreenshot.set(false);
                return;
            }

            const canvas = await html2canvas(targetDiv, { useCORS: true, backgroundColor: "#27272A" });

            canvas.toBlob(async (blob) => {
                if (!blob) return;
                try {
                    const item = new ClipboardItem({ "image/png": blob });
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

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    bind:this={targetDiv}
    class="scroll-mt-2 scroll-ml-8 text-gray-100"
    class:p-4={$takingScreenshot}
    oncontextmenu={handleRightClick}>
    <LogEncounterInfo
        boss={encounter.entities[encounter.currentBossName]}
        difficulty={encounter.difficulty}
        date={formatTimestampDate(encounter.fightStart, true)}
        encounterDuration={millisToMinutesAndSeconds(encounter.duration)}
        totalDamageDealt={enc.totalDamageDealt}
        dps={encounter.encounterDamageStats.dps}
        cleared={encounter.cleared}
        bossOnlyDamage={encounter.bossOnlyDamage}
        raidGate={$raidGates.get(encounter.currentBossName)} />
    {#if !$takingScreenshot}
        <div class="mt-2 flex justify-between" style="width: calc(100vw - 4.5rem);">
            <div class="flex divide-x divide-gray-600">
                <button
                    class="rounded-xs px-2 py-1"
                    class:bg-accent-900={tab === MeterTab.DAMAGE}
                    class:bg-gray-700={tab !== MeterTab.DAMAGE}
                    onclick={() => updateTab(MeterTab.DAMAGE)}>
                    Damage
                </button>
                <!--{#if anyRdpsData || $rdpsEventDetails !== ""}-->
                <!--    <button-->
                <!--        class="shrink-0 rounded-xs px-3 py-1"-->
                <!--        class:bg-accent-900={tab === MeterTab.RDPS}-->
                <!--        class:bg-gray-700={tab !== MeterTab.RDPS}-->
                <!--        on:click={RDPSTab}>-->
                <!--        RDPS-->
                <!--    </button>-->
                <!--{/if}-->
                <button
                    class="shrink-0 rounded-xs px-2 py-1"
                    class:bg-accent-900={tab === MeterTab.PARTY_BUFFS}
                    class:bg-gray-700={tab !== MeterTab.PARTY_BUFFS}
                    onclick={() => updateTab(MeterTab.PARTY_BUFFS)}>
                    Party Buffs
                </button>
                <button
                    class="shrink-0 rounded-xs px-2 py-1"
                    class:bg-accent-900={tab === MeterTab.SELF_BUFFS}
                    class:bg-gray-700={tab !== MeterTab.SELF_BUFFS}
                    onclick={() => updateTab(MeterTab.SELF_BUFFS)}>
                    Self Buffs
                </button>
                {#if $settings.general.showShields && encounter.encounterDamageStats.totalShielding > 0}
                    <button
                        class="rounded-xs px-2 py-1"
                        class:bg-accent-900={tab === MeterTab.SHIELDS}
                        class:bg-gray-700={tab !== MeterTab.SHIELDS}
                        onclick={() => updateTab(MeterTab.SHIELDS)}>
                        Shields
                    </button>
                {/if}
                {#if $settings.general.showTanked && encounter.encounterDamageStats.totalDamageTaken > 0}
                    <button
                        class="rounded-xs px-2 py-1"
                        class:bg-accent-900={tab === MeterTab.TANK}
                        class:bg-gray-700={tab !== MeterTab.TANK}
                        onclick={() => updateTab(MeterTab.TANK)}>
                        Tanked
                    </button>
                {/if}
                {#if $settings.general.showBosses && enc.bosses.length > 0}
                    <button
                        class="rounded-xs px-2 py-1"
                        class:bg-accent-900={tab === MeterTab.BOSS}
                        class:bg-gray-700={tab !== MeterTab.BOSS}
                        onclick={() => updateTab(MeterTab.BOSS)}>
                        Bosses
                    </button>
                {/if}
                {#if localPlayerEntity && localPlayerEntity.skillStats.identityStats}
                    <button
                        class="rounded-xs px-2 py-1"
                        class:bg-accent-900={tab === MeterTab.IDENTITY}
                        class:bg-gray-700={tab !== MeterTab.IDENTITY}
                        onclick={() => updateTab(MeterTab.IDENTITY)}>
                        Identity
                    </button>
                {/if}
                {#if encounter.encounterDamageStats.staggerStats}
                    <button
                        class="rounded-xs px-2 py-1"
                        class:bg-accent-900={tab === MeterTab.STAGGER}
                        class:bg-gray-700={tab !== MeterTab.STAGGER}
                        onclick={() => updateTab(MeterTab.STAGGER)}>
                        Stagger
                    </button>
                {/if}
                <button
                    class="rounded-xs bg-gray-700 px-2 py-1"
                    aria-label="Take Screenshot"
                    use:tooltip={{ content: "Take Screenshot" }}
                    onclick={captureScreenshot}>
                    <svg
                        class="hover:fill-accent-800 h-5 w-5 fill-zinc-300"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 -960 960 960">
                        <path
                            d="M479.5-269.5q71.75 0 119.625-47.875T647-437q0-71-47.875-118.75T479.5-603.5q-71.75 0-119.125 47.75T313-437q0 71.75 47.375 119.625T479.5-269.5Zm0-57.5q-47 0-78-31.145T370.5-437q0-47 31-78t78-31q47 0 78.5 31t31.5 78.25q0 47.25-31.5 78.5T479.5-327Zm-328 227.5q-38.019 0-64.76-26.741Q60-152.981 60-191v-491.5q0-37.431 26.74-64.966Q113.482-775 151.5-775h132l83.057-97.5H594.5l82 97.5h132q37.431 0 64.966 27.534Q901-719.931 901-682.5V-191q0 38.019-27.534 64.759Q845.931-99.5 808.5-99.5h-657Zm657-91.5v-491.5H635L552.5-780H408.451L325.5-682.5h-174V-191h657ZM480-436.5Z" />
                    </svg>
                </button>
                {#if encounter.cleared}
                    {#if uploading}
                        <div class="rounded-xs bg-gray-700 px-2 py-1" use:tooltip={{ content: "Uploading..." }}>
                            <div class="group flex space-x-1">
                                <svg
                                    class="group-hover:fill-accent-800 h-5 w-5 animate-spin fill-zinc-300"
                                    xmlns="http://www.w3.org/2000/svg"
                                    viewBox="0 -960 960 960">
                                    <path
                                        xmlns="http://www.w3.org/2000/svg"
                                        d="M160-160v-80h110l-16-14q-52-46-73-105t-21-119q0-111 66.5-197.5T400-790v84q-72 26-116 88.5T240-478q0 45 17 87.5t53 78.5l10 10v-98h80v240H160Zm400-10v-84q72-26 116-88.5T720-482q0-45-17-87.5T650-648l-10-10v98h-80v-240h240v80H690l16 14q49 49 71.5 106.5T800-482q0 111-66.5 197.5T560-170Z" />
                                </svg>
                                <div class="group-hover:text-accent-800">Uploading...</div>
                            </div>
                        </div>
                    {:else if !encounter.sync}
                        <button
                            class="rounded-xs bg-gray-700 px-2 py-1"
                            aria-label="Sync to logs.snow.xyz"
                            use:tooltip={{ content: "Sync to logs.snow.xyz" }}
                            onclick={upload}>
                            <div class="group flex space-x-1">
                                <svg
                                    class="group-hover:fill-accent-800 h-5 w-5 fill-zinc-300"
                                    xmlns="http://www.w3.org/2000/svg"
                                    viewBox="0 -960 960 960">
                                    <path
                                        xmlns="http://www.w3.org/2000/svg"
                                        d="M450-313v-371L330-564l-43-43 193-193 193 193-43 43-120-120v371h-60ZM220-160q-24 0-42-18t-18-42v-143h60v143h520v-143h60v143q0 24-18 42t-42 18H220Z" />
                                </svg>
                                <div class="group-hover:text-accent-800">Upload</div>
                            </div>
                        </button>
                    {:else}
                        <a
                            class="rounded-xs bg-gray-700 px-2 py-1"
                            aria-label="Open on logs.snow.xyz"
                            use:tooltip={{ content: "Open on logs.snow.xyz" }}
                            href={LOG_SITE_URL + "/logs/" + encounter.sync}
                            target="_blank">
                            <div class="group flex space-x-1">
                                <svg
                                    class="group-hover:fill-accent-800 h-5 w-5 fill-zinc-300"
                                    xmlns="http://www.w3.org/2000/svg"
                                    viewBox="0 -960 960 960">
                                    <path
                                        xmlns="http://www.w3.org/2000/svg"
                                        d="m414-280 226-226-58-58-169 169-84-84-57 57 142 142ZM260-160q-91 0-155.5-63T40-377q0-78 47-139t123-78q25-92 100-149t170-57q117 0 198.5 81.5T760-520q69 8 114.5 59.5T920-340q0 75-52.5 127.5T740-160H260Zm0-80h480q42 0 71-29t29-71q0-42-29-71t-71-29h-60v-80q0-83-58.5-141.5T480-720q-83 0-141.5 58.5T280-520h-20q-58 0-99 41t-41 99q0 58 41 99t99 41Zm220-240Z" />
                                </svg>
                                <div class="group-hover:text-accent-800">Share Log</div>
                            </div>
                        </a>
                    {/if}
                {/if}
                <div class="relative flex items-center rounded-xs bg-gray-700" onfocusout={handleDropdownFocusLoss}>
                    <button onclick={handleDropdownClick} class="h-full px-2" aria-label="Settings">
                        <svg
                            class="h-4 w-4"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            xmlns="http://www.w3.org/2000/svg">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                        </svg>
                    </button>
                    {#if dropdownOpen}
                        <div class="absolute top-0 left-9 z-50 rounded-md bg-gray-700">
                            <div class="flex w-48 flex-col divide-y-2 divide-gray-600 px-2 py-1">
                                <button
                                    class="hover:text-accent-500 p-1 text-left"
                                    onclick={() => {
                                        dropdownOpen = false;
                                        captureScreenshot();
                                    }}>
                                    Take Screenshot
                                </button>
                                <button class="flex items-center justify-between bg-gray-700 p-1">
                                    <span class="text-sm">Show Names</span>
                                    <label class="relative inline-flex cursor-pointer items-center">
                                        <input
                                            type="checkbox"
                                            value=""
                                            class="peer sr-only"
                                            bind:checked={$settings.general.showNames} />
                                        <div
                                            class="peer-checked:bg-accent-800 peer h-5 w-9 rounded-full border-gray-600 bg-gray-800 peer-focus:outline-hidden after:absolute after:top-[2px] after:left-[2px] after:h-4 after:w-4 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white">
                                        </div>
                                    </label>
                                </button>
                                <button class="flex items-center justify-between bg-gray-700 p-1">
                                    <span class="text-sm">Split Party Damage</span>
                                    <label class="relative inline-flex cursor-pointer items-center">
                                        <input
                                            type="checkbox"
                                            value=""
                                            class="peer sr-only"
                                            bind:checked={$settings.logs.splitPartyDamage} />
                                        <div
                                            class="peer-checked:bg-accent-800 peer h-5 w-9 rounded-full border-gray-600 bg-gray-800 peer-focus:outline-hidden after:absolute after:top-[2px] after:left-[2px] after:h-4 after:w-4 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white">
                                        </div>
                                    </label>
                                </button>
                                <button class="flex items-center justify-between bg-gray-700 p-1">
                                    <span class="text-sm">Show Esther</span>
                                    <label class="relative inline-flex cursor-pointer items-center">
                                        <input
                                            type="checkbox"
                                            value=""
                                            class="peer sr-only"
                                            bind:checked={$settings.general.showEsther} />
                                        <div
                                            class="peer-checked:bg-accent-800 peer h-5 w-9 rounded-full border-gray-600 bg-gray-800 peer-focus:outline-hidden after:absolute after:top-[2px] after:left-[2px] after:h-4 after:w-4 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white">
                                        </div>
                                    </label>
                                </button>
                                <button
                                    class="p-1 text-left hover:text-red-600"
                                    onclick={() => {
                                        dropdownOpen = false;
                                        deleteConfirm = true;
                                    }}>
                                    Delete
                                </button>
                            </div>
                        </div>
                    {/if}
                </div>
            </div>

            {#if deleteConfirm}
                <div class="bg-opacity-80 fixed inset-0 z-50 bg-zinc-900"></div>
                <div class="h-modal fixed top-0 right-0 left-0 z-50 w-full items-center justify-center p-4">
                    <div class="relative top-[25%] mx-auto flex max-h-full w-full max-w-md">
                        <div
                            class="relative mx-auto flex flex-col rounded-lg border-gray-700 bg-zinc-800 text-gray-400 shadow-md">
                            <button
                                type="button"
                                class="absolute top-3 right-2.5 ml-auto rounded-lg p-1.5 whitespace-normal hover:bg-zinc-600 focus:outline-hidden"
                                aria-label="Close modal"
                                onclick={() => (deleteConfirm = false)}>
                                <span class="sr-only">Close modal</span>
                                <svg
                                    class="h-5 w-5"
                                    fill="currentColor"
                                    viewBox="0 0 20 20"
                                    xmlns="http://www.w3.org/2000/svg">
                                    <path
                                        fill-rule="evenodd"
                                        d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                                        clip-rule="evenodd" />
                                </svg>
                            </button>
                            <div id="modal" class="flex-1 space-y-6 overflow-y-auto overscroll-contain p-6">
                                <div class="text-center">
                                    <svg
                                        aria-hidden="true"
                                        class="mx-auto mb-4 h-14 w-14 text-gray-200"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                        xmlns="http://www.w3.org/2000/svg">
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                            class="s-Qbr4I8QhaoSZ" />
                                    </svg>
                                    <h3 class="mb-5 text-lg font-normal text-gray-400">
                                        Are you sure you want to delete this encounter?
                                    </h3>
                                    <button
                                        type="button"
                                        class="mr-2 inline-flex items-center justify-center rounded-lg bg-red-700 px-5 py-2.5 text-center text-sm font-medium text-white hover:bg-red-800 focus:outline-hidden"
                                        onclick={deleteEncounter}>
                                        Yes, I'm sure
                                    </button>
                                    <button
                                        type="button"
                                        class="inline-flex items-center justify-center rounded-lg bg-gray-800 bg-transparent px-5 py-2.5 text-center text-sm font-medium text-gray-400 hover:bg-zinc-700 hover:text-white focus:text-white focus:outline-hidden"
                                        onclick={() => (deleteConfirm = false)}>
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
    {#if tab === MeterTab.IDENTITY && localPlayerEntity !== null}
        <LogIdentity localPlayer={localPlayerEntity} duration={encounter.duration} />
    {:else if tab === MeterTab.STAGGER && encounter.encounterDamageStats.staggerStats}
        <LogStagger staggerStats={encounter.encounterDamageStats.staggerStats} />
    {:else}
        <div class="px relative top-0 overflow-x-auto overflow-y-visible">
            {#if tab === MeterTab.DAMAGE}
                {#if meterState === MeterState.PARTY}
                    {#if $settings.logs.splitPartyDamage && encounterPartyInfo && Object.keys(encounterPartyInfo).length >= 2}
                        <LogDamageMeterPartySplit {enc} {inspectPlayer} />
                    {:else}
                        <table class="relative w-full table-fixed">
                            <thead class="z-30 h-6">
                                <tr class="bg-zinc-900">
                                    <th class="w-7 px-2 font-normal"></th>
                                    <DamageMeterHeader {enc} />
                                </tr>
                            </thead>
                            <tbody class="relative z-10">
                                {#each enc.players as player, i (player.name)}
                                    <tr
                                        class="h-7 px-2 py-1 {$settings.general.underlineHovered
                                            ? 'hover:underline'
                                            : ''}"
                                        onclick={() => inspectPlayer(player.name)}>
                                        <PlayerRow {enc} entity={player} width={enc.playerDamagePercentages[i]} />
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    {/if}
                {:else if meterState === MeterState.PLAYER && player !== null}
                    <table class="relative w-full table-fixed">
                        <LogPlayerBreakdown entity={player} {enc} />
                    </table>
                    {#if player.class === "Arcanist"}
                        <table class="relative w-full table-fixed">
                            <ArcanistCardTable {player} duration={encounter.duration} />
                        </table>
                    {/if}
                {/if}
            {:else if tab === MeterTab.PARTY_BUFFS || tab === MeterTab.SELF_BUFFS}
                <Buffs {tab} {enc} focusedPlayer={player} {inspectPlayer} {handleRightClick} />
            {:else if tab === MeterTab.TANK}
                <DamageTaken {enc} />
            {:else if tab === MeterTab.SHIELDS}
                <LogShields {enc} />
            {:else if tab === MeterTab.BOSS}
                {#if !focusedBoss}
                    <BossTable {enc} {inspectBoss} />
                {:else}
                    <BossBreakdown
                        {enc}
                        boss={encounter.entities[focusedBoss]}
                        handleRightClick={() => {
                            focusedBoss = "";
                        }} />
                {/if}
            {/if}
        </div>
    {/if}
</div>
{#if tab !== MeterTab.IDENTITY && tab !== MeterTab.STAGGER}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="mt-4" oncontextmenu={handleRightClick}>
        {#if chartType === ChartType.SKILL_LOG}
            {#if player && player.entityType === EntityType.PLAYER}
                <OpenerSkills skills={player.skills} />
            {/if}
        {/if}
        {#if player?.entityType !== EntityType.ESTHER}
            <div class="text-lg font-medium">Charts</div>
            <div class="mt-2 flex divide-x divide-gray-600">
                {#if playerName === "" && meterState === MeterState.PARTY}
                    <button
                        class="rounded-xs px-2 py-1"
                        class:bg-accent-900={chartType === ChartType.AVERAGE_DPS}
                        class:bg-gray-700={chartType !== ChartType.AVERAGE_DPS}
                        onclick={() => (chartType = ChartType.AVERAGE_DPS)}>
                        Average DPS
                    </button>
                    <button
                        class="rounded-xs px-2 py-1"
                        class:bg-accent-900={chartType === ChartType.ROLLING_DPS}
                        class:bg-gray-700={chartType !== ChartType.ROLLING_DPS}
                        onclick={() => (chartType = ChartType.ROLLING_DPS)}>
                        10s DPS Window
                    </button>
                    {#if enc.anySkillCastLog}
                        {#if enc.anySupportBuff}
                            <button
                                class="rounded-sm px-2 py-1"
                                class:bg-accent-900={chartType === ChartType.AP_BUFF}
                                class:bg-gray-700={chartType !== ChartType.AP_BUFF}
                                onclick={() => (chartType = ChartType.AP_BUFF)}>
                                AP Buffs
                            </button>
                        {/if}
                        {#if enc.anySupportBrand}
                            <button
                                class="rounded-sm px-2 py-1"
                                class:bg-accent-900={chartType === ChartType.BRAND_BUFF}
                                class:bg-gray-700={chartType !== ChartType.BRAND_BUFF}
                                onclick={() => (chartType = ChartType.BRAND_BUFF)}>
                                Brand
                            </button>
                        {/if}
                        {#if enc.anySupportIdentity}
                            <button
                                class="rounded-sm px-2 py-1"
                                class:bg-accent-900={chartType === ChartType.IDENTITY_BUFF}
                                class:bg-gray-700={chartType !== ChartType.IDENTITY_BUFF}
                                onclick={() => (chartType = ChartType.IDENTITY_BUFF)}>
                                Identity
                            </button>
                        {/if}
                        {#if enc.anySupportHat}
                            <button
                                class="rounded-sm px-2 py-1"
                                class:bg-accent-900={chartType === ChartType.HAT_BUFF}
                                class:bg-gray-700={chartType !== ChartType.HAT_BUFF}
                                onclick={() => (chartType = ChartType.HAT_BUFF)}>
                                H.A Skill
                            </button>
                        {/if}
                    {/if}
                {:else if playerName !== "" && meterState === MeterState.PLAYER}
                    <!--  -->
                {/if}
            </div>
        {/if}
        {#if chartType === ChartType.AVERAGE_DPS}
            {#if !$settings.general.showNames}
                <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);"></div>
            {:else}
                <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);"></div>
            {/if}
        {:else if chartType === ChartType.ROLLING_DPS}
            {#if !$settings.general.showNames}
                <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);"></div>
            {:else}
                <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);"></div>
            {/if}
        {:else if chartType === ChartType.BRAND_BUFF}
            <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);"></div>
        {:else if chartType === ChartType.AP_BUFF}
            <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);"></div>
        {:else if chartType === ChartType.IDENTITY_BUFF}
            <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);"></div>
        {:else if chartType === ChartType.HAT_BUFF}
            <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);"></div>
        {:else if chartType === ChartType.SKILL_LOG}
            {#if player && player.entityType === EntityType.PLAYER && hasSkillCastLog}
                <LogSkillChart {chartOptions} {player} encounterDamageStats={encounter.encounterDamageStats} />
            {:else if (player && player.entityType === EntityType.PLAYER) || focusedBoss}
                <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);"></div>
            {/if}
        {/if}
    </div>
{/if}
{#if $uploadErrorStore}
    <Notification
        bind:showAlert={$uploadErrorStore}
        text={$uploadErrorMessage}
        dismissable={true}
        width="20rem"
        fixed={true}
        isError={true} />
{/if}
