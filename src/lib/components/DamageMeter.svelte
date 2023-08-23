<script lang="ts">
    import { MeterState, MeterTab, type Encounter, type EncounterEvent, type Entity, EntityType } from "$lib/types";
    import { millisToMinutesAndSeconds } from "$lib/utils/numbers";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";
    import { flip } from "svelte/animate";
    import EncounterInfo from "./EncounterInfo.svelte";
    import BossInfo from "./BossInfo.svelte";
    import DamageMeterPlayerRow from "./DamageMeterRow.svelte";
    import PlayerBreakdown from "./PlayerBreakdown.svelte";
    import Footer from "./Footer.svelte";
    import Buffs from "./Buffs.svelte";
    import { settings } from "$lib/utils/settings";
    import { tooltip } from "$lib/utils/tooltip";
    import { writable } from "svelte/store";
    import Notification from "./shared/Notification.svelte";
    import { takingScreenshot, screenshotAlert, screenshotError } from "$lib/utils/stores";
    import html2canvas from "html2canvas";

    let time = +Date.now();
    let encounter: Encounter | null = null;
    let events: Array<UnlistenFn> = [];

    let zoneChangeAlert = false;
    let resettingAlert = false;
    let pauseAlert = false;
    let phaseTransitionAlert = false;
    let phaseStartAlert = false;
    let bossDeadAlert = false;
    let adminAlert = false;
    let raidInProgress = writable(true);

    onMount(() => {
        setInterval(() => {
            time = +Date.now();
        }, 1000);

        (async () => {
            let encounterUpdateEvent = await listen("encounter-update", (event: EncounterEvent) => {
                // console.log(+Date.now(), event.payload);
                encounter = event.payload;
            });
            let zoneChangeEvent = await listen("zone-change", (event: any) => {
                // console.log("zone change event")
                zoneChangeAlert = true;
                $raidInProgress = false;
                setTimeout(() => {
                    reset();
                    zoneChangeAlert = false;
                    $raidInProgress = true;
                }, 6000);
            });
            let raidStartEvent = await listen("raid-start", (event: any) => {
                reset();
                $raidInProgress = true;
            });
            let resetEncounterEvent = await listen("reset-encounter", (event: any) => {
                reset();
                resettingAlert = true;
                setTimeout(() => {
                    resettingAlert = false;
                }, 1500);
            });
            let pauseEncounterEvent = await listen("pause-encounter", (event: any) => {
                $paused = !$paused;
                pauseAlert = !pauseAlert;
            });
            let phaseTransitionEvent = await listen("phase-transition", (event: any) => {
                let phaseCode = event.payload;
                // console.log(Date.now() + ": phase transition event: ", event.payload)
                if (phaseCode === 1) {
                    bossDeadAlert = true;
                    setTimeout(() => {
                        bossDeadAlert = false;
                    }, 3000);
                } else if (phaseCode === 2 && raidInProgress) {
                    phaseTransitionAlert = true;
                    setTimeout(() => {
                        phaseTransitionAlert = false;
                    }, 3000);
                } else if (phaseCode === 3 && raidInProgress) {
                    phaseStartAlert = true;
                    setTimeout(() => {
                        phaseStartAlert = false;
                    }, 3000);
                }
                $raidInProgress = false;
            });
            let adminErrorEvent = await listen("admin", (event: any) => {
                adminAlert = true;
            });

            events.push(
                encounterUpdateEvent,
                zoneChangeEvent,
                resetEncounterEvent,
                pauseEncounterEvent,
                phaseTransitionEvent,
                raidStartEvent,
                adminErrorEvent
            );
        })();
    });

    onDestroy(() => {
        events.forEach((unlisten) => unlisten());
    });

    let players: Array<Entity> = [];
    let playerDamagePercentages: Array<number> = [];
    let topDamageDealt = 0;
    let encounterDuration = "00:00";
    let duration = 0;
    let totalDamageDealt = 0;
    let dps = 0;
    let currentBoss: Entity | null = null;
    let state = MeterState.PARTY;
    let tab = MeterTab.DAMAGE;
    let player: Entity | null = null;
    let playerName = "";
    let lastCombatPacket = 0;
    let anyDead: boolean = false;
    let anyFrontAtk: boolean = false;
    let anyBackAtk: boolean = false;
    let anySupportBuff: boolean = false;
    let anySupportBrand: boolean = false;
    let isSolo: boolean = true;

    let paused = writable(false);

    $: {
        if (encounter) {
            if (encounter.fightStart !== 0 && !$paused) {
                if ($settings.general.showEsther) {
                    players = Object.values(encounter.entities)
                        .filter(
                            (e) =>
                                e.damageStats.damageDealt > 0 &&
                                (e.entityType === EntityType.ESTHER || e.entityType === EntityType.PLAYER)
                        )
                        .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
                } else {
                    players = Object.values(encounter.entities)
                        .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.PLAYER)
                        .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
                }
                isSolo = players.length === 1;
                anyDead = players.some((player) => player.isDead);
                anyFrontAtk = players.some((player) => player.skillStats.frontAttacks > 0);
                anyBackAtk = players.some((player) => player.skillStats.backAttacks > 0);
                anySupportBuff = players.some((player) => player.damageStats.buffedBySupport > 0);
                anySupportBrand = players.some((player) => player.damageStats.debuffedBySupport > 0);
                topDamageDealt = encounter.encounterDamageStats.topDamageDealt;
                playerDamagePercentages = players.map(
                    (player) => (player.damageStats.damageDealt / topDamageDealt) * 100
                );

                if (
                    // ((encounter.currentBoss && !encounter.currentBoss.isDead) || !encounter.currentBoss) &&
                    $raidInProgress
                ) {
                    duration = time - encounter.fightStart;
                }

                if (duration < 0) {
                    encounterDuration = millisToMinutesAndSeconds(0);
                    dps = 0;
                } else {
                    encounterDuration = millisToMinutesAndSeconds(duration);
                    dps = totalDamageDealt / (duration / 1000);
                }
                if ($settings.general.showEsther) {
                    totalDamageDealt =
                        encounter.encounterDamageStats.totalDamageDealt +
                        players
                            .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.ESTHER)
                            .reduce((a, b) => a + b.damageStats.damageDealt, 0);
                } else {
                    totalDamageDealt = encounter.encounterDamageStats.totalDamageDealt;
                }

                lastCombatPacket = encounter.lastCombatPacket;
            }

            if (encounter.currentBoss !== undefined) {
                currentBoss = encounter.currentBoss;
            }

            if (playerName) {
                player = encounter.entities[playerName];
                state = MeterState.PLAYER;
            } else {
                player = null;
                state = MeterState.PARTY;
            }
        }
    }

    function inspectPlayer(name: string) {
        state = MeterState.PLAYER;
        playerName = name;
        scrollToTopOfTable();
    }

    function handleRightClick() {
        if (state === MeterState.PLAYER) {
            state = MeterState.PARTY;
            player = null;
            playerName = "";
        }

        scrollToTopOfTable();
    }

    function scrollToTopOfTable() {
        let rows = document.querySelector("#live-meter-table")?.querySelectorAll("tr");
        if (rows && rows.length > 2) {
            rows[1].scrollIntoView({ behavior: "smooth", block: "center" });
        }
    }

    function reset() {
        state = MeterState.PARTY;
        player = null;
        playerName = "";
        encounter = null;
        players = [];
        currentBoss = null;
        encounterDuration = "00:00";
        totalDamageDealt = 0;
        dps = 0;
        anyDead = false;
        anyFrontAtk = false;
        anyBackAtk = false;
        anySupportBuff = false;
        anySupportBrand = false;
    }

    let targetDiv: HTMLElement;

    async function captureScreenshot() {
        takingScreenshot.set(true);
        document.body.style.pointerEvents = "none";
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
                        document.body.style.pointerEvents = "auto";
                    }, 2000);
                } catch (error) {
                    takingScreenshot.set(false);
                    $screenshotError = true;
                    setTimeout(() => {
                        $screenshotError = false;
                        document.body.style.pointerEvents = "auto";
                    }, 2000);
                }
            });
        }, 100);
    }
