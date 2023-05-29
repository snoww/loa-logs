<script lang="ts">
    import LogSidebar from "$lib/components/logs/LogSidebar.svelte";
    import { Tabs, TabItem, Alert } from "flowbite-svelte";
    import SettingItem from "$lib/components/settings/SettingItem.svelte";
    import { formatDurationFromS } from "$lib/utils/numbers";
    import { keyboardKeys, registerShortcuts, settings } from "$lib/utils/settings";
    import { onMount } from "svelte";
    import { backNavStore, ifaceChangedStore, pageStore, searchStore } from "$lib/utils/stores";
    import { invoke } from "@tauri-apps/api/tauri";

    let colorDropdownOpen = false;
    let networkDropdownOpen = false;

    let networkInterfaces: [string, string][];

    let hidden: boolean = true;

    $: {
        (async () => {
            registerShortcuts($settings.shortcuts);
        })();
    }

    const handleColorDropdownClick = () => {
        colorDropdownOpen = !colorDropdownOpen;
    };

    const handleColorDropdownFocusLoss = (event: FocusEvent) => {
        const relatedTarget = event.relatedTarget as HTMLElement;
        const currentTarget = event.currentTarget as HTMLElement;

        if (currentTarget.contains(relatedTarget)) return;
        colorDropdownOpen = false;
    };

    const handleNetDropdownClick = () => {
        networkDropdownOpen = !networkDropdownOpen;
    };

    const handleNetDropdownFocusLoss = (event: FocusEvent) => {
        const relatedTarget = event.relatedTarget as HTMLElement;
        const currentTarget = event.currentTarget as HTMLElement;

        if (currentTarget.contains(relatedTarget)) return;
        networkDropdownOpen = false;
    };

    async function toggleBlur() {
        if ($settings.general.blur) {
            await invoke("enable_blur");
        } else {
            await invoke("disable_blur");
        }
    }

    onMount(() => {
        // dunno if this is good lol XD
        $pageStore = 1;
        $backNavStore = false;
        $searchStore = "";
        (async () => {
            networkInterfaces = await invoke("get_network_interfaces");
        })();
    });
</script>

