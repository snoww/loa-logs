<script lang="ts">
  import { addToast } from "$lib/components/Toaster.svelte";
  import { settings } from "$lib/stores.svelte";
  import { networkSettingsChanged } from "$lib/utils/toasts";
  import { createRadioGroup, createSlider, melt } from "@melt-ui/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";
  import { writable } from "svelte/store";
  import Header from "../Header.svelte";
  import ClassColors from "./ClassColors.svelte";
  import DatabaseInfo from "./DatabaseInfo.svelte";
  import Shortcuts from "./Shortcuts.svelte";

  let currentTab = $state("General");

  let minDuration = writable([settings.app.logs.minEncounterDuration]);
  const {
    elements: { root, range, thumbs, ticks }
  } = createSlider({
    defaultValue: $minDuration,
    value: minDuration,
    min: 0,
    step: 10,
    max: 300
  });

  const {
    elements: { root: radioRoot, item },
    helpers: { isChecked }
  } = createRadioGroup({
    defaultValue: settings.app.mini.info
  });

  const infoOptions = [
    {
      value: "damage",
      label: "Damage %"
    },
    {
      value: "buff",
      label: "Support Buff Summary (Buff/Brand/Identity)"
    }
  ];

  $effect(() => {
    settings.app.logs.minEncounterDuration = $minDuration[0];
  });

  $effect(() => {
    let isMini = settings.app.general.mini;
    (async () => {
      const mini = await WebviewWindow.getByLabel("mini");
      const main = await WebviewWindow.getByLabel("main");
      if (isMini) {
        main?.hide();
        mini?.show();
      } else {
        mini?.hide();
      }
    })();
  });

  onMount(() => {
    (async () => {
      settings.app.general.startOnBoot = await invoke("check_start_on_boot");
    })();
  });

  const themes = [
    {
      name: "theme-red",
      color: "--color-red-500"
    },
    {
      name: "theme-pink",
      color: "--color-pink-500"
    },
    {
      name: "theme-rose",
      color: "oklch(69.9% 0.123 356.37)"
    },
    {
      name: "theme-violet",
      color: "oklch(72.3% 0.15 293.69)"
    },
    {
      name: "theme-purple",
      color: "--color-purple-500"
    },
    {
      name: "theme-blue",
      color: "--color-blue-500"
    },
    {
      name: "theme-green",
      color: "--color-green-500"
    },
    {
      name: "theme-yellow",
      color: "--color-yellow-500"
    },
    {
      name: "theme-orange",
      color: "--color-orange-500"
    }
  ];

  let before = $state(settings.app.general.autoIface);
  let beforePort = $state(settings.app.general.port);
  let networkChanged = $derived(before !== settings.app.general.autoIface || beforePort !== settings.app.general.port);
  let networkNotification = $state(false);

  $effect(() => {
    if (networkChanged) {
      if (!networkNotification) {
        addToast(networkSettingsChanged);
        setTimeout(() => {
          networkNotification = false;
        }, 20000);
      }
    }
  });
</script>

