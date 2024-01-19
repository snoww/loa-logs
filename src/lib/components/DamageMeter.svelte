<script lang="ts">
    import {
        MeterState,
        MeterTab,
        type Encounter,
        type EncounterEvent,
        type Entity,
        EntityType,
        type PartyInfo,
        type PartyEvent
    } from "$lib/types";
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
    import Details from "./Details.svelte";
    import DamageTaken from "./shared/DamageTaken.svelte";
    import BossTable from "./shared/BossTable.svelte";
    import BossBreakdown from "./shared/BossBreakdown.svelte";

    let time = +Date.now();
    let encounter: Encounter | null = null;
    let parties: PartyInfo | undefined;
    let events: Array<UnlistenFn> = [];

    let zoneChangeAlert = false;
    let resettingAlert = false;
    let pauseAlert = false;
    let saveAlert = false;
    let raidClear = false;
    let raidWipe = false;
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
            let partyUpdateEvent = await listen("party-update", (event: PartyEvent) => {
                if (event.payload) {
                    parties = event.payload;
                }
            });
            let zoneChangeEvent = await listen("zone-change", () => {
                // console.log("zone change event")
                zoneChangeAlert = true;
                $raidInProgress = false;
                setTimeout(() => {
                    reset();
                    zoneChangeAlert = false;
                    $raidInProgress = true;
                }, 6000);
            });
            let raidStartEvent = await listen("raid-start", () => {
                reset();
                $raidInProgress = true;
            });
            let resetEncounterEvent = await listen("reset-encounter", () => {
                reset();
                resettingAlert = true;
                setTimeout(() => {
                    resettingAlert = false;
                }, 1500);
            });
            let pauseEncounterEvent = await listen("pause-encounter", () => {
                $paused = !$paused;
                pauseAlert = !pauseAlert;
            });
            let saveEncounterEvent = await listen("save-encounter", () => {
                reset();
                saveAlert = true;
                setTimeout(() => {
                    saveAlert = false;
                }, 1500);
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
                    raidClear = true;
                    setTimeout(() => {
                        raidClear = false;
                    }, 3000);
                } else if (phaseCode === 4 && raidInProgress) {
                    raidWipe = true;
                    setTimeout(() => {
                        raidWipe = false;
                    }, 3000);
                }
                $raidInProgress = false;
            });
            let adminErrorEvent = await listen("admin", () => {
                adminAlert = true;
            });

            events.push(
                encounterUpdateEvent,
                partyUpdateEvent,
                zoneChangeEvent,
                resetEncounterEvent,
                pauseEncounterEvent,
                saveEncounterEvent,
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
    let bosses: Array<Entity> = [];
    let playerDamagePercentages: Array<number> = [];
    let topDamageDealt = 0;
    let encounterDuration = "00:00";
    let duration = 0;
    let totalDamageDealt = 0;
    let dps = 0;
    let timeUntilKill = "00:00";
    let currentBoss: Entity | null = null;
    let state = MeterState.PARTY;
    let tab = MeterTab.DAMAGE;
    let player: Entity | null = null;
    let playerName = "";
    let focusedBoss = "";
    let lastCombatPacket = 0;
    let anyDead: boolean = false;
    let anyFrontAtk: boolean = false;
    let anyBackAtk: boolean = false;
    let anySupportBuff: boolean = false;
    let anySupportIdentity: boolean = false;
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
                if ($settings.general.showBosses) {
                    bosses = Object.values(encounter.entities)
                        .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.BOSS)
                        .sort((a, b) => b.damageStats.damageDealt - a.damageStats.damageDealt);
                }
                isSolo = players.length === 1;
                anyDead = players.some((player) => player.isDead);
                anyFrontAtk = players.some((player) => player.skillStats.frontAttacks > 0);
                anyBackAtk = players.some((player) => player.skillStats.backAttacks > 0);
                anySupportBuff = players.some((player) => player.damageStats.buffedBySupport > 0);
                anySupportIdentity = players.some((player) => player.damageStats.buffedByIdentity > 0);
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
                if ($settings.meter.timeUntilKill) {
                    let remainingDps =
                        players
                            .filter(
                                (e) => e.damageStats.damageDealt > 0 && !e.isDead && e.entityType != EntityType.ESTHER
                            )
                            .reduce((a, b) => a + b.damageStats.damageDealt, 0) /
                        (duration / 1000);
                    let remainingBossHealth = 0;
                    if (encounter.currentBoss?.currentHp) {
                        remainingBossHealth += encounter.currentBoss.currentHp;
                    }
                    if (encounter.currentBoss?.currentShield) {
                        remainingBossHealth += encounter.currentBoss.currentShield;
                    }
                    let millisUntilKill = Math.floor((1000 * remainingBossHealth) / remainingDps);
                    millisUntilKill = Math.max(millisUntilKill, 0);
                    timeUntilKill = millisToMinutesAndSeconds(millisUntilKill);
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

    function inspectBoss(name: string) {
        focusedBoss = name;
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

    function handleBossRightClick() {
        focusedBoss = "";
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
        focusedBoss = "";
        encounter = null;
        players = [];
        parties = undefined;
        currentBoss = null;
        encounterDuration = "00:00";
        totalDamageDealt = 0;
        dps = 0;
        timeUntilKill = "00:00";
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
    <EncounterInfo {encounterDuration} {totalDamageDealt} {dps} {timeUntilKill} screenshotFn={captureScreenshot} />
    {#if currentBoss !== null && $settings.meter.bossHp}
        <div class="relative top-7">
            <BossInfo boss={currentBoss} />
        </div>
    {/if}
    <div
        class="relative top-7 scroll-ml-8 scroll-mt-2 overflow-scroll"
        style="height: calc(100vh - 1.5rem - 1.75rem {currentBoss !== null ? ' - 1.75rem' : ''});">
        {#if tab === MeterTab.DAMAGE}
            {#if state === MeterState.PARTY}
                <table class="relative w-full table-fixed" id="live-meter-table">
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
                            {#if $settings.meter.critDmg}
                                <th class="w-12 font-normal" use:tooltip={{ content: "% Damage that Crit" }}>CDMG</th>
                            {/if}
                            {#if anyFrontAtk && $settings.meter.frontAtk}
                                <th class="w-12 font-normal" use:tooltip={{ content: "Front Attack %" }}>F.A</th>
                            {/if}
                            {#if anyBackAtk && $settings.meter.backAtk}
                                <th class="w-12 font-normal" use:tooltip={{ content: "Back Attack %" }}>B.A</th>
                            {/if}
                            {#if anySupportBuff && $settings.meter.percentBuffBySup}
                                <th
                                    class="w-12 font-normal"
                                    use:tooltip={{ content: "% Damage buffed by Support Atk. Power buff" }}
                                    >Buff%
                                </th>
                            {/if}
                            {#if anySupportIdentity && $settings.meter.percentIdentityBySup}
                                <th
                                    class="w-12 font-normal"
                                    use:tooltip={{ content: "% Damage buffed by Support Identity" }}
                                    >Iden%
                                </th>
                            {/if}
                            {#if anySupportBrand && $settings.meter.percentBrand}
                                <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Brand" }}
                                    >B%</th>
                            {/if}
                            {#if $settings.meter.counters}
                                <th class="w-12 font-normal" use:tooltip={{ content: "Counters" }}>CTR</th>
                            {/if}
                        </tr>
                    </thead>
                    <tbody>
                        {#each players as entity, i (entity.name)}
                            <tr
                                class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
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
                                    {anySupportIdentity}
                                    {anySupportBrand}
                                    {isSolo} />
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {:else if state === MeterState.PLAYER && player !== null}
                <table class="relative w-full table-fixed" id="live-meter-table">
                    <PlayerBreakdown entity={player} {duration} {handleRightClick} />
                </table>
            {/if}
        {:else if tab === MeterTab.PARTY_BUFFS}
            {#if state === MeterState.PARTY}
                <Buffs
                    {tab}
                    encounterDamageStats={encounter?.encounterDamageStats}
                    {players}
                    {handleRightClick}
                    {inspectPlayer}
                    encounterPartyInfo={parties}
                    localPlayer={encounter?.localPlayer} />
            {:else}
                <Buffs
                    {tab}
                    encounterDamageStats={encounter?.encounterDamageStats}
                    {players}
                    focusedPlayer={player}
                    {handleRightClick}
                    {inspectPlayer}
                    encounterPartyInfo={parties}
                    localPlayer={encounter?.localPlayer} />
            {/if}
        {:else if tab === MeterTab.SELF_BUFFS}
            {#if state === MeterState.PARTY}
                <Buffs
                    {tab}
                    encounterDamageStats={encounter?.encounterDamageStats}
                    {players}
                    focusedPlayer={player}
                    {handleRightClick}
                    {inspectPlayer}
                    encounterPartyInfo={parties}
                    localPlayer={encounter?.localPlayer} />
            {:else}
                <Buffs
                    {tab}
                    encounterDamageStats={encounter?.encounterDamageStats}
                    {players}
                    focusedPlayer={player}
                    {handleRightClick}
                    {inspectPlayer}
                    encounterPartyInfo={parties}
                    localPlayer={encounter?.localPlayer} />
            {/if}
        {:else if tab === MeterTab.TANK}
            <DamageTaken {players} topDamageTaken={encounter?.encounterDamageStats.topDamageTaken} />
        {:else if tab === MeterTab.BOSS}
            {#if !focusedBoss}
                <BossTable {bosses} {duration} {inspectBoss} />
            {:else}
                <BossBreakdown
                    boss={encounter?.entities[focusedBoss]}
                    {duration}
                    handleRightClick={handleBossRightClick} />
            {/if}
        {:else if tab === MeterTab.DETAILS}
            <Details />
        {/if}
    </div>
    {#if zoneChangeAlert}
        <Notification bind:showAlert={zoneChangeAlert} text="Changing Zone" width={"11rem"} dismissable={false} />
    {/if}
    {#if resettingAlert}
        <Notification bind:showAlert={resettingAlert} text="Resetting" width={"9rem"} dismissable={false} />
    {/if}
    {#if pauseAlert}
        <Notification bind:showAlert={pauseAlert} text="Paused" width={"8rem"} dismissable={false} />
    {/if}
    {#if saveAlert}
        <Notification bind:showAlert={saveAlert} text="Saving" width={"8rem"} dismissable={false} />
    {/if}
    {#if raidClear}
        <Notification bind:showAlert={raidClear} text="Phase Clear" width={"9.5rem"} dismissable={false} />
    {/if}
    {#if raidWipe}
        <Notification bind:showAlert={raidWipe} text="Phase Wipe" width={"9rem"} dismissable={false} />
    {/if}
    {#if bossDeadAlert}
        <Notification bind:showAlert={bossDeadAlert} text="Boss Dead" width={"10rem"} dismissable={false} />
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
        <Notification
            bind:showAlert={$screenshotError}
            text={"Screenshot Copied to Clipboard"}
            width="18rem"
            dismissable={false} />
    {/if}
    {#if $screenshotError}
        <Notification
            bind:showAlert={$screenshotError}
            text={"Error Taking Screenshot"}
            width="15rem"
            isError={true}
            dismissable={false} />
    {/if}
    <Footer bind:tab />
</div>
