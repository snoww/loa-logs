<script lang="ts">
    import {
        MeterState,
        MeterTab,
        type Entity,
        type Encounter,
        ChartType,
        EntityType,
        type PartyInfo
    } from "$lib/types";
    import { formatTimestampDate, millisToMinutesAndSeconds } from "$lib/utils/numbers";
    import { invoke } from "@tauri-apps/api/tauri";
    import LogDamageMeterRow from "./LogDamageMeterRow.svelte";
    import LogPlayerBreakdown from "./LogPlayerBreakdown.svelte";
    import LogEncounterInfo from "./LogEncounterInfo.svelte";
    import LogBuffs from "./LogBuffs.svelte";
    import { page } from "$app/state";
    import { chartable, type EChartsOptions } from "$lib/utils/charts";
    import { colors, settings, skillIcon } from "$lib/utils/settings";
    import { goto } from "$app/navigation";
    import html2canvas from "html2canvas";
    import {
        screenshotAlert,
        screenshotError,
        takingScreenshot,
        raidGates,
        localPlayer,
        rdpsEventDetails,
        uploadErrorStore,
        uploadErrorMessage
    } from "$lib/utils/stores";
    import LogIdentity from "./identity/LogIdentity.svelte";
    import LogStagger from "./stagger/LogStagger.svelte";
    import { tooltip } from "$lib/utils/tooltip";
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
    import OpenerSkills from "./OpenerSkills.svelte";
    import ArcanistCardTable from "../shared/ArcanistCardTable.svelte";
    import DamageTaken from "../shared/DamageTaken.svelte";
    import BossTable from "../shared/BossTable.svelte";
    import BossBreakdown from "../shared/BossBreakdown.svelte";
    import LogShields from "$lib/components/logs/LogShields.svelte";
    import Rdps from "$lib/components/shared/Rdps.svelte";
    import LogSkillChart from "./LogSkillChart.svelte";
    import LogDamageMeterPartySplit from "./LogDamageMeterPartySplit.svelte";
    import LogDamageMeterHeader from "./LogDamageMeterHeader.svelte";
    import { LOG_SITE_URL, uploadLog } from "$lib/utils/sync";
    import Notification from "$lib/components/shared/Notification.svelte";

    interface Props {
        id: string;
        encounter: Encounter;
    }

    let { id, encounter = $bindable() }: Props = $props();

    let players: Array<Entity> = $derived.by(() => {
        if ($settings.general.showEsther) {
            return Object.values(encounter.entities)
                .filter(
                    (e) =>
                        e.damageStats.damageDealt > 0 &&
                        (e.entityType === EntityType.ESTHER || (e.entityType === EntityType.PLAYER && e.classId != 0))
                )
                .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
        } else {
            return Object.values(encounter.entities)
                .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.PLAYER && e.classId != 0)
                .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
        }
    });
    let bosses: Array<Entity> = $state([]);
    let player: Entity | null = $state(null);
    let totalDamageDealt = $state(0);
    let anyRdpsData: boolean = $state(false);

    let hasSkillCastLog = $state(false);

    let deleteConfirm = $state(false);

    let encounterPartyInfo: PartyInfo | undefined = $state(encounter.encounterDamageStats.misc?.partyInfo);

    $effect(() => {
        if ($settings.general.showBosses) {
            bosses = Object.values(encounter.entities)
                .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.BOSS)
                .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
        }
        $localPlayer = encounter.localPlayer;
        if (!anyDead) {
            multipleDeaths = players.some((player) => player.damageStats.deaths > 0);
        } else {
            multipleDeaths = players.some((player) => player.damageStats.deaths > 1);
        }

        // if (
        //     encounter.encounterDamageStats.misc?.rdpsValid === undefined ||
        //     encounter.encounterDamageStats.misc?.rdpsValid
        // ) {
        //     anyRdpsData = players.some((player) => player.damageStats.rdpsDamageReceived > 0);
        // }
        // if (
        //     encounter.encounterDamageStats.misc?.rdpsMessage === undefined ||
        //     encounter.encounterDamageStats.misc?.rdpsMessage
        // ) {
        //     $rdpsEventDetails = encounter.encounterDamageStats.misc?.rdpsMessage || "";
        // } else {
        //     $rdpsEventDetails = "";
        // }
        if ($settings.general.showEsther) {
            totalDamageDealt =
                encounter.encounterDamageStats.totalDamageDealt +
                players
                    .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.ESTHER)
                    .reduce((a, b) => a + b.damageStats.damageDealt, 0);
        } else {
            totalDamageDealt = encounter.encounterDamageStats.totalDamageDealt;
        }

        if (encounter.localPlayer) {
            localPlayerEntity = encounter.entities[encounter.localPlayer];
        }

        if (playerName) {
            player = encounter.entities[playerName];
            meterState = MeterState.PLAYER;
        } else {
            player = null;
            meterState = MeterState.PARTY;
        }
    });

    let playerDamagePercentages: Array<number> = $derived(
        players.map((player) => (player.damageStats.damageDealt / topDamageDealt) * 100)
    );
    let topDamageDealt = $derived(encounter.encounterDamageStats.topDamageDealt);
    let localPlayerEntity: Entity | null = $state(null);

    let anyDead: boolean = $derived(players.some((player) => player.isDead));
    let multipleDeaths: boolean = $state(false);
    let anyFrontAtk: boolean = $derived(players.some((player) => player.skillStats.frontAttacks > 0));
    let anyBackAtk: boolean = $derived(players.some((player) => player.skillStats.backAttacks > 0));
    let anySupportBuff: boolean = $derived(players.some((player) => player.damageStats.buffedBySupport > 0));
    let anySupportIdentity: boolean = $derived(players.some((player) => player.damageStats.buffedByIdentity > 0));
    let anySupportBrand: boolean = $derived(players.some((player) => player.damageStats.debuffedBySupport > 0));
    let isSolo = $derived(players.length === 1);

    let meterState = $state(MeterState.PARTY);
    let tab = $state(MeterTab.DAMAGE);
    let chartType = $state(ChartType.AVERAGE_DPS);
    let playerName = $state("");
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
                let boss = bosses.find((boss) => boss.name === focusedBoss);
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

    function damageTab() {
        tab = MeterTab.DAMAGE;
        setChartView();
    }

    // function RDPSTab() {
    //     tab = MeterTab.RDPS;
    //     setChartView();
    // }

    function partySynergyTab() {
        tab = MeterTab.PARTY_BUFFS;
        setChartView();
    }

    function selfSynergyTab() {
        tab = MeterTab.SELF_BUFFS;
        setChartView();
    }

    function tankTab() {
        handleRightClick();
        tab = MeterTab.TANK;
        setChartView();
    }

    function shieldTab() {
        handleRightClick();
        tab = MeterTab.SHIELDS;
        setChartView();
    }

    function bossTab() {
        handleRightClick();
        tab = MeterTab.BOSS;
        setChartView();
    }

    function identityTab() {
        if (!localPlayerEntity) return;
        tab = MeterTab.IDENTITY;
        chartType = ChartType.IDENTITY;
    }

    function staggerTab() {
        tab = MeterTab.STAGGER;
        chartType = ChartType.STAGGER;
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
            player = null;
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

            const canvas = await html2canvas(targetDiv, {
                useCORS: true,
                backgroundColor: "#27272A"
            });

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

<svelte:window oncontextmenu={(e) => e.preventDefault()} />
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    bind:this={targetDiv}
    class="scroll-ml-8 scroll-mt-2 text-gray-100"
    class:p-4={$takingScreenshot}
    oncontextmenu={handleRightClick}>
    <LogEncounterInfo
        boss={encounter.entities[encounter.currentBossName]}
        difficulty={encounter.difficulty}
        date={formatTimestampDate(encounter.fightStart, true)}
        encounterDuration={millisToMinutesAndSeconds(encounter.duration)}
        {totalDamageDealt}
        dps={encounter.encounterDamageStats.dps}
        cleared={encounter.cleared}
        bossOnlyDamage={encounter.bossOnlyDamage}
        raidGate={$raidGates.get(encounter.currentBossName)} />
    {#if !$takingScreenshot}
        <div class="mt-2 flex justify-between" style="width: calc(100vw - 4.5rem);">
            <div class="flex divide-x divide-gray-600">
                <button
                    class="rounded-sm px-2 py-1"
                    class:bg-accent-900={tab === MeterTab.DAMAGE}
                    class:bg-gray-700={tab !== MeterTab.DAMAGE}
                    onclick={damageTab}>
                    Damage
                </button>
                <!--{#if anyRdpsData || $rdpsEventDetails !== ""}-->
                <!--    <button-->
                <!--        class="flex-shrink-0 rounded-sm px-3 py-1"-->
                <!--        class:bg-accent-900={tab === MeterTab.RDPS}-->
                <!--        class:bg-gray-700={tab !== MeterTab.RDPS}-->
                <!--        on:click={RDPSTab}>-->
                <!--        RDPS-->
                <!--    </button>-->
                <!--{/if}-->
                <button
                    class="flex-shrink-0 rounded-sm px-2 py-1"
                    class:bg-accent-900={tab === MeterTab.PARTY_BUFFS}
                    class:bg-gray-700={tab !== MeterTab.PARTY_BUFFS}
                    onclick={partySynergyTab}>
                    Party Buffs
                </button>
                <button
                    class="flex-shrink-0 rounded-sm px-2 py-1"
                    class:bg-accent-900={tab === MeterTab.SELF_BUFFS}
                    class:bg-gray-700={tab !== MeterTab.SELF_BUFFS}
                    onclick={selfSynergyTab}>
                    Self Buffs
                </button>
                {#if $settings.general.showShields && encounter.encounterDamageStats.totalShielding > 0}
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={tab === MeterTab.SHIELDS}
                        class:bg-gray-700={tab !== MeterTab.SHIELDS}
                        onclick={shieldTab}>
                        Shields
                    </button>
                {/if}
                {#if $settings.general.showTanked && encounter.encounterDamageStats.totalDamageTaken > 0}
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={tab === MeterTab.TANK}
                        class:bg-gray-700={tab !== MeterTab.TANK}
                        onclick={tankTab}>
                        Tanked
                    </button>
                {/if}
                {#if $settings.general.showBosses && bosses.length > 0}
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={tab === MeterTab.BOSS}
                        class:bg-gray-700={tab !== MeterTab.BOSS}
                        onclick={bossTab}>
                        Bosses
                    </button>
                {/if}
                {#if localPlayerEntity && localPlayerEntity.skillStats.identityStats}
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={tab === MeterTab.IDENTITY}
                        class:bg-gray-700={tab !== MeterTab.IDENTITY}
                        onclick={identityTab}>
                        Identity
                    </button>
                {/if}
                {#if encounter.encounterDamageStats.staggerStats}
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={tab === MeterTab.STAGGER}
                        class:bg-gray-700={tab !== MeterTab.STAGGER}
                        onclick={staggerTab}>
                        Stagger
                    </button>
                {/if}
                <button
                    class="rounded-sm bg-gray-700 px-2 py-1"
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
                {#if encounter.cleared && $settings.sync.enabled && $settings.sync.accessToken && $settings.sync.validToken}
                    {#if uploading}
                        <div class="rounded-sm bg-gray-700 px-2 py-1" use:tooltip={{ content: "Uploading..." }}>
                            <svg
                                class="hover:fill-accent-800 h-5 w-5 animate-spin fill-zinc-300"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 -960 960 960">
                                <path
                                    xmlns="http://www.w3.org/2000/svg"
                                    d="M160-160v-80h110l-16-14q-52-46-73-105t-21-119q0-111 66.5-197.5T400-790v84q-72 26-116 88.5T240-478q0 45 17 87.5t53 78.5l10 10v-98h80v240H160Zm400-10v-84q72-26 116-88.5T720-482q0-45-17-87.5T650-648l-10-10v98h-80v-240h240v80H690l16 14q49 49 71.5 106.5T800-482q0 111-66.5 197.5T560-170Z" />
                            </svg>
                        </div>
                    {:else if !encounter.sync}
                        <button
                            class="rounded-sm bg-gray-700 px-2 py-1"
                            aria-label="Sync to logs.snow.xyz"
                            use:tooltip={{ content: "Sync to logs.snow.xyz" }}
                            onclick={upload}>
                            <svg
                                class="hover:fill-accent-800 h-5 w-5 fill-zinc-300"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 -960 960 960">
                                <path
                                    xmlns="http://www.w3.org/2000/svg"
                                    d="M450-313v-371L330-564l-43-43 193-193 193 193-43 43-120-120v371h-60ZM220-160q-24 0-42-18t-18-42v-143h60v143h520v-143h60v143q0 24-18 42t-42 18H220Z" />
                            </svg>
                        </button>
                    {:else}
                        <a
                            class="rounded-sm bg-gray-700 px-2 py-1"
                            aria-label="Open on logs.snow.xyz"
                            use:tooltip={{ content: "Open on logs.snow.xyz" }}
                            href={LOG_SITE_URL + "/logs/" + encounter.sync}
                            target="_blank">
                            <svg
                                class="hover:fill-accent-800 h-5 w-5 fill-zinc-300"
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 -960 960 960">
                                <path
                                    xmlns="http://www.w3.org/2000/svg"
                                    d="m414-280 226-226-58-58-169 169-84-84-57 57 142 142ZM260-160q-91 0-155.5-63T40-377q0-78 47-139t123-78q25-92 100-149t170-57q117 0 198.5 81.5T760-520q69 8 114.5 59.5T920-340q0 75-52.5 127.5T740-160H260Zm0-80h480q42 0 71-29t29-71q0-42-29-71t-71-29h-60v-80q0-83-58.5-141.5T480-720q-83 0-141.5 58.5T280-520h-20q-58 0-99 41t-41 99q0 58 41 99t99 41Zm220-240Z" />
                            </svg>
                        </a>
                    {/if}
                {/if}
                <div class="relative flex items-center rounded-sm bg-gray-700" onfocusout={handleDropdownFocusLoss}>
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
                        <div class="absolute left-9 top-0 z-50 rounded-md bg-gray-700">
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
                                            class="peer-checked:bg-accent-800 peer h-5 w-9 rounded-full border-gray-600 bg-gray-800 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white peer-focus:outline-none">
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
                                            class="peer-checked:bg-accent-800 peer h-5 w-9 rounded-full border-gray-600 bg-gray-800 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white peer-focus:outline-none">
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
                                            class="peer-checked:bg-accent-800 peer h-5 w-9 rounded-full border-gray-600 bg-gray-800 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white peer-focus:outline-none">
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
                <div class="fixed inset-0 z-50 bg-zinc-900 bg-opacity-80"></div>
                <div class="fixed left-0 right-0 top-0 z-50 h-modal w-full items-center justify-center p-4">
                    <div class="relative top-[25%] mx-auto flex max-h-full w-full max-w-md">
                        <div
                            class="relative mx-auto flex flex-col rounded-lg border-gray-700 bg-zinc-800 text-gray-400 shadow-md">
                            <button
                                type="button"
                                class="absolute right-2.5 top-3 ml-auto whitespace-normal rounded-lg p-1.5 hover:bg-zinc-600 focus:outline-none"
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
                                        class="mr-2 inline-flex items-center justify-center rounded-lg bg-red-700 px-5 py-2.5 text-center text-sm font-medium text-white hover:bg-red-800 focus:outline-none"
                                        onclick={deleteEncounter}>
                                        Yes, I'm sure
                                    </button>
                                    <button
                                        type="button"
                                        class="inline-flex items-center justify-center rounded-lg bg-gray-800 bg-transparent px-5 py-2.5 text-center text-sm font-medium text-gray-400 hover:bg-zinc-700 hover:text-white focus:text-white focus:outline-none"
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
                        <LogDamageMeterPartySplit
                            {players}
                            {encounterPartyInfo}
                            {topDamageDealt}
                            {totalDamageDealt}
                            {anyFrontAtk}
                            {anyBackAtk}
                            {anySupportBuff}
                            {anySupportIdentity}
                            {anySupportBrand}
                            {anyRdpsData}
                            end={encounter.lastCombatPacket}
                            {isSolo}
                            {inspectPlayer} />
                    {:else}
                        <table class="relative w-full table-fixed">
                            <thead class="z-30 h-6">
                                <tr class="bg-zinc-900">
                                    <th class="w-7 px-2 font-normal"></th>
                                    <th class="w-14 px-2 text-left font-normal"></th>
                                    <th class="w-full"></th>
                                    <LogDamageMeterHeader
                                        {anyDead}
                                        {multipleDeaths}
                                        {anyFrontAtk}
                                        {anyBackAtk}
                                        {anySupportBuff}
                                        {anySupportIdentity}
                                        {anySupportBrand}
                                        {anyRdpsData}
                                        {isSolo} />
                                </tr>
                            </thead>
                            <tbody class="relative z-10">
                                {#each players as player, i (player.name)}
                                    <tr
                                        class="h-7 px-2 py-1 {$settings.general.underlineHovered
                                            ? 'hover:underline'
                                            : ''}"
                                        onclick={() => inspectPlayer(player.name)}>
                                        <LogDamageMeterRow
                                            entity={player}
                                            percentage={playerDamagePercentages[i]}
                                            {totalDamageDealt}
                                            {anyDead}
                                            {multipleDeaths}
                                            {anyFrontAtk}
                                            {anyBackAtk}
                                            {anySupportBuff}
                                            {anySupportIdentity}
                                            {anySupportBrand}
                                            {anyRdpsData}
                                            end={encounter.lastCombatPacket}
                                            {isSolo} />
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    {/if}
                {:else if meterState === MeterState.PLAYER && player !== null}
                    <table class="relative w-full table-fixed">
                        <LogPlayerBreakdown entity={player} duration={encounter.duration} {totalDamageDealt} />
                    </table>
                    {#if player.class === "Arcanist"}
                        <table class="relative w-full table-fixed">
                            <ArcanistCardTable {player} duration={encounter.duration} />
                        </table>
                    {/if}
                {/if}
            {:else if tab === MeterTab.RDPS}
                <Rdps
                    meterSettings={$settings.logs}
                    {players}
                    totalDamageDealt={encounter.encounterDamageStats.totalDamageDealt}
                    duration={encounter.duration}
                    {encounterPartyInfo} />
            {:else if tab === MeterTab.PARTY_BUFFS}
                {#if meterState === MeterState.PARTY}
                    <LogBuffs {tab} encounterDamageStats={encounter.encounterDamageStats} {players} {inspectPlayer} />
                {:else}
                    <LogBuffs
                        {tab}
                        encounterDamageStats={encounter.encounterDamageStats}
                        {players}
                        focusedPlayer={player}
                        {inspectPlayer} />
                {/if}
            {:else if tab === MeterTab.SELF_BUFFS}
                {#if meterState === MeterState.PARTY}
                    <LogBuffs {tab} encounterDamageStats={encounter.encounterDamageStats} {players} {inspectPlayer} />
                {:else}
                    <LogBuffs
                        {tab}
                        encounterDamageStats={encounter.encounterDamageStats}
                        {players}
                        focusedPlayer={player}
                        {inspectPlayer} />
                {/if}
            {:else if tab === MeterTab.TANK}
                <DamageTaken {players} topDamageTaken={encounter.encounterDamageStats.topDamageTaken} tween={false} />
            {:else if tab === MeterTab.SHIELDS}
                <LogShields {players} encounterDamageStats={encounter.encounterDamageStats} />
            {:else if tab === MeterTab.BOSS}
                {#if !focusedBoss}
                    <BossTable {bosses} duration={encounter.duration} {inspectBoss} tween={false} />
                {:else}
                    <BossBreakdown
                        boss={encounter.entities[focusedBoss]}
                        duration={encounter.duration}
                        handleRightClick={() => {
                            focusedBoss = "";
                        }}
                        tween={false} />
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
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={chartType === ChartType.AVERAGE_DPS}
                        class:bg-gray-700={chartType !== ChartType.AVERAGE_DPS}
                        onclick={() => (chartType = ChartType.AVERAGE_DPS)}>
                        Average DPS
                    </button>
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={chartType === ChartType.ROLLING_DPS}
                        class:bg-gray-700={chartType !== ChartType.ROLLING_DPS}
                        onclick={() => (chartType = ChartType.ROLLING_DPS)}>
                        10s DPS Window
                    </button>
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
