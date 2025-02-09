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
    import {
        takingScreenshot,
        screenshotAlert,
        screenshotError,
        rdpsEventDetails,
        localPlayer,
        missingInfo
    } from "$lib/utils/stores";
    import html2canvas from "html2canvas";
    import Details from "./Details.svelte";
    import DamageTaken from "./shared/DamageTaken.svelte";
    import BossTable from "./shared/BossTable.svelte";
    import BossBreakdown from "./shared/BossBreakdown.svelte";
    import Rdps from "$lib/components/shared/Rdps.svelte";
    import { isValidName } from "$lib/utils/strings";
    import MissingInfo from "./shared/MissingInfo.svelte";
    import { invoke } from "@tauri-apps/api";
    import { uploadLog } from "$lib/utils/sync";

    let time = $state(+Date.now());
    let encounter: Encounter | null = $state(null);
    let parties: PartyInfo | undefined = $state();
    let events: Array<UnlistenFn> = [];

    let zoneChangeAlert = $state(false);
    let resettingAlert = $state(false);
    let pauseAlert = $state(false);
    let saveAlert = $state(false);
    let raidClear = $state(false);
    let raidWipe = $state(false);
    let bossDeadAlert = $state(false);
    let adminAlert = $state(false);
    let raidInProgress = writable(true);

    onMount(() => {
        setInterval(() => {
            time = +Date.now();
        }, 1000);

        $rdpsEventDetails = "not_available";

        (async () => {
            let encounterUpdateEvent = await listen("encounter-update", (event: EncounterEvent) => {
                // console.log(+Date.now(), event.payload);
                encounter = event.payload;
            });
            let invalidDamageEvent = await listen("invalid-damage", () => {
                $missingInfo = true;
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
            let clearEncounterEvent = await listen("clear-encounter", async (event: any) => {
                if (!$settings.sync.auto) {
                    return;
                }

                let id = event.payload.toString();
                const encounter = (await invoke("load_encounter", { id })) as Encounter;
                await uploadLog(id, encounter, $settings.sync);
            });
            let adminErrorEvent = await listen("admin", () => {
                adminAlert = true;
            });
            let rdpsEvent = await listen("rdps", (event: any) => {
                if (event.payload === "request_success") {
                    $rdpsEventDetails = "";
                } else {
                    $rdpsEventDetails = event.payload;
                }
            });

            events.push(
                encounterUpdateEvent,
                invalidDamageEvent,
                partyUpdateEvent,
                zoneChangeEvent,
                resetEncounterEvent,
                pauseEncounterEvent,
                saveEncounterEvent,
                phaseTransitionEvent,
                raidStartEvent,
                adminErrorEvent,
                rdpsEvent,
                clearEncounterEvent
            );
        })();
    });

    onDestroy(() => {
        events.forEach((unlisten) => unlisten());
    });

    let players: Array<Entity> = $state([]);
    let bosses: Array<Entity> = $state([]);
    let playerDamagePercentages: Array<number> = $state([]);
    let topDamageDealt = $state(0);
    let encounterDuration = $state("00:00");
    let duration = $state(0);
    let totalDamageDealt = $state(0);
    let dps = $state(0);
    let timeUntilKill = $state("00:00");
    let currentBoss: Entity | null = $state(null);
    let meterState = $state(MeterState.PARTY);
    let tab = $state(MeterTab.DAMAGE);
    let player: Entity | null = $state(null);
    let playerName = $state("");
    let focusedBoss = $state("");
    let lastCombatPacket = $state(0);
    let anyDead: boolean = $state(false);
    let multipleDeaths: boolean = $state(false);
    let anyFrontAtk: boolean = $state(false);
    let anyBackAtk: boolean = $state(false);
    let anySupportBuff: boolean = $state(false);
    let anySupportIdentity: boolean = $state(false);
    let anySupportBrand: boolean = $state(false);
    let anyRdpsData: boolean = $state(false);
    let isSolo: boolean = $state(true);

    let paused = writable(false);

    $effect(() => {
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
            }
        }
    });

    $effect(() => {
        if (encounter) {
            if (encounter.fightStart !== 0 && !$paused) {
                if (duration < 0) {
                    encounterDuration = millisToMinutesAndSeconds(0);
                    dps = 0;
                    timeUntilKill = "00:00";
                } else {
                    encounterDuration = millisToMinutesAndSeconds(duration);
                    dps = totalDamageDealt / (duration / 1000);
                    if ($settings.meter.showTimeUntilKill && encounter.currentBoss) {
                        let remainingDpm =
                            players
                                .filter(
                                    (e) =>
                                        e.damageStats.damageDealt > 0 && !e.isDead && e.entityType == EntityType.PLAYER
                                )
                                .reduce((a, b) => a + b.damageStats.damageDealt, 0) / duration;
                        let remainingBossHealth = encounter.currentBoss.currentHp + encounter.currentBoss.currentShield;
                        let millisUntilKill = Math.max(remainingBossHealth / remainingDpm, 0);
                        if (millisUntilKill > 3.6e6) {
                            // 1 hr
                            timeUntilKill = "∞";
                        } else {
                            timeUntilKill = millisToMinutesAndSeconds(millisUntilKill);
                        }
                    }
                }

                if (
                    // ((encounter.currentBoss && !encounter.currentBoss.isDead) || !encounter.currentBoss) &&
                    $raidInProgress
                ) {
                    duration = time - encounter.fightStart;
                }
            }
        }
    });

    $effect(() => {
        if (encounter) {
            if (encounter.fightStart !== 0 && !$paused) {
                topDamageDealt = encounter.encounterDamageStats.topDamageDealt;
                playerDamagePercentages = players.map(
                    (player) => (player.damageStats.damageDealt / topDamageDealt) * 100
                );

                if ($settings.general.showEsther) {
                    totalDamageDealt =
                        encounter.encounterDamageStats.totalDamageDealt +
                        players
                            .filter((e) => e.damageStats.damageDealt > 0 && e.entityType === EntityType.ESTHER)
                            .reduce((a, b) => a + b.damageStats.damageDealt, 0);
                } else {
                    totalDamageDealt = encounter.encounterDamageStats.totalDamageDealt;
                }
            }
        }
    });

    $effect(() => {
        if (encounter) {
            if (encounter.fightStart !== 0 && !$paused) {
                if (!$missingInfo) {
                    if (encounter.localPlayer === "You" || !isValidName(encounter.localPlayer)) {
                        $missingInfo = true;
                    }
                }
                $localPlayer = encounter.localPlayer;
                isSolo = players.length === 1;
                anyDead = players.some((player) => player.isDead);
                if (!anyDead) {
                    multipleDeaths = players.some((player) => player.damageStats.deaths > 0);
                } else {
                    multipleDeaths = players.some((player) => player.damageStats.deaths > 1);
                }
                if (!anyFrontAtk) {
                    anyFrontAtk = players.some((player) => player.skillStats.frontAttacks > 0);
                }
                if (!anyBackAtk) {
                    anyBackAtk = players.some((player) => player.skillStats.backAttacks > 0);
                }
                if (!anySupportBuff) {
                    anySupportBuff = players.some((player) => player.damageStats.buffedBySupport > 0);
                }
                if (!anySupportIdentity) {
                    anySupportIdentity = players.some((player) => player.damageStats.buffedByIdentity > 0);
                }
                if (!anySupportBrand) {
                    anySupportBrand = players.some((player) => player.damageStats.debuffedBySupport > 0);
                }
                if (!anyRdpsData) {
                    anyRdpsData = players.some((player) => player.damageStats.rdpsDamageReceived > 0);
                }
                if (!anyRdpsData) {
                    if ($rdpsEventDetails === "") {
                        $rdpsEventDetails = "not_available";
                    }
                } else {
                    $rdpsEventDetails = "";
                }

                lastCombatPacket = encounter.lastCombatPacket;
            }

            if (encounter.currentBoss) {
                currentBoss = encounter.currentBoss;
            }

            if (playerName) {
                player = encounter.entities[playerName];
                meterState = MeterState.PLAYER;
            } else {
                player = null;
                meterState = MeterState.PARTY;
            }
        }
    });

    function inspectPlayer(name: string) {
        meterState = MeterState.PLAYER;
        playerName = name;
        scrollToTopOfTable();
    }

    function inspectBoss(name: string) {
        focusedBoss = name;
        scrollToTopOfTable();
    }

    function handleRightClick(e: MouseEvent | undefined = undefined) {
        if (e) {
            e.preventDefault();
        }
        if (meterState === MeterState.PLAYER) {
            meterState = MeterState.PARTY;
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
        meterState = MeterState.PARTY;
        player = null;
        playerName = "";
        focusedBoss = "";
        encounter = null;
        players = [];
        bosses = [];
        parties = undefined;
        currentBoss = null;
        encounterDuration = "00:00";
        totalDamageDealt = 0;
        dps = 0;
        timeUntilKill = "00:00";
        isSolo = true;
        anyDead = false;
        anyFrontAtk = false;
        anyBackAtk = false;
        anySupportBuff = false;
        anySupportIdentity = false;
        anySupportBrand = false;
        anyRdpsData = false;
        $rdpsEventDetails = "not_available";
        $missingInfo = false;
    }

    let screenshotAreaDiv: HTMLElement | undefined = $state();

    async function captureScreenshot() {
        takingScreenshot.set(true);
        document.body.style.pointerEvents = "none";
        setTimeout(async () => {
            if (!screenshotAreaDiv) {
                takingScreenshot.set(false);
                return;
            }

            const canvas = await html2canvas(screenshotAreaDiv, {
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

<svelte:window oncontextmenu={(e) => e.preventDefault()} />
<div bind:this={screenshotAreaDiv} style="height: calc(100vh - 1.5rem);">
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
            {#if meterState === MeterState.PARTY}
                <table class="relative w-full table-fixed" id="live-meter-table">
                    <thead class="sticky top-0 z-40 h-6">
                        <tr class="bg-zinc-900 tracking-tighter">
                            <th class="w-7 px-2 font-normal">
                                <MissingInfo />
                            </th>
                            <th class="w-14 px-2 text-left font-normal"></th>
                            <th class="w-full"></th>
                            {#if anyDead && $settings.meter.deathTime}
                                <th class="w-14 font-normal" use:tooltip={{ content: "Dead for" }}>Dead</th>
                            {/if}
                            {#if multipleDeaths && $settings.meter.deathTime}
                                <th class="w-14 font-normal" use:tooltip={{ content: "Death Count" }}>Deaths</th>
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
                            {#if anySupportBrand && $settings.meter.percentBrand}
                                <th class="w-12 font-normal" use:tooltip={{ content: "% Damage buffed by Brand" }}
                                    >B%</th>
                            {/if}
                            {#if anySupportIdentity && $settings.meter.percentIdentityBySup}
                                <th
                                    class="w-12 font-normal"
                                    use:tooltip={{ content: "% Damage buffed by Support Identity" }}
                                    >Iden%
                                </th>
                            {/if}
                            {#if anyRdpsData && $rdpsEventDetails === "" && $settings.meter.ssyn}
                                <th class="w-12 font-normal" use:tooltip={{ content: "% Damage gained from Support" }}
                                    >sSyn%
                                </th>
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
                                onclick={() => inspectPlayer(entity.name)}>
                                <DamageMeterPlayerRow
                                    {entity}
                                    percentage={playerDamagePercentages[i]}
                                    {duration}
                                    {totalDamageDealt}
                                    {lastCombatPacket}
                                    {anyDead}
                                    {multipleDeaths}
                                    {anyFrontAtk}
                                    {anyBackAtk}
                                    {anySupportBuff}
                                    {anySupportIdentity}
                                    {anySupportBrand}
                                    anyRdpsData={anyRdpsData && $rdpsEventDetails === ""}
                                    {isSolo} />
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {:else if meterState === MeterState.PLAYER && player !== null}
                <table class="relative w-full table-fixed" id="live-meter-table">
                    <PlayerBreakdown entity={player} {duration} {handleRightClick} />
                </table>
            {/if}
        {:else if tab === MeterTab.RDPS}
            <Rdps
                {players}
                {duration}
                {totalDamageDealt}
                meterSettings={$settings.meter}
                encounterPartyInfo={parties} />
        {:else if tab === MeterTab.PARTY_BUFFS}
            {#if meterState === MeterState.PARTY}
                <Buffs
                    {tab}
                    encounterDamageStats={encounter?.encounterDamageStats}
                    entities={players}
                    {handleRightClick}
                    {inspectPlayer}
                    encounterPartyInfo={parties}
                    localPlayer={encounter?.localPlayer} />
            {:else}
                <Buffs
                    {tab}
                    encounterDamageStats={encounter?.encounterDamageStats}
                    entities={players}
                    focusedPlayer={player}
                    {handleRightClick}
                    {inspectPlayer}
                    encounterPartyInfo={parties}
                    localPlayer={encounter?.localPlayer} />
            {/if}
        {:else if tab === MeterTab.SELF_BUFFS}
            {#if meterState === MeterState.PARTY}
                <Buffs
                    {tab}
                    encounterDamageStats={encounter?.encounterDamageStats}
                    entities={players}
                    focusedPlayer={player}
                    {handleRightClick}
                    {inspectPlayer}
                    encounterPartyInfo={parties}
                    localPlayer={encounter?.localPlayer} />
            {:else}
                <Buffs
                    {tab}
                    encounterDamageStats={encounter?.encounterDamageStats}
                    entities={players}
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
