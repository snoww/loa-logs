<script lang="ts">
    import { MeterState, MeterTab, type Entity, type Encounter, ChartType, EntityType } from "$lib/types";
    import { formatTimestampDate, millisToMinutesAndSeconds } from "$lib/utils/numbers";
    import { invoke } from "@tauri-apps/api/tauri";
    import LogDamageMeterRow from "./LogDamageMeterRow.svelte";
    import LogPlayerBreakdown from "./LogPlayerBreakdown.svelte";
    import LogEncounterInfo from "./LogEncounterInfo.svelte";
    import LogBuffs from "./LogBuffs.svelte";
    import { page } from "$app/stores";
    import { chartable, type EChartsOptions } from "$lib/utils/charts";
    import { colors, settings, skillIcon } from "$lib/utils/settings";
    import { goto } from "$app/navigation";
    import html2canvas from "html2canvas";
    import { screenshotAlert, screenshotError, takingScreenshot, raidGates } from "$lib/utils/stores";
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
        getSkillLogChart
    } from "$lib/utils/dpsCharts";
    import OpenerSkills from "./OpenerSkills.svelte";
    import ArcanistCardTable from "../shared/ArcanistCardTable.svelte";
    import DamageTaken from "../shared/DamageTaken.svelte";
    import BossTable from "../shared/BossTable.svelte";
    import BossBreakdown from "../shared/BossBreakdown.svelte";

    export let id: string;
    export let encounter: Encounter;

    let players: Array<Entity> = [];
    let bosses: Array<Entity> = [];
    let player: Entity | null = null;
    let playerDamagePercentages: Array<number> = [];
    let topDamageDealt = 0;
    let totalDamageDealt = 0;
    let localPlayer: Entity | null = null;

    let anyDead: boolean;
    let anyFrontAtk: boolean = false;
    let anyBackAtk: boolean = false;
    let anySupportBuff: boolean = false;
    let anySupportIdentity: boolean = false;
    let anySupportBrand: boolean = false;

    let isSolo = true;

    let state = MeterState.PARTY;
    let tab = MeterTab.DAMAGE;
    let chartType = ChartType.AVERAGE_DPS;
    let playerName = "";
    let focusedBoss = "";

    let deleteConfirm = false;

    let chartOptions: EChartsOptions = {};

    $: {
        if (encounter) {
            if ($settings.general.showEsther) {
                players = Object.values(encounter.entities)
                    .filter(
                        (e) =>
                            e.damageStats.damageDealt > 0 &&
                            (e.entityType === EntityType.ESTHER ||
                                (e.entityType === EntityType.PLAYER && e.classId != 0))
                    )
                    .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
            } else {
                players = Object.values(encounter.entities)
                    .filter(
                        (e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.PLAYER && e.classId != 0
                    )
                    .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
            }
            if ($settings.general.showBosses) {
                bosses = Object.values(encounter.entities)
                    .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.BOSS)
                    .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
            }
            isSolo = players.length === 1;
            topDamageDealt = encounter.encounterDamageStats.topDamageDealt;
            playerDamagePercentages = players.map((player) => (player.damageStats.damageDealt / topDamageDealt) * 100);
            anyDead = players.some((player) => player.isDead);
            anyFrontAtk = players.some((player) => player.skillStats.frontAttacks > 0);
            anyBackAtk = players.some((player) => player.skillStats.backAttacks > 0);
            anySupportBuff = players.some((player) => player.damageStats.buffedBySupport > 0);
            anySupportIdentity = players.some((player) => player.damageStats.buffedByIdentity > 0);
            anySupportBrand = players.some((player) => player.damageStats.debuffedBySupport > 0);
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
                localPlayer = encounter.entities[encounter.localPlayer];
            }

            if (playerName) {
                player = encounter.entities[playerName];
                state = MeterState.PLAYER;
            } else {
                player = null;
                state = MeterState.PARTY;
            }

            let chartablePlayers = Object.values(encounter.entities)
                .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.PLAYER && e.classId != 0)
                .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);

            if (
                chartablePlayers.length > 0 &&
                chartablePlayers[0].damageStats &&
                chartablePlayers[0].damageStats.dpsAverage.length > 0 &&
                chartablePlayers[0].damageStats.dpsRolling10sAvg.length > 0
            ) {
                let legendNames = getLegendNames(chartablePlayers, $settings.general.showNames);
                let deathTimes = getDeathTimes(chartablePlayers, legendNames, encounter.fightStart);
                let bossHpLogs = Object.entries(encounter.encounterDamageStats.misc?.bossHpLog || {});
                if (chartType === ChartType.AVERAGE_DPS) {
                    let chartPlayers = getAveragePlayerSeries(
                        chartablePlayers,
                        legendNames,
                        encounter.fightStart,
                        $colors
                    );
                    let bossChart = getBossHpSeries(
                        bossHpLogs,
                        legendNames,
                        chartablePlayers[0].damageStats.dpsAverage.length,
                        5
                    );
                    chartOptions = getAverageDpsChart(
                        chartablePlayers,
                        legendNames,
                        chartPlayers,
                        bossChart,
                        deathTimes
                    );
                } else if (chartType === ChartType.ROLLING_DPS) {
                    let chartPlayers = getRollingPlayerSeries(
                        chartablePlayers,
                        legendNames,
                        encounter.fightStart,
                        $colors
                    );
                    let bossChart = getBossHpSeries(
                        bossHpLogs,
                        legendNames,
                        chartablePlayers[0].damageStats.dpsRolling10sAvg.length,
                        1
                    );
                    chartOptions = getRollingDpsChart(
                        chartablePlayers,
                        legendNames,
                        chartPlayers,
                        bossChart,
                        deathTimes
                    );
                } else if (chartType === ChartType.SKILL_LOG && player && player.entityType === EntityType.PLAYER) {
                    chartOptions = getSkillLogChart(
                        player,
                        $skillIcon.path,
                        encounter.lastCombatPacket,
                        encounter.fightStart
                    );
                }
            }
        }
    }

    function inspectPlayer(name: string) {
        state = MeterState.PLAYER;
        playerName = name;
        chartType = ChartType.SKILL_LOG;
    }

    function inspectBoss(name: string) {
        focusedBoss = name;
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

    function tankTab() {
        handleRightClick();
        tab = MeterTab.TANK;
        setChartView();
    }

    function bossTab() {
        handleRightClick();
        tab = MeterTab.BOSS;
        setChartView();
    }

    function identityTab() {
        if (!localPlayer) return;
        tab = MeterTab.IDENTITY;
        chartType = ChartType.IDENTITY;
    }

    function staggerTab() {
        tab = MeterTab.STAGGER;
        chartType = ChartType.STAGGER;
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
            scrollToTop();
        }
    }

    function scrollToTop() {
        targetDiv.scrollIntoView({ behavior: "smooth", block: "start", inline: "start" });
    }

    async function deleteEncounter() {
        await invoke("delete_encounter", { id: id });
        if ($page.url.searchParams.has("page")) {
            let currentPage = parseInt($page.url.searchParams.get("page")!);
            goto(`/logs?page=${currentPage}`);
        } else {
            goto("/logs");
        }
    }

    let dropdownOpen = false;

    const handleDropdownClick = () => {
        dropdownOpen = !dropdownOpen;
    };

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

