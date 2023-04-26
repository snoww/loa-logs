<script lang="ts">
    import LogSidebar from '$lib/components/logs/LogSidebar.svelte';
    import { Tabs, TabItem } from 'flowbite-svelte';
    import SettingItem from '$lib/components/settings/SettingItem.svelte';
    import { formatDurationFromS } from '$lib/utils/numbers';
    import { registerShortcuts, settings } from '$lib/utils/settings';
    import { onMount } from 'svelte';
    import { backNavStore, pageStore, searchStore } from '$lib/utils/stores';
    import { relaunch } from '@tauri-apps/api/process';

    let dropdownOpen = false;

    let hidden: boolean = true;
    const keys = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
        'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        'F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8', 'F9', 'F10', 'F11', 'F12',
        'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight',
    ];

    $: {
        (async () => {
            registerShortcuts($settings.shortcuts);       
        })();        
    }

    const handleDropdownClick = () => {
        dropdownOpen = !dropdownOpen
    }

    const handleDropdownFocusLoss = (event: FocusEvent) => {
        const relatedTarget = event.relatedTarget as HTMLElement;
        const currentTarget = event.currentTarget as HTMLElement;

        if (currentTarget.contains(relatedTarget)) return;
        dropdownOpen = false;
    };

    async function restartApp() {
        setTimeout(() => {
            relaunch();
        }, 500);
    }

    onMount(() => {
        // dunno if this is good lol XD
        $pageStore = 1;
        $backNavStore = false;
        $searchStore = '';
    });

</script>

