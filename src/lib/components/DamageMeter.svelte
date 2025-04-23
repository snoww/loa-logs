<script lang="ts">
    import { EncounterState } from "$lib/encounter.svelte";
    import { EntityType, MeterState, MeterTab, type Encounter, type EncounterEvent, type PartyEvent } from "$lib/types";
    import { millisToMinutesAndSeconds } from "$lib/utils/numbers";
    import { colors, settings } from "$lib/utils/settings";
    import { localPlayer, missingInfo, screenshotAlert, screenshotError, takingScreenshot } from "$lib/utils/stores";
    import { isValidName } from "$lib/utils/strings";
    import { uploadLog } from "$lib/utils/sync";
    import { invoke } from "@tauri-apps/api";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import html2canvas from "html2canvas-pro";
    import { onDestroy, onMount } from "svelte";
    import { flip } from "svelte/animate";
    import { writable } from "svelte/store";
    import BossInfo from "./BossInfo.svelte";
    import Details from "./Details.svelte";
    import EncounterInfo from "./EncounterInfo.svelte";
    import Footer from "./Footer.svelte";
    import PlayerBreakdown from "./PlayerBreakdown.svelte";
    import BossBreakdown from "./shared/BossBreakdown.svelte";
    import BossTable from "./shared/BossTable.svelte";
    import Buffs from "./shared/Buffs.svelte";
    import DamageMeterHeader from "./shared/DamageMeterHeader.svelte";
    import DamageTaken from "./shared/DamageTaken.svelte";
    import MissingInfo from "./shared/MissingInfo.svelte";
    import Notification from "./shared/Notification.svelte";
    import PlayerRow from "./shared/PlayerRow.svelte";
    import { liveServerListeningAlert } from "$lib/utils/live";

    let time = $state(+Date.now());

    let enc = $derived(new EncounterState(undefined, $settings, true, $colors));

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
            if ($raidInProgress && !$paused) {
                time = +Date.now();
            }
        }, 1000);

        (async () => {
            let encounterUpdateEvent = await listen("encounter-update", (event: EncounterEvent) => {
                // console.log(+Date.now(), event.payload);
                enc.updateEncounter(event.payload);
            });
            let invalidDamageEvent = await listen("invalid-damage", () => {
                $missingInfo = true;
            });
            let partyUpdateEvent = await listen("party-update", (event: PartyEvent) => {
                if (event.payload) {
                    enc.updatePartyInfo(event.payload);
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
                clearEncounterEvent
            );
        })();
    });

    onDestroy(() => {
        events.forEach((unlisten) => unlisten());
    });

    let duration = $derived.by(() => {
        if (enc.encounter && enc.encounter.fightStart !== 0) {
            return time - enc.encounter.fightStart;
        } else {
            return 0;
        }
    });
    let encounterDuration = $derived.by(() => {
        if (duration < 0) {
            return millisToMinutesAndSeconds(0);
        } else {
            return millisToMinutesAndSeconds(duration);
        }
    });
    let totalDps = $derived.by(() => {
        if (duration < 0) {
            return 0;
        } else {
            return enc.totalDamageDealt / (duration / 1000);
        }
    });
    let timeUntilKill = $derived.by(() => {
        if (duration < 0) {
            return "00:00";
        }
        if ($settings.meter.showTimeUntilKill && enc.encounter && enc.encounter.currentBoss) {
            let remainingDpm =
                enc.players
                    .filter((e) => e.damageStats.damageDealt > 0 && !e.isDead && e.entityType == EntityType.PLAYER)
                    .reduce((a, b) => a + b.damageStats.damageDealt, 0) / duration;
            let remainingBossHealth = enc.encounter.currentBoss.currentHp + enc.encounter.currentBoss.currentShield;
            let millisUntilKill = Math.max(remainingBossHealth / remainingDpm, 0);
            if (millisUntilKill > 3.6e6) {
                // 1 hr
                return "∞";
            } else {
                return millisToMinutesAndSeconds(millisUntilKill);
            }
        }

        return "00:00";
    });

    let meterState = $state(MeterState.PARTY);
    let tab = $state(MeterTab.DAMAGE);
    let playerName = $state("");
    let player = $derived.by(() => {
        if (playerName && enc.encounter) {
            return enc.encounter.entities[playerName];
        }
    });
    let focusedBoss = $state("");

    let paused = writable(false);
    $effect(() => {
        enc.updateDuration(duration);
    });

    $effect(() => {
        if (enc.encounter) {
            if (enc.encounter.fightStart !== 0 && !$paused) {
                if (!$missingInfo) {
                    if (enc.encounter.localPlayer === "You" || !isValidName(enc.encounter.localPlayer)) {
                        $missingInfo = true;
                    }
                }
                $localPlayer = enc.encounter.localPlayer;
            }

            if (playerName) {
                meterState = MeterState.PLAYER;
            } else {
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
        enc.reset();
        playerName = "";
        focusedBoss = "";
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

<div bind:this={screenshotAreaDiv} style="height: calc(100vh - 1.5rem);">
    <EncounterInfo
        {encounterDuration}
        totalDamageDealt={enc.totalDamageDealt}
        dps={totalDps}
        {timeUntilKill}
        screenshotFn={captureScreenshot} />
    {#if enc.encounter?.currentBoss && $settings.meter.bossHp}
        <div class="relative top-7">
            <BossInfo boss={enc.encounter?.currentBoss} />
        </div>
    {/if}
    <div
        class="relative top-7 scroll-mt-2 scroll-ml-8 overflow-scroll"
        style="height: calc(100vh - 1.5rem - 1.75rem {enc.encounter?.currentBoss ? ' - 1.75rem' : ''});">
        {#if tab === MeterTab.DAMAGE}
            {#if meterState === MeterState.PARTY}
                <table class="relative w-full table-fixed" id="live-meter-table">
                    <thead class="sticky top-0 z-40 h-6">
                        <tr class="bg-zinc-900 tracking-tighter">
                            <th class="w-7 px-2 font-normal">
                                <MissingInfo />
                            </th>
                            <DamageMeterHeader {enc} />
                        </tr>
                    </thead>
                    <tbody>
                        {#each enc.players as entity, i (entity.name)}
                            <tr
                                class="h-7 px-2 py-1 {$settings.general.underlineHovered ? 'hover:underline' : ''}"
                                animate:flip={{ duration: 200 }}
                                onclick={() => inspectPlayer(entity.name)}>
                                <PlayerRow {entity} {enc} width={enc.playerDamagePercentages[i]} />
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {:else if meterState === MeterState.PLAYER && player !== null}
                <table class="relative w-full table-fixed" id="live-meter-table">
                    <PlayerBreakdown entity={player!} {enc} {handleRightClick} />
                </table>
            {/if}
        {:else if tab === MeterTab.PARTY_BUFFS || tab === MeterTab.SELF_BUFFS}
            <Buffs {tab} {enc} focusedPlayer={player} {inspectPlayer} {handleRightClick} />
        {:else if tab === MeterTab.TANK}
            <DamageTaken {enc} />
        {:else if tab === MeterTab.BOSS}
            {#if !focusedBoss}
                <BossTable {enc} {inspectBoss} />
            {:else}
                <BossBreakdown
                    {enc}
                    boss={enc.encounter!.entities[focusedBoss]}
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
    {#if $liveServerListeningAlert}
        <Notification
            bind:showAlert={$liveServerListeningAlert}
            text="Copied Live Sharing URL To Clipboard"
            width="20rem"
            dismissable={false} />
    {/if}
    <Footer bind:tab />
</div>