<svelte:window on:contextmenu|preventDefault />
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
    bind:this={targetDiv}
    class="scroll-ml-8 scroll-mt-2 text-gray-100"
    class:p-4={$takingScreenshot}
    on:contextmenu|preventDefault={handleRightClick}>
    <LogEncounterInfo
        bossName={encounter.currentBossName}
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
                    class:bg-accent-900={tab == MeterTab.DAMAGE}
                    class:bg-gray-700={tab != MeterTab.DAMAGE}
                    on:click={damageTab}>
                    Damage
                </button>
                <button
                    class="rounded-sm px-2 py-1 flex-shrink-0"
                    class:bg-accent-900={tab == MeterTab.PARTY_BUFFS}
                    class:bg-gray-700={tab != MeterTab.PARTY_BUFFS}
                    on:click={partySynergyTab}>
                    Party Buffs
                </button>
                <button
                    class="rounded-sm px-2 py-1 flex-shrink-0"
                    class:bg-accent-900={tab == MeterTab.SELF_BUFFS}
                    class:bg-gray-700={tab != MeterTab.SELF_BUFFS}
                    on:click={selfSynergyTab}>
                    Self Buffs
                </button>
                {#if $settings.general.showTanked && encounter.encounterDamageStats.totalDamageTaken > 0}
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={tab == MeterTab.TANK}
                        class:bg-gray-700={tab != MeterTab.TANK}
                        on:click={tankTab}>
                        Tanked
                    </button>
                {/if}
                {#if $settings.general.showBosses && bosses.length > 0}
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={tab == MeterTab.BOSS}
                        class:bg-gray-700={tab != MeterTab.BOSS}
                        on:click={bossTab}>
                        Bosses
                    </button>
                {/if}
                {#if localPlayer && localPlayer.skillStats.identityStats}
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={tab == MeterTab.IDENTITY}
                        class:bg-gray-700={tab != MeterTab.IDENTITY}
                        on:click={identityTab}>
                        Identity
                    </button>
                {/if}
                {#if encounter.encounterDamageStats.misc && encounter.encounterDamageStats.misc.staggerStats}
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={tab == MeterTab.STAGGER}
                        class:bg-gray-700={tab != MeterTab.STAGGER}
                        on:click={staggerTab}>
                        Stagger
                    </button>
                {/if}
                <button
                    class="rounded-sm bg-gray-700 px-2 py-1"
                    use:tooltip={{ content: "Take Screenshot" }}
                    on:click={captureScreenshot}>
                    <svg
                        class="hover:fill-accent-800 h-5 w-5 fill-zinc-300"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 -960 960 960"
                        ><path
                            d="M479.5-269.5q71.75 0 119.625-47.875T647-437q0-71-47.875-118.75T479.5-603.5q-71.75 0-119.125 47.75T313-437q0 71.75 47.375 119.625T479.5-269.5Zm0-57.5q-47 0-78-31.145T370.5-437q0-47 31-78t78-31q47 0 78.5 31t31.5 78.25q0 47.25-31.5 78.5T479.5-327Zm-328 227.5q-38.019 0-64.76-26.741Q60-152.981 60-191v-491.5q0-37.431 26.74-64.966Q113.482-775 151.5-775h132l83.057-97.5H594.5l82 97.5h132q37.431 0 64.966 27.534Q901-719.931 901-682.5V-191q0 38.019-27.534 64.759Q845.931-99.5 808.5-99.5h-657Zm657-91.5v-491.5H635L552.5-780H408.451L325.5-682.5h-174V-191h657ZM480-436.5Z" /></svg>
                </button>
                <div class="relative flex items-center rounded-sm bg-gray-700" on:focusout={handleDropdownFocusLoss}>
                    <button on:click={handleDropdownClick} class="h-full px-2">
                        <svg
                            class="h-4 w-4"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            xmlns="http://www.w3.org/2000/svg"
                            ><path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M19 9l-7 7-7-7" /></svg>
                    </button>
                    {#if dropdownOpen}
                        <div class="absolute left-9 top-0 z-50 rounded-md bg-gray-700">
                            <div class="flex w-40 flex-col divide-y-2 divide-gray-600 px-2 py-1">
                                <button
                                    class="hover:text-accent-500 p-1 text-left"
                                    on:click={() => {
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
                                            class="peer-checked:bg-accent-800 peer h-5 w-9 rounded-full border-gray-600 bg-gray-800 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white peer-focus:outline-none" />
                                    </label>
                                </button>
                                <button
                                    class="p-1 text-left hover:text-red-600"
                                    on:click={() => {
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
                <div class="fixed inset-0 z-50 bg-zinc-900 bg-opacity-80" />
                <div class="fixed left-0 right-0 top-0 z-50 h-modal w-full items-center justify-center p-4">
                    <div class="relative top-[25%] mx-auto flex max-h-full w-full max-w-md">
                        <div
                            class="relative mx-auto flex flex-col rounded-lg border-gray-700 bg-zinc-800 text-gray-400 shadow-md">
                            <button
                                type="button"
                                class="absolute right-2.5 top-3 ml-auto whitespace-normal rounded-lg p-1.5 hover:bg-zinc-600 focus:outline-none"
                                aria-label="Close modal"
                                on:click={() => (deleteConfirm = false)}>
                                <span class="sr-only">Close modal</span>
                                <svg
                                    class="h-5 w-5"
                                    fill="currentColor"
                                    viewBox="0 0 20 20"
                                    xmlns="http://www.w3.org/2000/svg"
                                    ><path
                                        fill-rule="evenodd"
                                        d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                                        clip-rule="evenodd" /></svg>
                            </button>
                            <div id="modal" class="flex-1 space-y-6 overflow-y-auto overscroll-contain p-6">
                                <div class="text-center">
                                    <svg
                                        aria-hidden="true"
                                        class="mx-auto mb-4 h-14 w-14 text-gray-200"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                        xmlns="http://www.w3.org/2000/svg"
                                        ><path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                            class="s-Qbr4I8QhaoSZ" /></svg>
                                    <h3 class="mb-5 text-lg font-normal text-gray-400">
                                        Are you sure you want to delete this encounter?
                                    </h3>
                                    <button
                                        type="button"
                                        class="mr-2 inline-flex items-center justify-center rounded-lg bg-red-700 px-5 py-2.5 text-center text-sm font-medium text-white hover:bg-red-800 focus:outline-none"
                                        on:click={deleteEncounter}>
                                        Yes, I'm sure
                                    </button>
                                    <button
                                        type="button"
                                        class="inline-flex items-center justify-center rounded-lg bg-gray-800 bg-transparent px-5 py-2.5 text-center text-sm font-medium text-gray-400 hover:bg-zinc-700 hover:text-white focus:text-white focus:outline-none"
                                        on:click={() => (deleteConfirm = false)}>
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
    {#if tab === MeterTab.IDENTITY && localPlayer !== null}
        <LogIdentity {localPlayer} duration={encounter.duration} />
    {:else if tab === MeterTab.STAGGER && encounter.encounterDamageStats.misc && encounter.encounterDamageStats.misc.staggerStats}
        <LogStagger staggerStats={encounter.encounterDamageStats.misc.staggerStats} />
    {:else}
        <div class="px relative top-0 overflow-x-auto overflow-y-visible">
            {#if tab === MeterTab.DAMAGE}
                {#if state === MeterState.PARTY}
                    <table class="relative w-full table-fixed">
                        <thead
                            class="z-30 h-6"
                            on:contextmenu|preventDefault={() => {
                                console.log("titlebar clicked");
                            }}>
                            <tr class="bg-zinc-900">
                                <th class="w-7 px-2 font-normal" />
                                <th class="w-14 px-2 text-left font-normal" />
                                <th class="w-full" />
                                {#if anyDead && $settings.logs.deathTime}
                                    <th class="w-16 font-normal" use:tooltip={{ content: "Dead for" }}>Dead for</th>
                                {/if}
                                {#if $settings.logs.damage}
                                    <th class="w-14 font-normal" use:tooltip={{ content: "Damage Dealt" }}>DMG</th>
                                {/if}
                                {#if $settings.logs.dps}
                                    <th class="w-14 font-normal" use:tooltip={{ content: "Damage per second" }}>DPS</th>
                                {/if}
                                {#if !isSolo && $settings.logs.damagePercent}
                                    <th class="w-12 font-normal" use:tooltip={{ content: "Damage %" }}>D%</th>
                                {/if}
                                {#if $settings.logs.critRate}
                                    <th class="w-12 font-normal" use:tooltip={{ content: "Crit %" }}>CRIT</th>
                                {/if}
                                {#if $settings.logs.critDmg}
                                    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage that Crit" }}
                                        >CDMG</th>
                                {/if}
                                {#if anyFrontAtk && $settings.logs.frontAtk}
                                    <th class="w-12 font-normal" use:tooltip={{ content: "Front Attack %" }}>F.A</th>
                                {/if}
                                {#if anyBackAtk && $settings.logs.backAtk}
                                    <th class="w-12 font-normal" use:tooltip={{ content: "Back Attack %" }}>B.A</th>
                                {/if}
                                {#if anySupportBuff && $settings.logs.percentBuffBySup}
                                    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support" }}
                                        >Buff%</th>
                                {/if}
                                {#if anySupportIdentity && $settings.logs.percentIdentityBySup}
                                    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support Identity" }}
                                    >Iden%
                                    </th>
                                {/if}
                                {#if anySupportBrand && $settings.logs.percentBrand}
                                    <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Brand" }}
                                        >B%</th>
                                {/if}
                                {#if $settings.logs.counters}
                                    <th class="w-12 font-normal" use:tooltip={{ content: "Counters" }}>CTR</th>
                                {/if}
                            </tr>
                        </thead>
                        <tbody class="relative z-10">
                            {#each players as player, i (player.name)}
                                <tr
                                    class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                                    on:click={() => inspectPlayer(player.name)}>
                                    <LogDamageMeterRow
                                        entity={player}
                                        percentage={playerDamagePercentages[i]}
                                        {totalDamageDealt}
                                        {anyDead}
                                        {anyFrontAtk}
                                        {anyBackAtk}
                                        {anySupportBuff}
                                        {anySupportIdentity}
                                        {anySupportBrand}
                                        end={encounter.lastCombatPacket}
                                        {isSolo} />
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                {:else if state === MeterState.PLAYER && player !== null}
                    <table class="relative w-full table-fixed">
                        <LogPlayerBreakdown entity={player} duration={encounter.duration} {totalDamageDealt} />
                    </table>
                    {#if player.class === "Arcanist"}
                        <table class="relative w-full table-fixed">
                            <ArcanistCardTable {player} duration={encounter.duration} />
                        </table>
                    {/if}
                {/if}
            {:else if tab === MeterTab.PARTY_BUFFS}
                {#if state === MeterState.PARTY}
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
                {#if state === MeterState.PARTY}
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
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="mt-4" on:contextmenu|preventDefault={handleRightClick}>
        {#if chartType === ChartType.SKILL_LOG}
            {#if player && player.entityType === EntityType.PLAYER}
                <OpenerSkills skills={player.skills} />
            {/if}
        {/if}
        {#if player?.entityType !== EntityType.ESTHER}
            <div class="text-lg font-medium">Charts</div>
            <div class="mt-2 flex divide-x divide-gray-600">
                {#if playerName === "" && state === MeterState.PARTY}
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={chartType == ChartType.AVERAGE_DPS}
                        class:bg-gray-700={chartType != ChartType.AVERAGE_DPS}
                        on:click={() => (chartType = ChartType.AVERAGE_DPS)}>
                        Average DPS
                    </button>
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={chartType == ChartType.ROLLING_DPS}
                        class:bg-gray-700={chartType != ChartType.ROLLING_DPS}
                        on:click={() => (chartType = ChartType.ROLLING_DPS)}>
                        10s DPS Window
                    </button>
                {:else if playerName !== "" && state === MeterState.PLAYER}
                    <button
                        class="rounded-sm px-2 py-1"
                        class:bg-accent-900={chartType == ChartType.SKILL_LOG}
                        class:bg-gray-700={chartType != ChartType.SKILL_LOG}
                        on:click={() => (chartType = ChartType.SKILL_LOG)}>
                        Skill Casts
                    </button>
                {/if}
            </div>
        {/if}
        {#if chartType === ChartType.AVERAGE_DPS}
            {#if !$settings.general.showNames}
                <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);" />
            {:else}
                <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);" />
            {/if}
        {:else if chartType === ChartType.ROLLING_DPS}
            {#if !$settings.general.showNames}
                <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);" />
            {:else}
                <div class="mt-2 h-[300px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);" />
            {/if}
        {:else if chartType === ChartType.SKILL_LOG}
            {#if player && player.entityType === EntityType.PLAYER}
                <div class="mt-2 h-[400px]" use:chartable={chartOptions} style="width: calc(100vw - 4.5rem);" />
            {/if}
        {/if}
    </div>
{/if}
