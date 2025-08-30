<script lang="ts">
  import LiveDamageMeter from "./LiveDamageMeter.svelte";
  import { addToast } from "$lib/components/Toaster.svelte";
  import { EncounterState } from "$lib/encounter.svelte";
  import { misc, settings } from "$lib/stores.svelte";
  import type { Encounter, EncounterEvent, PartyEvent } from "$lib/types";
  import { uploadLog } from "$lib/utils/sync";
  import {
    adminAlert,
    bossDead,
    manualSave,
    pausing,
    raidClear,
    raidWipe,
    resetting,
    resuming,
    zoneChange
  } from "$lib/utils/toasts";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { loadEncounter, onAdmin, onClearEncounter, onEncounterUpdate, onInvalidDamage, onPartyUpdate, onPauseEncounter, onPhaseTransition, onRaidStart, onResetEncounter, onSaveEncounter, onZoneChange, unregisterAll } from "$lib/api";

  let enc = $derived(new EncounterState(undefined, true));
  let time = $state(+Date.now());

  onMount(() => {
    const interval = setInterval(() => {
      if (misc.raidInProgress && !misc.paused) {
        time = +Date.now();
      }
    }, 1000);

    (async () => {
      onEncounterUpdate((payload) => {
        if (!settings.app.general.mini) {
          enc.encounter = payload;
        }
      });
      onPartyUpdate((payload) => {
        if (payload) {
          enc.partyInfo = payload as any;
        }
      });
      onInvalidDamage(() => {
        misc.missingInfo = true;
      });
      onZoneChange(() => {
        misc.raidInProgress = false;
        addToast(zoneChange);
        setTimeout(() => {
          misc.raidInProgress = true;
        }, 6000);
      });
      onRaidStart(() => {
        misc.raidInProgress = true;
      });
      onResetEncounter(() => {
        // just need to trigger an update
        misc.reset = !misc.reset;
        addToast(resetting);
      })
      onPauseEncounter(() => {
        if (misc.paused) {
          addToast(pausing);
        } else {
          addToast(resuming);
        }
      })
      onSaveEncounter(() => {
        addToast(manualSave);
        setTimeout(() => {
          misc.reset = !misc.reset;
        }, 1000);
      })
      onPhaseTransition((phaseCode) => {
        if (phaseCode === 1) {
          addToast(bossDead);
        } else if (phaseCode === 2 && misc.raidInProgress) {
          addToast(raidClear);
        } else if (phaseCode === 4 && misc.raidInProgress) {
          addToast(raidWipe);
        }
        misc.raidInProgress = false;
      })
      onAdmin(() => {
        addToast(adminAlert);
      })
      onClearEncounter((id) => {
        if (!settings.sync.auto) {
          return;
        }

        (async () => {
          const encounter = await loadEncounter(id);
          await uploadLog(id, encounter, false);
        })()
      })

    })();

    return () => {
      unregisterAll();
    };
  });

  $effect(() => {
    if (enc.encounter && enc.encounter.fightStart) {
      enc.duration = time - enc.encounter.fightStart;
    } else {
      enc.duration = 0;
    }
  });

  $effect(() => {

    (async () => {
      if (settings.app.general.autoShow && !settings.app.general.mini) {
        const appWindow = getCurrentWindow();
        
        if (misc.raidInProgress && enc.encounter?.currentBossName) {
          await appWindow.show();
        } else {
          // hide with delay
          setTimeout(async () => {
            if (!enc.encounter) {
              await appWindow.hide();
            }
          }, settings.app.general.autoHideDelay);
        }
      }
    })();
  });
</script>

<LiveDamageMeter {enc} />