<svelte:window on:contextmenu|preventDefault />
<LogSidebar bind:hidden />
<div class="custom-scroll h-screen overflow-y-scroll bg-zinc-800 pb-8">
    <div class="sticky top-0 flex h-16 justify-between bg-zinc-800 px-8 py-5 shadow-md">
        <div class="ml-2 flex space-x-2">
            <div class="">
                <button on:click={() => (hidden = false)} class="mt-px block">
                    <svg
                        class="hover:fill-accent-500 h-6 w-6 fill-gray-300"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 96 960 960"
                        ><path
                            d="M107 841v-91.5h746.5V841H107Zm0-219.5V530h746.5v91.5H107Zm0-219V310h746.5v92.5H107Z" /></svg>
                </button>
            </div>
            <div class="pl-2 text-xl font-bold text-gray-300">Settings</div>
        </div>
    </div>
    <div class="px-8">
        <Tabs style="underline" contentClass="" defaultClass="flex flex-wrap space-x-2">
            <TabItem
                open
                title="General"
                activeClasses="p-4 text-accent-500 border-b border-accent-500"
                inactiveClasses="p-4 hover:text-gray-200 text-gray-400">
                <div class="flex flex-col space-y-4 divide-y-[1px]">
                    <div class="mt-4 flex flex-col space-y-2 px-2">
                        <SettingItem
                            name="Show Names"
                            description="Show player names if it's loaded. If disabled, it will show the class name (e.g. Arcanist)"
                            bind:setting={$settings.general.showNames} />
                        <SettingItem
                            name="Show Gear Score"
                            description="Show player's item level if it's loaded."
                            bind:setting={$settings.general.showGearScore} />
                        <SettingItem
                            name="Show Esther"
                            description="Show damage dealt by Esther skills in meter and log view."
                            bind:setting={$settings.general.showEsther} />
                        <div class="">
                            <label class="flex items-center font-medium">
                                <input
                                    type="checkbox"
                                    bind:checked={$settings.general.blur}
                                    on:change={toggleBlur}
                                    class="text-accent-500 h-5 w-5 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                                <div class="ml-5">
                                    <div class="text-gray-100">Blur Meter Background</div>
                                    <div class="text-xs text-gray-300">
                                        Makes live meter translucent. Turn this off if experiencing lag.
                                    </div>
                                </div>
                            </label>
                        </div>
                        <SettingItem
                            name="Auto Network Selection"
                            description="Automatically select network interface. If using a VPN, turn this off and select the VPN network."
                            bind:setting={$settings.general.autoIface} />
                        {#if !$settings.general.autoIface}
                            <div class="relative" on:focusout={handleNetDropdownFocusLoss}>
                                <div class="flex items-center font-medium">
                                    <div class="">
                                        <div class="text-gray-100">Select Network Interface</div>
                                        <div class="text-xs text-gray-300">
                                            Select your network interface from the list that Lost Ark is running on.
                                        </div>
                                    </div>
                                </div>
                                <div class="flex items-baseline space-x-2">
                                    <div>Interface:</div>
                                    <!-- svelte-ignore a11y-click-events-have-key-events -->
                                    <div
                                        class="mt-2 w-80 truncate rounded bg-zinc-700 p-1"
                                        on:click={handleNetDropdownClick}>
                                        <div class="px-2">
                                            {#if $settings.general.ifDesc}
                                                {$settings.general.ifDesc}
                                            {:else}
                                                No interface selected. Using default.
                                            {/if}
                                        </div>
                                    </div>
                                </div>
                                {#if networkDropdownOpen}
                                    <div
                                        id="dropdown"
                                        class="absolute left-[4.5rem] z-10 mt-1 flex w-80 cursor-pointer flex-col rounded bg-zinc-600 shadow">
                                        {#each networkInterfaces as iface (iface)}
                                            <button
                                                class="truncate rounded px-2 py-1 text-left text-sm text-gray-200 hover:bg-gray-700"
                                                aria-labelledby="dropdownDefaultButton"
                                                on:click={() => {
                                                    $settings.general.ifDesc = iface[0];
                                                    $settings.general.ip = iface[1];
                                                    networkDropdownOpen = false;
                                                    $ifaceChangedStore = true;
                                                }}>
                                                {iface[0]}
                                            </button>
                                        {/each}
                                    </div>
                                {/if}
                            </div>
                            <div>
                                <label class="flex items-center font-medium">
                                    <input
                                        type="number"
                                        class="h-8 w-24 rounded-md bg-zinc-700 text-sm text-gray-300"
                                        bind:value={$settings.general.port}
                                        placeholder={$settings.general.port} />
                                    <div class="ml-5">
                                        <div class="text-gray-100">Port</div>
                                        <div class="text-xs text-gray-300">
                                            Set custom port if not using default. Default is 6040.
                                        </div>
                                    </div>
                                </label>
                            </div>
                        {/if}
                        <div class="pt-2" on:focusout={handleColorDropdownFocusLoss}>
                            <div class="flex items-center font-medium">
                                <button
                                    id=""
                                    class="bg-accent-800 inline-flex items-center rounded-lg px-2 py-2 text-center text-sm font-medium"
                                    type="button"
                                    on:click={handleColorDropdownClick}>
                                    <svg
                                        class="h-4 w-4 fill-white"
                                        xmlns="http://www.w3.org/2000/svg"
                                        viewBox="0 96 960 960"
                                        ><path
                                            d="M480 996q-86.035 0-162.566-33.158t-133.825-90.451q-57.293-57.294-90.451-133.802Q60 662.08 60 576.062 60 487 93.196 410.724q33.196-76.275 91.5-133.25Q243 220.5 320.769 187.75 398.538 155 487.189 155q83.023 0 157.706 28.207 74.683 28.207 131.885 77.88 57.202 49.672 90.711 118.242Q901 447.9 901 527q0 112.5-62.75 184.5t-175.664 72H605.5q-17 0-29.5 13.25T563.5 827q0 25.447 10 36.224 10 10.776 10 32.276 0 40-28.55 70.25T480 996Zm0-420Zm-222.5 24.5q19.7 0 34.1-14.4Q306 571.7 306 552q0-19.7-14.4-34.1-14.4-14.4-34.1-14.4-19.7 0-34.1 14.4Q209 532.3 209 552q0 19.7 14.4 34.1 14.4 14.4 34.1 14.4Zm121-162q20.2 0 34.6-14.4 14.4-14.4 14.4-34.1 0-20.7-14.4-34.6-14.4-13.9-34.1-13.9-20.7 0-34.6 13.9-13.9 13.9-13.9 34.1 0 20.2 13.9 34.6 13.9 14.4 34.1 14.4Zm203.5 0q20.2 0 34.6-14.4Q631 409.7 631 390q0-20.7-14.4-34.6-14.4-13.9-34.1-13.9-20.7 0-34.6 13.9-13.9 13.9-13.9 34.1 0 20.2 13.9 34.6 13.9 14.4 34.1 14.4Zm123.5 162q19.7 0 34.1-14.4Q754 571.7 754 552q0-19.7-14.4-34.1-14.4-14.4-34.1-14.4-20.7 0-34.6 14.4Q657 532.3 657 552q0 19.7 13.9 34.1 13.9 14.4 34.6 14.4Zm-229.342 304q7.592 0 11.717-3.545Q492 897.41 492 888.938 492 874.5 477.25 865q-14.75-9.5-14.75-47.5 0-48.674 32.73-87.087Q527.96 692 576.25 692h86.25q74 0 110-43.75t36-115.25q0-131-97.843-208.25t-223.16-77.25q-140.595 0-238.296 95.919T151.5 576.479q0 136.521 95.211 232.271t229.447 95.75Z" /></svg>
                                </button>
                                <div class="ml-5">
                                    <div class="text-gray-100">Accent Color</div>
                                    <div class="text-xs text-gray-300">Set the accent color for the app</div>
                                </div>
                            </div>
                            {#if colorDropdownOpen}
                                <div id="dropdown" class="z-10 mt-2 w-24 cursor-pointer rounded-lg shadow">
                                    <ul class="text-sm text-gray-200" aria-labelledby="dropdownDefaultButton">
                                        <li>
                                            <button
                                                class="block w-full rounded-t-lg bg-red-800 px-4 py-2 text-left"
                                                on:click={() => {
                                                    $settings.general.accentColor = "theme-red";
                                                    colorDropdownOpen = false;
                                                }}>Red</button>
                                        </li>
                                        <li>
                                            <button
                                                class="block w-full bg-pink-800 px-4 py-2 text-left"
                                                on:click={() => {
                                                    $settings.general.accentColor = "theme-pink";
                                                    colorDropdownOpen = false;
                                                }}>Pink</button>
                                        </li>
                                        <li>
                                            <button
                                                class="block w-full bg-purple-800 px-4 py-2 text-left"
                                                on:click={() => {
                                                    $settings.general.accentColor = "theme-purple";
                                                    colorDropdownOpen = false;
                                                }}>Purple</button>
                                        </li>
                                        <li>
                                            <button
                                                class="block w-full bg-sky-800 px-4 py-2 text-left"
                                                on:click={() => {
                                                    $settings.general.accentColor = "theme-blue";
                                                    colorDropdownOpen = false;
                                                }}>Blue</button>
                                        </li>
                                        <li>
                                            <button
                                                class="block w-full bg-green-800 px-4 py-2 text-left"
                                                on:click={() => {
                                                    $settings.general.accentColor = "theme-green";
                                                    colorDropdownOpen = false;
                                                }}>Green</button>
                                        </li>
                                        <li>
                                            <button
                                                class="block w-full bg-yellow-400 px-4 py-2 text-left"
                                                on:click={() => {
                                                    $settings.general.accentColor = "theme-yellow";
                                                    colorDropdownOpen = false;
                                                }}>Yellow</button>
                                        </li>
                                        <li>
                                            <button
                                                class="block w-full rounded-b-lg bg-orange-500 px-4 py-2 text-left"
                                                on:click={() => {
                                                    $settings.general.accentColor = "theme-orange";
                                                    colorDropdownOpen = false;
                                                }}>Orange</button>
                                        </li>
                                    </ul>
                                </div>
                            {/if}
                        </div>
                    </div>
                </div>
            </TabItem>
            <TabItem
                title="Live Meter"
                activeClasses="p-4 text-accent-500 border-b border-accent-500"
                inactiveClasses="p-4 hover:text-gray-200 text-gray-400">
                <div class="flex flex-col space-y-4 divide-y-[1px]">
                    <div class="mt-4 flex flex-col space-y-2 px-2">
                        <SettingItem
                            name="Boss HP"
                            description="Show the HP bar for the current boss"
                            bind:setting={$settings.meter.bossHp} />
                        <SettingItem
                            name="Split Boss HP Bar"
                            description="Add vertical bars to the boss hp at 25%, 50%, and 75% intervals."
                            bind:setting={$settings.meter.splitBossHpBar} />
                        <SettingItem
                            name="Abbreviate Header"
                            description="Abbreviates the Total DMG and Total DPS numbers in the header"
                            bind:setting={$settings.meter.abbreviateHeader} />
                        <SettingItem
                            name="Show Class Colors"
                            description="Shows class colors in the meter. Width of the bar shows relative % damage dealt."
                            bind:setting={$settings.meter.showClassColors} />
                        <SettingItem
                            name="Death Time"
                            description="Show how long a party member has died"
                            bind:setting={$settings.meter.deathTime} />
                        <SettingItem
                            name="Damage"
                            description="Show the damage dealt by player in the current encounter"
                            bind:setting={$settings.meter.damage} />
                        <SettingItem
                            name="DPS"
                            description="Show the current damage per second"
                            bind:setting={$settings.meter.dps} />
                        <SettingItem
                            name="Crit Rate"
                            description="Show the critical strike rate"
                            bind:setting={$settings.meter.critRate} />
                        <SettingItem
                            name="Front Attack"
                            description="Show the front attack percentage"
                            bind:setting={$settings.meter.frontAtk} />
                        <SettingItem
                            name="Back Attack"
                            description="Show the back attack percentage"
                            bind:setting={$settings.meter.backAtk} />
                        <SettingItem
                            name="Support Buff %"
                            description="Show the percentage of damage buffed by support"
                            bind:setting={$settings.meter.percentBuffBySup} />
                        <SettingItem
                            name="Support Brand %"
                            description="Show the percentage of damage buffed by support's brand skill (e.g. Bard's Sound Shock)"
                            bind:setting={$settings.meter.percentBrand} />
                        <SettingItem
                            name="Counters"
                            description="Show the number of counters hit"
                            bind:setting={$settings.meter.counters} />
                    </div>
                    <div class="pt-4">
                        <div class="px-2">Skill Breakdown</div>
                        <div class="mt-4 flex flex-col space-y-2 px-2">
                            <SettingItem
                                name="Skill Damage"
                                description="Show the total damage dealt by the skill"
                                bind:setting={$settings.meter.breakdown.damage} />
                            <SettingItem
                                name="Skill Damage %"
                                description="Show the damage percentage of the skill relative to all skills"
                                bind:setting={$settings.meter.breakdown.damagePercent} />
                            <SettingItem
                                name="Skill DPS"
                                description="Show the damage per second of the skill"
                                bind:setting={$settings.meter.breakdown.dps} />
                            <SettingItem
                                name="Skill Crit Rate"
                                description="Show the critical strike rate of the skill"
                                bind:setting={$settings.meter.breakdown.critRate} />
                            <SettingItem
                                name="Skill Front Attack"
                                description="Show the front attack percentage of the skill"
                                bind:setting={$settings.meter.breakdown.frontAtk} />
                            <SettingItem
                                name="Skill Back Attack"
                                description="Show the back attack percentage of the skill"
                                bind:setting={$settings.meter.breakdown.backAtk} />
                            <SettingItem
                                name="Support Buff %"
                                description="Show the percentage of damage of the skill buffed by support"
                                bind:setting={$settings.meter.breakdown.percentBuffBySup} />
                            <SettingItem
                                name="Support Brand %"
                                description="Show the percentage of damage of the skill buffed by support's brand skill (e.g. Bard's Sound Shock)"
                                bind:setting={$settings.meter.breakdown.percentBrand} />
                            <SettingItem
                                name="Skill Average Damage"
                                description="Show the average damage dealt by the skill"
                                bind:setting={$settings.meter.breakdown.avgDamage} />
                            <SettingItem
                                name="Skill Max Damage"
                                description="Show the maximum damage dealt by the skill"
                                bind:setting={$settings.meter.breakdown.maxDamage} />
                            <SettingItem
                                name="Skill Casts"
                                description="Show the total number of casts of the skill (note: cancelled skills still count as cast)"
                                bind:setting={$settings.meter.breakdown.casts} />
                            <SettingItem
                                name="Skill Casts/min"
                                description="Show the casts per minute of the skill"
                                bind:setting={$settings.meter.breakdown.cpm} />
                            <SettingItem
                                name="Skill Hits"
                                description="Show the hits of the skill (note: each tick of a multi-hit skill is counted as a hit)"
                                bind:setting={$settings.meter.breakdown.hits} />
                            <SettingItem
                                name="Skill Hits/min"
                                description="Show the hits per minute of the skill"
                                bind:setting={$settings.meter.breakdown.hpm} />
                        </div>
                    </div>
                </div>
            </TabItem>
            <TabItem
                title="Logs"
                activeClasses="p-4 text-accent-500 border-b border-accent-500"
                inactiveClasses="p-4 hover:text-gray-200 text-gray-400">
                <div class="flex flex-col space-y-4 divide-y-[1px]">
                    <div class="mt-4 flex flex-col space-y-2 px-2">
                        <label class="flex flex-col pb-4 pt-2 font-medium">
                            <div class="flex items-center justify-between px-2">
                                <div class="pb-2">
                                    <div class="text-gray-100">Minimum Encounter Duration</div>
                                    <div class="text-xs text-gray-300">
                                        Only show encounters that are longer than the specified duration
                                    </div>
                                </div>
                                <div class="rounded bg-zinc-700 px-2 py-1">
                                    {formatDurationFromS($settings.logs.minEncounterDuration)}
                                </div>
                            </div>
                            <input
                                type="range"
                                bind:value={$settings.logs.minEncounterDuration}
                                class="accent-accent-700"
                                list="markers"
                                min="0"
                                max="300"
                                step="10" />
                            <datalist id="markers">
                                {#each Array.from({ length: 11 }, (_, i) => i * 30) as i}
                                    <option value={i} />
                                {/each}
                            </datalist>
                        </label>
                        <SettingItem
                            name="Abbreviate Header"
                            description="Abbreviates the Total DMG and Total DPS numbers in the header"
                            bind:setting={$settings.logs.abbreviateHeader} />
                        <SettingItem
                            name="Death Time"
                            description="Show how long a party member has died"
                            bind:setting={$settings.logs.deathTime} />
                        <SettingItem
                            name="Damage"
                            description="Show the damage dealt by the player in the current encounter"
                            bind:setting={$settings.logs.damage} />
                        <SettingItem
                            name="Damage %"
                            description="Show the damage percentage of the player relative to the entire raid"
                            bind:setting={$settings.logs.damagePercent} />
                        <SettingItem
                            name="DPS"
                            description="Show the current damage per second"
                            bind:setting={$settings.logs.dps} />
                        <SettingItem
                            name="Crit Rate"
                            description="Show the critical strike rate"
                            bind:setting={$settings.logs.critRate} />
                        <SettingItem
                            name="Front Attack"
                            description="Show the front attack percentage"
                            bind:setting={$settings.logs.frontAtk} />
                        <SettingItem
                            name="Back Attack"
                            description="Show the back attack percentage"
                            bind:setting={$settings.logs.backAtk} />
                        <SettingItem
                            name="Support Buff %"
                            description="Show the percentage of damage buffed by support"
                            bind:setting={$settings.logs.percentBuffBySup} />
                        <SettingItem
                            name="Support Brand %"
                            description="Show the percentage of damage buffed by support's brand skill (e.g. Bard's Sound Shock)"
                            bind:setting={$settings.logs.percentBrand} />
                        <SettingItem
                            name="Counters"
                            description="Show the number of counters hit"
                            bind:setting={$settings.logs.counters} />
                    </div>
                    <div class="pt-4">
                        <div>Skill Breakdown</div>
                        <div class="mt-4 flex flex-col space-y-2 px-2">
                            <SettingItem
                                name="Skill Damage"
                                description="Show the total damage dealt by the skill"
                                bind:setting={$settings.logs.breakdown.damage} />
                            <SettingItem
                                name="Skill Damage %"
                                description="Show the damage percentage of the skill relative to all skills"
                                bind:setting={$settings.logs.breakdown.damagePercent} />
                            <SettingItem
                                name="Skill DPS"
                                description="Show the damage per second of the skill"
                                bind:setting={$settings.logs.breakdown.dps} />
                            <SettingItem
                                name="Skill Crit Rate"
                                description="Show the critical strike rate of the skill"
                                bind:setting={$settings.logs.breakdown.critRate} />
                            <SettingItem
                                name="Skill Front Attack"
                                description="Show the front attack percentage of the skill"
                                bind:setting={$settings.logs.breakdown.frontAtk} />
                            <SettingItem
                                name="Skill Back Attack"
                                description="Show the back attack percentage of the skill"
                                bind:setting={$settings.logs.breakdown.backAtk} />
                            <SettingItem
                                name="Support Buff %"
                                description="Show the percentage of damage of the skill buffed by support"
                                bind:setting={$settings.logs.breakdown.percentBuffBySup} />
                            <SettingItem
                                name="Support Brand %"
                                description="Show the percentage of damage of the skill buffed by support's brand skill (e.g. Bard's Sound Shock)"
                                bind:setting={$settings.logs.breakdown.percentBrand} />
                            <SettingItem
                                name="Skill Average Damage"
                                description="Show the average damage dealt by the skill"
                                bind:setting={$settings.logs.breakdown.avgDamage} />
                            <SettingItem
                                name="Skill Max Damage"
                                description="Show the maximum damage dealt by the skill"
                                bind:setting={$settings.logs.breakdown.maxDamage} />
                            <SettingItem
                                name="Skill Casts"
                                description="Show the total number of casts of the skill (note: cancelled skills still count as cast)"
                                bind:setting={$settings.logs.breakdown.casts} />
                            <SettingItem
                                name="Skill Casts/min"
                                description="Show the casts per minute of the skill"
                                bind:setting={$settings.logs.breakdown.cpm} />
                            <SettingItem
                                name="Skill Hits"
                                description="Show the hits of the skill (note: each tick of a multi-hit skill is counted as a hit)"
                                bind:setting={$settings.logs.breakdown.hits} />
                            <SettingItem
                                name="Skill Hits/min"
                                description="Show the hits per minute of the skill"
                                bind:setting={$settings.logs.breakdown.hpm} />
                        </div>
                    </div>
                </div>
            </TabItem>
            <TabItem
                title="Shortcuts"
                activeClasses="p-4 text-accent-500 border-b border-accent-500"
                inactiveClasses="p-4 hover:text-gray-200 text-gray-400">
                <div class="flex flex-col space-y-4 divide-y-[1px]">
                    <div class="mt-4 flex flex-col space-y-2 px-2">
                        <div class="flex justify-between">
                            <label class="flex items-center font-medium" for="modifiers">
                                <div class="">
                                    <div class="text-gray-100">Show/Hide Meter</div>
                                </div>
                            </label>
                            <div class="flex items-center space-x-2">
                                <select
                                    id="modifiers"
                                    bind:value={$settings.shortcuts.hideMeter.modifier}
                                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                                    <option value="Ctrl">Ctrl</option>
                                    <option value="Alt">Alt</option>
                                    <option value="Shift"><kbd>Shift</kbd></option>
                                </select>
                                <div>+</div>
                                <select
                                    id="keys"
                                    bind:value={$settings.shortcuts.hideMeter.key}
                                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                                    {#each keyboardKeys as key}
                                        <option value={key}>{key.toUpperCase()}</option>
                                    {/each}
                                </select>
                            </div>
                        </div>
                        <div class="flex justify-between">
                            <label class="flex items-center font-medium" for="modifiers">
                                <div class="">
                                    <div class="text-gray-100">Show/Hide Logs</div>
                                </div>
                            </label>
                            <div class="flex items-center space-x-2">
                                <select
                                    id="modifiers"
                                    bind:value={$settings.shortcuts.showLogs.modifier}
                                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                                    <option value="Ctrl">Ctrl</option>
                                    <option value="Alt">Alt</option>
                                    <option value="Shift"><kbd>Shift</kbd></option>
                                </select>
                                <div>+</div>
                                <select
                                    id="keys"
                                    bind:value={$settings.shortcuts.showLogs.key}
                                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                                    {#each keyboardKeys as key}
                                        <option value={key}>{key.toUpperCase()}</option>
                                    {/each}
                                </select>
                            </div>
                        </div>
                        <div class="flex justify-between">
                            <label class="flex items-center font-medium" for="modifiers">
                                <div class="">
                                    <div class="text-gray-100">Show Most Recent Encounter</div>
                                </div>
                            </label>
                            <div class="flex items-center space-x-2">
                                <select
                                    id="modifiers"
                                    bind:value={$settings.shortcuts.showLatestEncounter.modifier}
                                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                                    <option value="Ctrl">Ctrl</option>
                                    <option value="Alt">Alt</option>
                                    <option value="Shift"><kbd>Shift</kbd></option>
                                </select>
                                <div>+</div>
                                <select
                                    id="keys"
                                    bind:value={$settings.shortcuts.showLatestEncounter.key}
                                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                                    {#each keyboardKeys as key}
                                        <option value={key}>{key.toUpperCase()}</option>
                                    {/each}
                                </select>
                            </div>
                        </div>
                        <div class="flex justify-between">
                            <label class="flex items-center font-medium" for="modifiers">
                                <div class="">
                                    <div class="text-gray-100">Reset Session</div>
                                </div>
                            </label>
                            <div class="flex items-center space-x-2">
                                <select
                                    id="modifiers"
                                    bind:value={$settings.shortcuts.resetSession.modifier}
                                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                                    <option value="Ctrl">Ctrl</option>
                                    <option value="Alt">Alt</option>
                                    <option value="Shift"><kbd>Shift</kbd></option>
                                </select>
                                <div>+</div>
                                <select
                                    id="keys"
                                    bind:value={$settings.shortcuts.resetSession.key}
                                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                                    {#each keyboardKeys as key}
                                        <option value={key}>{key.toUpperCase()}</option>
                                    {/each}
                                </select>
                            </div>
                        </div>
                        <div class="flex justify-between">
                            <label class="flex items-center font-medium" for="modifiers">
                                <div class="">
                                    <div class="text-gray-100">Toggle Pause</div>
                                </div>
                            </label>
                            <div class="flex items-center space-x-2">
                                <select
                                    id="modifiers"
                                    bind:value={$settings.shortcuts.pauseSession.modifier}
                                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                                    <option value="Ctrl">Ctrl</option>
                                    <option value="Alt">Alt</option>
                                    <option value="Shift"><kbd>Shift</kbd></option>
                                </select>
                                <div>+</div>
                                <select
                                    id="keys"
                                    bind:value={$settings.shortcuts.pauseSession.key}
                                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                                    {#each keyboardKeys as key}
                                        <option value={key}>{key.toUpperCase()}</option>
                                    {/each}
                                </select>
                            </div>
                        </div>
                    </div>
                </div>
            </TabItem>
        </Tabs>
    </div>
    {#if $ifaceChangedStore}
        <div>
            <Alert color="none" class="absolute inset-x-0 bottom-10 z-50 mx-auto w-72 bg-red-800 py-2">
                <span slot="icon"
                    ><svg
                        aria-hidden="true"
                        class="h-5 w-5"
                        fill="currentColor"
                        viewBox="0 0 20 20"
                        xmlns="http://www.w3.org/2000/svg"
                        ><path
                            fill-rule="evenodd"
                            d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                            clip-rule="evenodd" /></svg>
                </span>
                Network Interface Changed. Please fully Restart the App.
            </Alert>
        </div>
    {/if}
</div>