</script>

<svelte:window on:contextmenu|preventDefault />
<div bind:this={targetDiv}>
    <EncounterInfo {encounterDuration} {totalDamageDealt} {dps} screenshotFn={captureScreenshot}/>
    {#if currentBoss !== null && $settings.meter.bossHp}
        <div class="relative top-7">
            <BossInfo boss={currentBoss} />
        </div>
    {/if}
    <div
        class="relative top-7 overflow-scroll"
        style="height: calc(100vh - 1.5rem - 1.75rem {currentBoss !== null ? ' - 1.75rem' : ''});">
        <table class="relative w-full table-fixed" id="live-meter-table">
            {#if tab === MeterTab.DAMAGE}
                {#if state === MeterState.PARTY}
                    <thead
                        class="sticky top-0 z-40 h-6"
                        on:contextmenu|preventDefault={() => {
                            // console.log("titlebar clicked");
                        }}>
                        <tr class="bg-zinc-900 tracking-tighter">
                            <th class="w-7 px-2 font-normal" />
                            <th class="w-14 px-2 text-left font-normal" />
                            <th class="w-full" />
                            {#if anyDead && $settings.meter.deathTime}
                                <th class="w-14 font-normal" use:tooltip={{ content: "Dead for" }}>Dead</th>
                            {/if}
                            {#if $settings.meter.damage}
                                <th class="w-14 font-normal" use:tooltip={{ content: "Damage Dealt" }}>DMG</th>
                            {/if}
                            {#if $settings.meter.dps}
                                <th class="w-14 font-normal" use:tooltip={{ content: "Damage per second" }}>DPS</th>
                            {/if}
                            {#if !isSolo && $settings.meter.damagePercent}
                                <th class="w-12 font-normal" use:tooltip={{ content: "Damage %" }}>D%</th>
                            {/if}
                            {#if $settings.meter.critRate}
                                <th class="w-12 font-normal" use:tooltip={{ content: "Crit %" }}>CRIT</th>
                            {/if}
                            {#if anyFrontAtk && $settings.meter.frontAtk}
                                <th class="w-12 font-normal" use:tooltip={{ content: "Front Attack %" }}>F.A</th>
                            {/if}
                            {#if anyBackAtk && $settings.meter.backAtk}
                                <th class="w-12 font-normal" use:tooltip={{ content: "Back Attack %" }}>B.A</th>
                            {/if}
                            {#if anySupportBuff && $settings.meter.percentBuffBySup}
                                <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Support" }}
                                    >Buff%</th>
                            {/if}
                            {#if anySupportBrand && $settings.meter.percentBrand}
                                <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Brand" }}>B%</th>
                            {/if}
                            {#if $settings.meter.counters}
                                <th class="w-12 font-normal" use:tooltip={{ content: "Counters" }}>CTR</th>
                            {/if}
                        </tr>
                    </thead>
                    <tbody>
                        {#each players as entity, i (entity.id)}
                            <tr
                                class="h-7 px-2 py-1"
                                animate:flip={{ duration: 200 }}
                                on:click={() => inspectPlayer(entity.name)}>
                                <DamageMeterPlayerRow
                                    {entity}
                                    percentage={playerDamagePercentages[i]}
                                    {duration}
                                    {totalDamageDealt}
                                    {lastCombatPacket}
                                    {anyDead}
                                    {anyFrontAtk}
                                    {anyBackAtk}
                                    {anySupportBuff}
                                    {anySupportBrand}
                                    {isSolo} />
                            </tr>
                        {/each}
                    </tbody>
                {:else if state === MeterState.PLAYER && player !== null}
                    <PlayerBreakdown entity={player} {duration} {handleRightClick} />
                {/if}
            {:else if tab === MeterTab.PARTY_BUFFS}
                {#if state === MeterState.PARTY}
                    <Buffs
                        {tab}
                        encounterDamageStats={encounter?.encounterDamageStats}
                        {players}
                        {handleRightClick}
                        {inspectPlayer} />
                {:else}
                    <Buffs
                        {tab}
                        encounterDamageStats={encounter?.encounterDamageStats}
                        {players}
                        focusedPlayer={player}
                        {handleRightClick}
                        {inspectPlayer} />
                {/if}
            {:else if tab === MeterTab.SELF_BUFFS}
                {#if state === MeterState.PARTY}
                    <Buffs
                        {tab}
                        encounterDamageStats={encounter?.encounterDamageStats}
                        {players}
                        focusedPlayer={player}
                        {handleRightClick}
                        {inspectPlayer} />
                {:else}
                    <Buffs
                        {tab}
                        encounterDamageStats={encounter?.encounterDamageStats}
                        {players}
                        focusedPlayer={player}
                        {handleRightClick}
                        {inspectPlayer} />
                {/if}
            {/if}
        </table>
    </div>
    {#if zoneChangeAlert}
        <Notification bind:showAlert={zoneChangeAlert} text="Changing Zone" width={"14rem"} />
    {/if}
    {#if resettingAlert}
        <Notification bind:showAlert={resettingAlert} text="Resetting" width={"10rem"} />
    {/if}
    {#if pauseAlert}
        <Notification bind:showAlert={pauseAlert} text="Paused" width={"8rem"} dismissable={false} />
    {/if}
    {#if phaseTransitionAlert}
        <Notification bind:showAlert={phaseTransitionAlert} text="Wipe/Phase Clear" width={"15rem"} />
    {/if}
    {#if phaseStartAlert}
        <Notification bind:showAlert={phaseStartAlert} text="Raid Start" width={"12rem"} />
    {/if}
    {#if bossDeadAlert}
        <Notification bind:showAlert={bossDeadAlert} text="Boss Dead" width={"12rem"} />
    {/if}
    {#if adminAlert}
        <Notification
            bind:showAlert={adminAlert}
            text="Please restart as Admin"
            width={"16em"}
            dismissable={false}
            isError={true} />
    {/if}
    {#if $screenshotAlert}
        <Notification bind:showAlert={$screenshotError} text={"Screenshot Copied to Clipboard"} width="20rem"/>
    {/if}
    {#if $screenshotError}
        <Notification bind:showAlert={$screenshotError} text={"Error Taking Screenshot"} width="18rem" isError={true}/>
    {/if}
    <Footer bind:tab />
</div>