<svelte:window on:contextmenu|preventDefault/>
<LogSidebar bind:hidden={hidden} />
<div class="bg-zinc-800 h-screen pb-8 overflow-y-scroll custom-scroll">
    <div class="flex justify-between py-5 px-8 sticky top-0 bg-zinc-800 shadow-md h-16">
        <div class="flex space-x-2 ml-2">
            <div class="">
                <button on:click={() => (hidden = false)} class="block mt-px">
                    <svg class="fill-gray-300 w-6 h-6 hover:fill-accent-500" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="M107 841v-91.5h746.5V841H107Zm0-219.5V530h746.5v91.5H107Zm0-219V310h746.5v92.5H107Z"/></svg>
                </button>
            </div>
            <div class="text-xl font-bold text-gray-300 pl-2">
                Settings
            </div>
        </div>
    </div>
    <div class="px-8">
        <Tabs style="underline" contentClass="" defaultClass="flex flex-wrap space-x-2">
            <TabItem open title="General" activeClasses="p-4 text-accent-500 border-b border-accent-500" inactiveClasses="p-4 hover:text-gray-200 text-gray-400">
                <div class="flex flex-col space-y-4 divide-y-[1px]">
                    <div class="mt-4 px-2 flex flex-col space-y-2">
                        <SettingItem name="Show Names" description="Show player names if it's loaded. If disabled, it will show the class name (e.g. Arcanist)" bind:setting={$settings.general.showNames} />
                        <div class="">
                            <label class="font-medium flex items-center">
                                <input type="checkbox" bind:checked={$settings.general.blur} on:change={restartApp} class="rounded h-5 w-5 text-accent-500 bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                                <div class="ml-5">
                                    <div class="text-gray-100">Blur Meter Background</div>
                                    <div class="text-xs text-gray-300">Makes live meter translucent. Turn this off if experiencing lag (requires restart)</div>
                                </div>
                            </label>
                        </div>
                        <div class="pt-2" on:focusout={handleDropdownFocusLoss}>
                            <div class="flex font-medium items-center">
                                <button id="" class="font-medium rounded-lg text-sm px-2 py-2 text-center inline-flex items-center bg-accent-800" type="button" on:click={handleDropdownClick}>
                                    <svg class="w-4 h-4 fill-white" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="M480 996q-86.035 0-162.566-33.158t-133.825-90.451q-57.293-57.294-90.451-133.802Q60 662.08 60 576.062 60 487 93.196 410.724q33.196-76.275 91.5-133.25Q243 220.5 320.769 187.75 398.538 155 487.189 155q83.023 0 157.706 28.207 74.683 28.207 131.885 77.88 57.202 49.672 90.711 118.242Q901 447.9 901 527q0 112.5-62.75 184.5t-175.664 72H605.5q-17 0-29.5 13.25T563.5 827q0 25.447 10 36.224 10 10.776 10 32.276 0 40-28.55 70.25T480 996Zm0-420Zm-222.5 24.5q19.7 0 34.1-14.4Q306 571.7 306 552q0-19.7-14.4-34.1-14.4-14.4-34.1-14.4-19.7 0-34.1 14.4Q209 532.3 209 552q0 19.7 14.4 34.1 14.4 14.4 34.1 14.4Zm121-162q20.2 0 34.6-14.4 14.4-14.4 14.4-34.1 0-20.7-14.4-34.6-14.4-13.9-34.1-13.9-20.7 0-34.6 13.9-13.9 13.9-13.9 34.1 0 20.2 13.9 34.6 13.9 14.4 34.1 14.4Zm203.5 0q20.2 0 34.6-14.4Q631 409.7 631 390q0-20.7-14.4-34.6-14.4-13.9-34.1-13.9-20.7 0-34.6 13.9-13.9 13.9-13.9 34.1 0 20.2 13.9 34.6 13.9 14.4 34.1 14.4Zm123.5 162q19.7 0 34.1-14.4Q754 571.7 754 552q0-19.7-14.4-34.1-14.4-14.4-34.1-14.4-20.7 0-34.6 14.4Q657 532.3 657 552q0 19.7 13.9 34.1 13.9 14.4 34.6 14.4Zm-229.342 304q7.592 0 11.717-3.545Q492 897.41 492 888.938 492 874.5 477.25 865q-14.75-9.5-14.75-47.5 0-48.674 32.73-87.087Q527.96 692 576.25 692h86.25q74 0 110-43.75t36-115.25q0-131-97.843-208.25t-223.16-77.25q-140.595 0-238.296 95.919T151.5 576.479q0 136.521 95.211 232.271t229.447 95.75Z"/></svg>
                                </button>
                                <div class="ml-5">
                                    <div class="text-gray-100">Accent Color</div>
                                    <div class="text-xs text-gray-300">
                                        Set the accent color for the app
                                    </div>
                                </div>
                            </div>
                            {#if dropdownOpen}
                            <div id="dropdown" class="mt-2 z-10 cursor-pointer shadow w-24 rounded-lg">
                                <ul class="text-sm text-gray-200" aria-labelledby="dropdownDefaultButton">
                                  <li>
                                    <button class="block text-left w-full px-4 py-2 bg-red-800 rounded-t-lg" on:click={() => {$settings.general.accentColor = "theme-red"; dropdownOpen = false}}>Red</button>
                                  </li>
                                  <li>
                                    <button class="block text-left w-full px-4 py-2 bg-pink-800" on:click={() => {$settings.general.accentColor = "theme-pink"; dropdownOpen = false}}>Pink</button>
                                  </li>
                                  <li>
                                    <button class="block text-left w-full px-4 py-2 bg-purple-800" on:click={() => {$settings.general.accentColor = "theme-purple"; dropdownOpen = false}}>Purple</button>
                                  </li>
                                  <li>
                                    <button class="block text-left w-full px-4 py-2 bg-sky-800" on:click={() => {$settings.general.accentColor = "theme-blue"; dropdownOpen = false}}>Blue</button>
                                  </li>
                                  <li>
                                    <button class="block text-left w-full px-4 py-2 bg-green-800" on:click={() => {$settings.general.accentColor = "theme-green"; dropdownOpen = false}}>Green</button>
                                  </li>
                                  <li>
                                    <button class="block text-left w-full px-4 py-2 bg-yellow-400" on:click={() => {$settings.general.accentColor = "theme-yellow"; dropdownOpen = false}}>Yellow</button>
                                  </li>
                                  <li>
                                    <button class="block text-left w-full px-4 py-2 bg-orange-500 rounded-b-lg" on:click={() => {$settings.general.accentColor = "theme-orange"; dropdownOpen = false}}>Orange</button>
                                  </li>
                                </ul>
                            </div>
                            {/if}
                        </div>
                    </div>
                </div>
            </TabItem>
            <TabItem title="Live Meter" activeClasses="p-4 text-accent-500 border-b border-accent-500" inactiveClasses="p-4 hover:text-gray-200 text-gray-400">
                <div class="flex flex-col space-y-4 divide-y-[1px]">
                    <div class="mt-4 px-2 flex flex-col space-y-2">
                        <SettingItem name="Boss HP" description="Show the HP bar for the current boss" bind:setting={$settings.meter.bossHp} />
                        <SettingItem name="Death Time" description="Show how long a party member has died" bind:setting={$settings.meter.deathTime} />
                        <SettingItem name="Damage" description="Show the damage dealt by player in the current encounter" bind:setting={$settings.meter.damage} />
                        <SettingItem name="DPS" description="Show the current damage per second" bind:setting={$settings.meter.dps} />
                        <SettingItem name="Crit Rate" description="Show the critical strike rate" bind:setting={$settings.meter.critRate} />
                        <SettingItem name="Front Attack" description="Show the front attack percentage" bind:setting={$settings.meter.frontAtk} />
                        <SettingItem name="Back Attack" description="Show the back attack percentage" bind:setting={$settings.meter.backAtk} />
                        <SettingItem name="Counters" description="Show the number of counters hit" bind:setting={$settings.meter.counters} />
                    </div>
                    <div class="pt-4">
                        <div class="px-2">
                            Skill Breakdown
                        </div>
                        <div class="mt-4 px-2 flex flex-col space-y-2">
                            <SettingItem name="Skill Damage" description="Show the total damage dealt by the skill" bind:setting={$settings.meter.breakdown.damage} />
                            <SettingItem name="Skill Damage %" description="Show the damage percentage of the skill relative to all skills" bind:setting={$settings.meter.breakdown.damagePercent} />
                            <SettingItem name="Skill DPS" description="Show the damage per second of the skill" bind:setting={$settings.meter.breakdown.dps} />
                            <SettingItem name="Skill Crit Rate" description="Show the critical strike rate of the skill" bind:setting={$settings.meter.breakdown.critRate} />
                            <SettingItem name="Skill Front Attack" description="Show the front attack percentage of the skill" bind:setting={$settings.meter.breakdown.frontAtk} />
                            <SettingItem name="Skill Back Attack" description="Show the back attack percentage of the skill" bind:setting={$settings.meter.breakdown.backAtk} />    
                            <SettingItem name="Skill Average Damage" description="Show the average damage dealt by the skill" bind:setting={$settings.meter.breakdown.avgDamage} />
                            <SettingItem name="Skill Max Damage" description="Show the maximum damage dealt by the skill" bind:setting={$settings.meter.breakdown.maxDamage} />
                            <SettingItem name="Skill Casts/min" description="Show the casts per minute of the skill (note: cancelled skills still count as cast)" bind:setting={$settings.meter.breakdown.casts} />
                            <SettingItem name="Skill Hits/min" description="Show the hits per minute of the skill (note: each tick of a multi-hit skill is counted as a hit)" bind:setting={$settings.meter.breakdown.hits} />
                        </div>
                    </div>
                </div>
            </TabItem>
            <TabItem title="Logs" activeClasses="p-4 text-accent-500 border-b border-accent-500" inactiveClasses="p-4 hover:text-gray-200 text-gray-400">
                <div class="flex flex-col space-y-4 divide-y-[1px]">
                    <div class="mt-4 px-2 flex flex-col space-y-2">
                        <label class="font-medium flex flex-col pt-2 pb-4">
                            <div class="flex justify-between px-2 items-center">
                                <div class="pb-2">
                                    <div class="text-gray-100">Minimum Encounter Duration</div>
                                    <div class="text-xs text-gray-300">Only show encounters that are longer than the specified duration</div>
                                </div>
                                <div class="bg-zinc-700 px-2 py-1 rounded">
                                    {formatDurationFromS($settings.logs.minEncounterDuration)}
                                </div>
                            </div>
                            <input type="range" bind:value={$settings.logs.minEncounterDuration} class="accent-accent-700" list="markers" min=0 max=300 step=10/>
                            <datalist id="markers">
                                {#each Array.from({length: 11}, (_, i) => i * 30) as i}
                                    <option value="{i}"></option>
                                {/each}
                            </datalist>
                        </label>
                        <SettingItem name="Death Time" description="Show how long a party member has died" bind:setting={$settings.logs.deathTime} />
                        <SettingItem name="Damage" description="Show the damage dealt by the player in the current encounter" bind:setting={$settings.logs.damage} />
                        <SettingItem name="Damage %" description="Show the damage percentage of the player relative to the entire raid" bind:setting={$settings.logs.damagePercent} />
                        <SettingItem name="DPS" description="Show the current damage per second" bind:setting={$settings.logs.dps} />
                        <SettingItem name="Crit Rate" description="Show the critical strike rate" bind:setting={$settings.logs.critRate} />
                        <SettingItem name="Front Attack" description="Show the front attack percentage" bind:setting={$settings.logs.frontAtk} />
                        <SettingItem name="Back Attack" description="Show the back attack percentage" bind:setting={$settings.logs.backAtk} />
                        <SettingItem name="Counters" description="Show the number of counters hit" bind:setting={$settings.logs.counters} />
                    </div>
                    <div class="pt-4">
                        <div>
                            Skill Breakdown
                        </div>
                        <div class="mt-4 px-2 flex flex-col space-y-2">
                            <SettingItem name="Skill Damage" description="Show the total damage dealt by the skill" bind:setting={$settings.logs.breakdown.damage} />
                            <SettingItem name="Skill Damage %" description="Show the damage percentage of the skill relative to all skills" bind:setting={$settings.logs.breakdown.damagePercent} />
                            <SettingItem name="Skill DPS" description="Show the damage per second of the skill" bind:setting={$settings.logs.breakdown.dps} />
                            <SettingItem name="Skill Crit Rate" description="Show the critical strike rate of the skill" bind:setting={$settings.logs.breakdown.critRate} />
                            <SettingItem name="Skill Front Attack" description="Show the front attack percentage of the skill" bind:setting={$settings.logs.breakdown.frontAtk} />
                            <SettingItem name="Skill Back Attack" description="Show the back attack percentage of the skill" bind:setting={$settings.logs.breakdown.backAtk} />    
                            <SettingItem name="Skill Average Damage" description="Show the average damage dealt by the skill" bind:setting={$settings.logs.breakdown.avgDamage} />
                            <SettingItem name="Skill Max Damage" description="Show the maximum damage dealt by the skill" bind:setting={$settings.logs.breakdown.maxDamage} />
                            <SettingItem name="Skill Casts/min" description="Show the casts per minute of the skill (note: cancelled skills still count as cast)" bind:setting={$settings.logs.breakdown.casts} />
                            <SettingItem name="Skill Hits/min" description="Show the hits per minute of the skill (note: each tick of a multi-hit skill is counted as a hit)" bind:setting={$settings.logs.breakdown.hits} />

                        </div>
                    </div>
                </div>
            </TabItem>
            <TabItem title="Shortcuts" activeClasses="p-4 text-accent-500 border-b border-accent-500" inactiveClasses="p-4 hover:text-gray-200 text-gray-400">
                <div class="flex flex-col space-y-4 divide-y-[1px]">
                    <div class="mt-4 px-2 flex flex-col space-y-2">
                        <div class="flex justify-between">
                            <label class="font-medium flex items-center" for="modifiers">
                                <div class="">
                                    <div class="text-gray-100">Show/Hide Meter</div>
                                </div>
                            </label>
                            <div class="flex space-x-2 items-center">
                                <select id="modifiers" bind:value={$settings.shortcuts.hideMeter.modifier} class="border text-sm rounded-lg block w-20 p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-accent-500 focus:border-accent-500">
                                    <option value="Ctrl">Ctrl</option>
                                    <option value="Alt">Alt</option>
                                    <option value="Shift"><kbd>Shift</kbd></option>
                                </select>
                                <div>
                                    +
                                </div>
                                <select id="keys" bind:value={$settings.shortcuts.hideMeter.key} class="border text-sm rounded-lg block p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-accent-500 focus:border-accent-500">
                                    {#each keys as key}
                                        <option value={key}>{key.toUpperCase()}</option>
                                    {/each}
                                </select>
                            </div>
                        </div>
                        <div class="flex justify-between">
                            <label class="font-medium flex items-center" for="modifiers">
                                <div class="">
                                    <div class="text-gray-100">Show/Hide Logs</div>
                                </div>
                            </label>
                            <div class="flex space-x-2 items-center">
                                <select id="modifiers" bind:value={$settings.shortcuts.showLogs.modifier} class="border text-sm rounded-lg block w-20 p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-accent-500 focus:border-accent-500">
                                    <option value="Ctrl">Ctrl</option>
                                    <option value="Alt">Alt</option>
                                    <option value="Shift"><kbd>Shift</kbd></option>
                                </select>
                                <div>
                                    +
                                </div>
                                <select id="keys" bind:value={$settings.shortcuts.showLogs.key} class="border text-sm rounded-lg block p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-accent-500 focus:border-accent-500">
                                    {#each keys as key}
                                        <option value={key}>{key.toUpperCase()}</option>
                                    {/each}
                                </select>
                            </div>
                        </div>
                        <div class="flex justify-between">
                            <label class="font-medium flex items-center" for="modifiers">
                                <div class="">
                                    <div class="text-gray-100">Show Most Recent Encounter</div>
                                </div>
                            </label>
                            <div class="flex space-x-2 items-center">
                                <select id="modifiers" bind:value={$settings.shortcuts.showLatestEncounter.modifier} class="border text-sm rounded-lg block w-20 p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-accent-500 focus:border-accent-500">
                                    <option value="Ctrl">Ctrl</option>
                                    <option value="Alt">Alt</option>
                                    <option value="Shift"><kbd>Shift</kbd></option>
                                </select>
                                <div>
                                    +
                                </div>
                                <select id="keys" bind:value={$settings.shortcuts.showLatestEncounter.key} class="border text-sm rounded-lg block p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-accent-500 focus:border-accent-500">
                                    {#each keys as key}
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
</div>