{#snippet settingsTab(tabName: string)}
  <button
    class="focus:outline-hidden text-nowrap rounded-sm px-2 py-1 text-sm text-white transition {tabName === currentTab
      ? 'bg-accent-600/80 border-transparent'
      : 'bg-transparent hover:bg-neutral-700/60'}"
    onclick={() => {
      currentTab = tabName;
    }}
  >
    {tabName}
  </button>
{/snippet}
{#snippet settingOption(category: string, setting: string, name: string, description?: string, breakdown?: boolean)}
  {@const appSettings = settings.app as any}
  <div class="w-fit">
    <label class="flex items-center gap-2">
      {#if !breakdown}
        <input
          type="checkbox"
          bind:checked={appSettings[category][setting]}
          class="form-checkbox checked:text-accent-600/80 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0"
        />
      {:else}
        <input
          type="checkbox"
          bind:checked={appSettings[category]["breakdown"][setting]}
          class="form-checkbox checked:text-accent-600/80 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0"
        />
      {/if}
      <div class="ml-5">
        <div class="text-sm">{name}</div>
        {#if description}
          <div class="text-xs text-neutral-300">{description}</div>
        {/if}
      </div>
    </label>
  </div>
{/snippet}
{#snippet scaleOption(tab: string)}
  <div class="flex items-center gap-2 py-1">
    <div>
      <select
        id="modifiers"
        bind:value={settings.app.general[tab === "meter" ? "scale" : "logScale"]}
        class="focus:ring-accent-500 focus:border-accent-500 w-28 rounded-lg bg-neutral-700 py-1 text-sm placeholder-neutral-400"
      >
        <option value="0">Small</option>
        <option value="1">Normal</option>
        <option value="2">Large</option>
        <option value="3">Largest</option>
      </select>
    </div>
    <div>{tab === "meter" ? "Meter" : "Logs"} UI Scale</div>
  </div>
{/snippet}
{#snippet themeSetting()}
  <div class="flex flex-col gap-2 py-2">
    <div class="text-sm">Color Theme</div>
    <div class="flex items-center gap-2">
      {#each themes as theme}
        {@render themePreview(theme.name, theme.color)}
      {/each}
    </div>
  </div>
{/snippet}
{#snippet themePreview(theme: string, color: string)}
  <button
    class="size-8 rounded-full opacity-90 hover:opacity-100 {theme === settings.app.general.accentColor
      ? 'border-2 border-white'
      : ''}"
    style="background-color: var({color}); background-color: {color}"
    aria-label={theme}
    onclick={() => {
      settings.app.general.accentColor = theme;
    }}
  ></button>
{/snippet}

<Header title="Settings" />
<div class="mx-auto max-w-[180rem] px-8 py-4">
  <div class="flex flex-col gap-2">
    <div class="flex gap-2 overflow-x-auto px-2 max-md:max-w-[100vw]">
      {@render settingsTab("General")}
      {@render settingsTab("Accessibility")}
      {@render settingsTab("Logs")}
      {@render settingsTab("Meter")}
      {#if settings.app.general.mini}
        {@render settingsTab("Mini")}
      {/if}
      {@render settingsTab("Buffs")}
      {@render settingsTab("Colors")}
      {@render settingsTab("Shortcuts")}
      {@render settingsTab("Database")}
    </div>
    <div class="flex flex-col gap-2 px-4 py-2">
      {#if currentTab === "General"}
        {@render themeSetting()}
        {@render settingOption("general", "mini", "Mini Mode", "Experimental horizontal mode for live meter.")}
        {#if settings.app.general.mini}
          {@render settingOption(
            "general",
            "miniEdit",
            "Edit Mini Meter",
            "Enable this setting to drag window to desired position, turn off when finished."
          )}
        {/if}
        {@render settingOption(
          "general",
          "autoShow",
          "Auto Show/Hide",
          "Automatically show and hide meter window when encounter starts and ends."
        )}
        {#if settings.app.general.autoShow}
          <div class="flex items-center">
            <input
              type="number"
              class="form-input h-8 w-12 rounded-md border-0 bg-neutral-700 text-sm focus:ring-0"
              bind:value={settings.app.general.autoHideDelay}
              placeholder={settings.app.general.autoHideDelay.toString()}
            />
            <div class="ml-5">
              <div>Hide Delay</div>
              <div class="text-xs text-neutral-300">
                Set a delay in seconds before the meter hides after an encounter ends.
              </div>
            </div>
          </div>
        {/if}
        {@render settingOption(
          "general",
          "startLoaOnStart",
          "Auto Launch Lost Ark",
          "Automatically start Lost Ark when the app is opened."
        )}
        <label class="flex items-center gap-2">
          <input
            type="checkbox"
            bind:checked={settings.app.general.startOnBoot}
            onchange={async () => {
              await invoke("set_start_on_boot", { set: settings.app.general.startOnBoot });
            }}
            class="form-checkbox checked:text-accent-600 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0"
          />
          <div class="ml-5">
            <div>Start with Windows</div>
            <div class="text-xs text-neutral-300">Automatically start the app when Windows boots up.</div>
          </div>
        </label>
        {@render settingOption(
          "general",
          "lowPerformanceMode",
          "Low Performance Mode",
          "Lowers meter update frequency to reduce CPU usage. (Requires Restart)"
        )}
        {@render settingOption(
          "general",
          "showNames",
          "Show Player Names",
          "Show player names if it's loaded. If disabled, it will show the class name (e.g. Arcanist)."
        )}
        {@render settingOption(
          "general",
          "showGearScore",
          "Show Gear Score",
          "Show player's item level if it's loaded."
        )}
        {@render settingOption(
          "general",
          "hideNames",
          "Hide Names",
          "Hides player names completely, will not show class name either."
        )}
        {@render settingOption(
          "general",
          "showEsther",
          "Show Esther",
          "Show damage dealt by Esther skills in meter and log view"
        )}
        {@render settingOption(
          "general",
          "hideLogo",
          "Hide Logo in Screenshot",
          'Hides the meter name "LOA Logs" in the screenshot.'
        )}
        <label class="flex items-center gap-2">
          <input
            type="checkbox"
            bind:checked={settings.app.general.bossOnlyDamage}
            onchange={() => {
              emit("boss-only-damage-request", settings.app.general.bossOnlyDamage);
            }}
            class="form-checkbox checked:text-accent-600/80 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0"
          />
          <div class="ml-5">
            <div>Boss Only Damage</div>
            <div class="text-xs text-neutral-300">Only track damage dealt to bosses.</div>
          </div>
        </label>
        {@render settingOption(
          "general",
          "bossOnlyDamageDefaultOn",
          "Boss Only Damage Default On",
          "This setting makes it so that the meter will start with boss only damage turned on every time."
        )}
        {@render settingOption(
          "general",
          "showDetails",
          "Show Details",
          "Enables live details tab in live meter for your character."
        )}
        {@render settingOption(
          "general",
          "showRaidsOnly",
          "Show Raids Only",
          "Only show raids in recent encounters (bosses with valid difficulty). Logs from cube, paradise, etc. will be hidden."
        )}
        <label class="flex items-center gap-2">
          <input
            type="checkbox"
            bind:checked={settings.app.general.autoIface}
            onchange={() => {}}
            class="form-checkbox checked:text-accent-600/80 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0"
          />
          <div class="ml-5">
            <div>Auto Port Selection</div>
            <div class="text-xs text-neutral-300">Automatically select port to listen on. (Requires Restart)</div>
          </div>
        </label>
        {#if !settings.app.general.autoIface}
          <div>
            <label class="flex items-center">
              <input
                type="number"
                class="form-input w-18 h-8 rounded-md border-0 bg-neutral-700 text-sm focus:ring-0"
                bind:value={settings.app.general.port}
                placeholder={settings.app.general.port.toString()}
              />
              <div class="ml-5">
                <div>Port</div>
                <div class="text-xs text-neutral-300">
                  Set custom port if not using default. Default is 6040. (Requires Restart)
                </div>
              </div>
            </label>
          </div>
        {/if}
        {@render settingOption(
          "general",
          "experimentalFeatures",
          "Enable Experimental Features",
          "Enables experimental features that may not be fully complete or stable."
        )}
      {:else if currentTab === "Logs"}
        <div class="flex flex-col gap-2">
          <label class="flex items-center justify-between gap-2">
            <div>Minimum Encounter Duration (to be shown)</div>
            <input
              type="text"
              disabled
              class="form-input h-8 w-16 rounded-md border-0 bg-neutral-700 text-sm focus:ring-0"
              value={$minDuration[0] + "s"}
            />
          </label>
          <span use:melt={$root} class="relative flex h-[20px] items-center">
            <span class="h-[3px] w-full bg-neutral-700">
              <span use:melt={$range} class="bg-accent-500/80 h-[3px]"></span>
            </span>

            {#each $ticks as tick}
              <span use:melt={tick} class="h-[3px] w-[3px] rounded-full bg-neutral-300/50"></span>
            {/each}

            <span use:melt={$thumbs[0]} class="h-5 w-5 rounded-full bg-white focus:ring-0"></span>
          </span>
        </div>
        {@render settingOption(
          "logs",
          "abbreviateHeader",
          "Abbreviate Header",
          "Abbreviates the Total DMG and Total DPS numbers in the header"
        )}
        {@render settingOption(
          "logs",
          "splitPartyDamage",
          "Split Party Damage",
          "Split players into their respective parties for damage dealt"
        )}
        {@render settingOption(
          "logs",
          "splitPartyBuffs",
          "Split Party Buffs",
          "Split players into their respective parties for party buffs"
        )}
        {@render settingOption("logs", "deathTime", "Death Time", "Show how long a party member has died")}
        {@render settingOption(
          "logs",
          "incapacitatedTime",
          "Incapacitated Time",
          "Show how long a party member has been incapacitated for (e.g. on the floor, stunned, trapped)"
        )}
        {@render settingOption(
          "logs",
          "damage",
          "Damage",
          "Show the damage dealt by the player in the current encounter"
        )}
        {@render settingOption(
          "logs",
          "damagePercent",
          "Damage %",
          "Show the damage percentage of the player relative to the entire raid"
        )}
        {@render settingOption("logs", "dps", "DPS", "Show the current damage per second")}
        {@render settingOption("logs", "critRate", "Crit Rate", "Show the critical strike rate")}
        {@render settingOption("logs", "critDmg", "Crit Damage", "Show percentage of damage that crit")}
        {@render settingOption("logs", "frontAtk", "Front Attack", "Show the front attack percentage")}
        {@render settingOption("logs", "backAtk", "Back Attack", "Show the back attack percentage")}
        {@render settingOption(
          "logs",
          "positionalDmgPercent",
          "Positional Damage %",
          "Show front/back attack percentage as % of total damage"
        )}
        {@render settingOption(
          "logs",
          "percentBuffBySup",
          "Support Buff %",
          "Show the percentage of damage buffed by support attack power buff"
        )}
        {@render settingOption(
          "logs",
          "percentBrand",
          "Support Brand %",
          "Show the percentage of damage buffed by support's brand skill (e.g. Bard's Sound Shock)"
        )}
        {@render settingOption(
          "logs",
          "percentIdentityBySup",
          "Support Identity %",
          "Show the percentage of damage buffed by support identity"
        )}
        {@render settingOption(
          "logs",
          "percentHatBySup",
          "Support Hyper %",
          "Show the percentage of damage buffed by support hyper awakening skill (T Skill)"
        )}
        {@render settingOption("logs", "counters", "Counters", "Show the number of counters hit")}
        <div class="mt-4 h-px w-full bg-neutral-600"></div>
        <div class="py-2 text-sm">Skill Breakdown</div>
        {@render settingOption("logs", "damage", "Skill Damage", "Show the total damage dealt by the skill", true)}
        {@render settingOption(
          "logs",
          "damagePercent",
          "Skill Damage %",
          "Show the damage percentage of the skill relative to all skills",
          true
        )}
        {@render settingOption("logs", "dps", "Skill DPS", "Show the damage per second of the skill", true)}
        {@render settingOption(
          "logs",
          "critRate",
          "Skill Crit Rate",
          "Show the critical strike rate of the skill",
          true
        )}
        {@render settingOption(
          "logs",
          "adjustedCritRate",
          "Skill Adjusted Crit Rate",
          "Show the adjusted critical strike rate. Hits that are less than 5% of the average cast are excluded. Useful for skills with one big hit and many smaller hits like Doomsday.",
          true
        )}
        {@render settingOption(
          "logs",
          "critDmg",
          "Skill Crit Damage",
          "Show the percentage of damage that crit for the skill",
          true
        )}
        {@render settingOption(
          "logs",
          "frontAtk",
          "Skill Front Attack",
          "Show the front attack percentage of the skill",
          true
        )}
        {@render settingOption(
          "logs",
          "backAtk",
          "Skill Back Attack",
          "Show the back attack percentage of the skill",
          true
        )}
        {@render settingOption(
          "logs",
          "percentBuffBySup",
          "Support Buff %",
          "Show the percentage of damage of the skill buffed by support",
          true
        )}
        {@render settingOption(
          "logs",
          "percentBrand",
          "Support Brand %",
          "Show the percentage of damage of the skill buffed by support's brand skill (e.g. Bard's Sound Shock)",
          true
        )}
        {@render settingOption(
          "logs",
          "percentIdentityBySup",
          "Support Identity %",
          "Show the percentage of damage of the skill buffed by support identity",
          true
        )}
        {@render settingOption(
          "logs",
          "percentHatBySup",
          "Support Hyper %",
          "Show the percentage of damage of the skill buffed by support hyper awakening skill (T Skill)",
          true
        )}
        {@render settingOption(
          "logs",
          "avgDamage",
          "Skill Average Damage",
          "Show the average damage dealt by the skill",
          true
        )}
        {@render settingOption(
          "logs",
          "maxDamage",
          "Skill Max Damage",
          "Show the maximum damage dealt by the skill",
          true
        )}
        {@render settingOption(
          "logs",
          "casts",
          "Skill Casts",
          "Show the total number of casts of the skill (note: cancelled skills still count as cast)",
          true
        )}
        {@render settingOption("logs", "cpm", "Skill Casts/min", "Show the casts per minute of the skill", true)}
        {@render settingOption(
          "logs",
          "hits",
          "Skill Hits",
          "Show the hits of the skill (note: each tick of a multi-hit skill is counted as a hit)",
          true
        )}
        {@render settingOption("logs", "hpm", "Skill Hits/min", "Show the hits per minute of the skill", true)}
      {:else if currentTab === "Meter"}
        {@render settingOption("meter", "bossInfo", "Boss HP Info", "Show boss info in live meter")}
        {@render settingOption(
          "meter",
          "bossHpBar",
          "Boss HP Bars",
          "Show boss HP bars (e.g. x65) for the current boss. turn this off to show hp percentage."
        )}
        {@render settingOption(
          "meter",
          "splitBossHpBar",
          "Split Boss HP Bar",
          "Add vertical bars to the boss hp at 25%, 50%, and 75% intervals."
        )}
        {@render settingOption(
          "meter",
          "profileShortcut",
          "Profile Shortcut",
          "Show uwu shortcut when hovering over player name"
        )}
        {@render settingOption(
          "meter",
          "showTimeUntilKill",
          "Show Time To Kill",
          "Shows approximate time until Boss HP reaches 0"
        )}
        {@render settingOption(
          "meter",
          "showClassColors",
          "Show Class Colors",
          "Shows class colors in the meter. Width of the bar shows relative % damage dealt."
        )}
        {@render settingOption(
          "meter",
          "splitPartyBuffs",
          "Split Party Buffs",
          "Split players into their respective parties for party buffs"
        )}
        {@render settingOption(
          "meter",
          "pinSelfParty",
          "Pin Player Party",
          "Pin the local player's party to the top of the meter in the when party buffs are split."
        )}
        {@render settingOption("meter", "deathTime", "Death Time", "Show how long a party member has died")}
        {@render settingOption(
          "meter",
          "incapacitatedTime",
          "Incapacitated Time",
          "Show how long a party member has been incapacitated for (e.g. on the floor, stunned, trapped)"
        )}
        {@render settingOption("meter", "damage", "Damage", "Show the damage dealt by player in the current encounter")}
        {@render settingOption("meter", "dps", "DPS", "Show the current damage per second")}
        {@render settingOption("meter", "critRate", "Crit Rate", "Show the critical strike rate")}
        {@render settingOption("meter", "critDmg", "Crit Damage", "Show percentage of damage that crit")}
        {@render settingOption("meter", "frontAtk", "Front Attack", "Show the front attack percentage")}
        {@render settingOption("meter", "backAtk", "Back Attack", "Show the back attack percentage")}
        {@render settingOption(
          "meter",
          "positionalDmgPercent",
          "Positional Damage %",
          "Show front/back attack percentage as % of total damage instead of % of hits"
        )}
        {@render settingOption(
          "meter",
          "percentBuffBySup",
          "Support Buff %",
          "Show the percentage of damage buffed by support attack power buff"
        )}
        {@render settingOption(
          "meter",
          "percentBrand",
          "Support Brand %",
          "Show the percentage of damage buffed by support's brand skill (e.g. Bard's Sound Shock)"
        )}
        {@render settingOption(
          "meter",
          "percentIdentityBySup",
          "Support Identity %",
          "Show the percentage of damage buffed by support identity"
        )}
        {@render settingOption(
          "meter",
          "percentHatBySup",
          "Support Hyper %",
          "Show the percentage of damage buffed by support hyper awakening skill"
        )}
        {@render settingOption("meter", "counters", "Counters", "Show the number of counters hit")}
        <div class="mt-4 h-px w-full bg-neutral-600"></div>
        <div class="py-2 text-sm">Skill Breakdown</div>
        {@render settingOption("meter", "damage", "Skill Damage", "Show the total damage dealt by the skill", true)}
        {@render settingOption(
          "meter",
          "damagePercent",
          "Skill Damage %",
          "Show the damage percentage of the skill relative to all skills",
          true
        )}
        {@render settingOption("meter", "dps", "Skill DPS", "Show the damage per second of the skill", true)}
        {@render settingOption(
          "meter",
          "critRate",
          "Skill Crit Rate",
          "Show the critical strike rate of the skill",
          true
        )}
        {@render settingOption(
          "meter",
          "critDmg",
          "Skill Crit Damage",
          "Show the percentage of damage that crit for the skill",
          true
        )}
        {@render settingOption(
          "meter",
          "frontAtk",
          "Skill Front Attack",
          "Show the front attack percentage of the skill",
          true
        )}
        {@render settingOption(
          "meter",
          "backAtk",
          "Skill Back Attack",
          "Show the back attack percentage of the skill",
          true
        )}
        {@render settingOption(
          "meter",
          "percentBuffBySup",
          "Support Buff %",
          "Show the percentage of damage of the skill buffed by support",
          true
        )}
        {@render settingOption(
          "meter",
          "percentBrand",
          "Support Brand %",
          "Show the percentage of damage of the skill buffed by support's brand skill (e.g. Bard's Sound Shock)",
          true
        )}
        {@render settingOption(
          "meter",
          "percentIdentityBySup",
          "Support Identity %",
          "Show the percentage of damage of the skill buffed by support identity",
          true
        )}
        {@render settingOption(
          "meter",
          "percentHatBySup",
          "Support Hyper %",
          "Show the percentage of damage of the skill buffed by support hyper awakening skill (T Skill)",
          true
        )}
        {@render settingOption(
          "meter",
          "avgDamage",
          "Skill Average Damage",
          "Show the average damage dealt by the skill",
          true
        )}
        {@render settingOption(
          "meter",
          "maxDamage",
          "Skill Max Damage",
          "Show the maximum damage dealt by the skill",
          true
        )}
        {@render settingOption(
          "meter",
          "casts",
          "Skill Casts",
          "Show the total number of casts of the skill (note: cancelled skills still count as cast)",
          true
        )}
        {@render settingOption("meter", "cpm", "Skill Casts/min", "Show the casts per minute of the skill", true)}
        {@render settingOption(
          "meter",
          "hits",
          "Skill Hits",
          "Show the hits of the skill (note: each tick of a multi-hit skill is counted as a hit)",
          true
        )}
        {@render settingOption("meter", "hpm", "Skill Hits/min", "Show the hits per minute of the skill", true)}
      {:else if currentTab === "Mini"}
        {@render settingOption(
          "mini",
          "bossHpBar",
          "Boss HP Bars",
          "Show boss HP bars (e.g. x65) for the current boss."
        )}
        <div class="flex flex-col gap-2">
          <p>Player Info</p>
          <div use:melt={$radioRoot} class="flex flex-col gap-2">
            {#each infoOptions as option}
              <div class="flex items-center gap-2">
                <button
                  use:melt={$item(option.value)}
                  class="size-5 cursor-default place-items-center rounded-full bg-neutral-600 shadow-sm hover:bg-neutral-600/80"
                  onclick={() => (settings.app.mini.info = option.value)}
                  id={option.value}
                >
                  {#if $isChecked(option.value)}
                    <div class="bg-accent-500 h-3 w-3 rounded-full"></div>
                  {/if}
                </button>
                <label class="pl-5" for={option.value} id="{option}-label">
                  {option.label}
                </label>
              </div>
            {/each}
          </div>
        </div>
      {:else if currentTab === "Buffs"}
        {@render settingOption(
          "buffs",
          "default",
          "Offensive Buffs Only",
          "Only show Damage, Crit, Atk Speed, and Cooldown buffs. Disabling this will show all buffs"
        )}
      {:else if currentTab === "Accessibility"}
        {@render scaleOption("meter")}
        {@render scaleOption("logs")}
        <label class="flex items-center gap-2">
          <input
            type="checkbox"
            bind:checked={settings.app.general.alwaysOnTop}
            onchange={async () => {
              settings.app.general.alwaysOnTop ? await invoke("enable_aot") : await invoke("disable_aot");
            }}
            class="form-checkbox checked:text-accent-600 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0"
          />
          <div class="ml-5">
            <div>Always on Top</div>
            <div class="text-xs text-neutral-300">Sets the live meter to always be on top of other windows.</div>
          </div>
        </label>
        {@render settingOption(
          "general",
          "constantLocalPlayerColor",
          "Constant Local Player Color",
          "Keeps the color for the local player the same regardless of class. (Change in Class Colors)"
        )}
        {@render settingOption(
          "general",
          "splitLines",
          "Split Lines",
          "Split breakdown lines with alternating background colors for better readability"
        )}
        {@render settingOption(
          "general",
          "underlineHovered",
          "Underline Hovered",
          "Underlines the text in the row when hovering over it for better readability"
        )}
        {@render settingOption(
          "general",
          "hideMeterOnStart",
          "Hide Meter on Launch",
          "Hide the meter window when starting the app."
        )}
        {@render settingOption(
          "general",
          "hideLogsOnStart",
          "Hide Logs on Launch",
          "Hide the logs window when starting the app."
        )}
        {#if settings.app.general.isWin11}
          <label class="flex items-center gap-2">
            <input
              type="checkbox"
              bind:checked={settings.app.general.blurWin11}
              onchange={async () => {
                settings.app.general.blurWin11 ? await invoke("enable_blur") : await invoke("disable_blur");
              }}
              class="form-checkbox checked:text-accent-600 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0"
            />
            <div class="ml-5">
              <div>Blur Meter Background</div>
              <div class="text-xs text-neutral-300">
                Adds background blur effect to live meter (only works on Windows 10).
              </div>
            </div>
          </label>
        {:else}
          <label class="flex items-center gap-2">
            <input
              type="checkbox"
              bind:checked={settings.app.general.blur}
              onchange={async () => {
                settings.app.general.blur ? await invoke("enable_blur") : await invoke("disable_blur");
              }}
              class="form-checkbox checked:text-accent-600 size-5 rounded-sm border-0 bg-neutral-700 focus:ring-0"
            />
            <div class="ml-5">
              <div>Blur Meter Background</div>
              <div class="text-xs text-neutral-300">Adds background blur effect to live meter.</div>
            </div>
          </label>
        {/if}
        {#if settings.app.general.isWin11}
          {@render settingOption(
            "general",
            "transparent",
            "Transparent Meter",
            "Turn off to enable Dark Mode for Windows 11 (with blur setting off)."
          )}
        {:else}
          {@render settingOption(
            "general",
            "transparent",
            "Transparent Meter",
            "Toggle transparent background for live meter."
          )}
        {/if}
      {:else if currentTab === "Database"}
        <DatabaseInfo />
      {:else if currentTab === "Colors"}
        <ClassColors />
      {:else if currentTab === "Shortcuts"}
        <Shortcuts />
      {/if}
    </div>
  </div>
</div>

<style>
  input::-webkit-outer-spin-button,
  input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
</style>
