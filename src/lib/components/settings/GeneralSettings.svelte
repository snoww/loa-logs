<script lang="ts">
    import { settings } from "$lib/utils/settings";
    import { invoke } from "@tauri-apps/api";
    import SettingItem from "./SettingItem.svelte";
    import { ifaceChangedStore } from "$lib/utils/stores";
    import { onMount } from "svelte";
    import { emit } from "@tauri-apps/api/event";

    let colorDropdownOpen = false;
    let networkDropdownOpen = false;
    let networkInterfaces: [string, string][];

    const handleColorDropdownFocusLoss = (event: FocusEvent) => {
        const relatedTarget = event.relatedTarget as HTMLElement;
        const currentTarget = event.currentTarget as HTMLElement;

        if (currentTarget.contains(relatedTarget)) return;
        colorDropdownOpen = false;
    };

    const handleColorDropdownClick = () => {
        colorDropdownOpen = !colorDropdownOpen;
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

    onMount(() => {
        (async () => {
            networkInterfaces = await invoke("get_network_interfaces");
            networkInterfaces = networkInterfaces.filter((iface) => iface[1] !== "0.0.0.0");
        })();
    });

    async function toggleAlwaysOnTop() {
        if ($settings.general.alwaysOnTop) {
            await invoke("enable_aot");
        } else {
            await invoke("disable_aot");
        }
    }
</script>

<div class="flex flex-col space-y-4 divide-y-[1px]">
    <div class="mt-4 flex flex-col space-y-2 px-2">
        <div class="relative pt-2" on:focusout={handleColorDropdownFocusLoss}>
            <div class="flex items-center">
                <button
                    id=""
                    class="bg-accent-800 inline-flex items-center rounded-lg px-2 py-2 text-center text-sm"
                    type="button"
                    on:click={handleColorDropdownClick}>
                    <svg class="size-4 fill-white" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"
                        ><path
                            d="M480 996q-86.035 0-162.566-33.158t-133.825-90.451q-57.293-57.294-90.451-133.802Q60 662.08 60 576.062 60 487 93.196 410.724q33.196-76.275 91.5-133.25Q243 220.5 320.769 187.75 398.538 155 487.189 155q83.023 0 157.706 28.207 74.683 28.207 131.885 77.88 57.202 49.672 90.711 118.242Q901 447.9 901 527q0 112.5-62.75 184.5t-175.664 72H605.5q-17 0-29.5 13.25T563.5 827q0 25.447 10 36.224 10 10.776 10 32.276 0 40-28.55 70.25T480 996Zm0-420Zm-222.5 24.5q19.7 0 34.1-14.4Q306 571.7 306 552q0-19.7-14.4-34.1-14.4-14.4-34.1-14.4-19.7 0-34.1 14.4Q209 532.3 209 552q0 19.7 14.4 34.1 14.4 14.4 34.1 14.4Zm121-162q20.2 0 34.6-14.4 14.4-14.4 14.4-34.1 0-20.7-14.4-34.6-14.4-13.9-34.1-13.9-20.7 0-34.6 13.9-13.9 13.9-13.9 34.1 0 20.2 13.9 34.6 13.9 14.4 34.1 14.4Zm203.5 0q20.2 0 34.6-14.4Q631 409.7 631 390q0-20.7-14.4-34.6-14.4-13.9-34.1-13.9-20.7 0-34.6 13.9-13.9 13.9-13.9 34.1 0 20.2 13.9 34.6 13.9 14.4 34.1 14.4Zm123.5 162q19.7 0 34.1-14.4Q754 571.7 754 552q0-19.7-14.4-34.1-14.4-14.4-34.1-14.4-20.7 0-34.6 14.4Q657 532.3 657 552q0 19.7 13.9 34.1 13.9 14.4 34.6 14.4Zm-229.342 304q7.592 0 11.717-3.545Q492 897.41 492 888.938 492 874.5 477.25 865q-14.75-9.5-14.75-47.5 0-48.674 32.73-87.087Q527.96 692 576.25 692h86.25q74 0 110-43.75t36-115.25q0-131-97.843-208.25t-223.16-77.25q-140.595 0-238.296 95.919T151.5 576.479q0 136.521 95.211 232.271t229.447 95.75Z" /></svg>
                </button>
                <div class="ml-5">
                    <div class="text-gray-100">Accent Color</div>
                    <div class="text-xs text-gray-300">Set the accent color for the app</div>
                </div>
            </div>
            {#if colorDropdownOpen}
                <div id="dropdown" class="absolute -left-1 mt-2 w-28 cursor-pointer rounded-lg shadow">
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
                                class="block w-full px-4 py-2 text-left" style="background-color: rgb(218, 124, 160)"
                                on:click={() => {
                                    $settings.general.accentColor = "theme-rose";
                                    colorDropdownOpen = false;
                                }}>Rose</button>
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
                                class="block w-full bg-violet-500 px-4 py-2 text-left"
                                on:click={() => {
                                    $settings.general.accentColor = "theme-violet";
                                    colorDropdownOpen = false;
                                }}>Violet</button>
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
        <SettingItem
            name="Low Performance Mode"
            description="Lowers meter update frequency to reduce CPU usage. (Requires Restart)"
            bind:setting={$settings.general.lowPerformanceMode} />
        <SettingItem
            name="Show Names"
            description="Show player names if it's loaded. If disabled, it will show the class name (e.g. Arcanist)."
            bind:setting={$settings.general.showNames} />
        <SettingItem
            name="Show Gear Score"
            description="Show player's item level if it's loaded."
            bind:setting={$settings.general.showGearScore} />
        <SettingItem
            name="Show Esther"
            description="Show damage dealt by Esther skills in meter and log view."
            bind:setting={$settings.general.showEsther} />
        <label class="flex items-center">
            <input
                type="checkbox"
                bind:checked={$settings.general.bossOnlyDamage}
                on:change={() => {
                    emit("boss-only-damage-request", $settings.general.bossOnlyDamage);
                }}
                class="text-accent-500 size-5 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
            <div class="ml-5">
                <div class="text-gray-100">Boss Only Damage</div>
                <div class="text-xs text-gray-300">Only track damage dealt to bosses.</div>
            </div>
        </label>
        <SettingItem
            name="Boss Only Damage Default On"
            description={"This setting makes it so that the meter will start with boss only damage turned on every time."}
            bind:setting={$settings.general.bossOnlyDamageDefaultOn} />
        <SettingItem
            name="Show Raid Difficulty"
            description={"Shows the difficulty of the raid."}
            bind:setting={$settings.general.showDifficulty} />
        <SettingItem
            name="Show Raid Gate"
            description={"Shows the gate of the raid."}
            bind:setting={$settings.general.showGate} />
        <SettingItem
            name="Show Shield Tab"
            description={"Shows the shield stats for the raid."}
            bind:setting={$settings.general.showShields} />
        <SettingItem
            name="Show Tanked Tab"
            description={"Shows the damage taken by players."}
            bind:setting={$settings.general.showTanked} />
        <SettingItem
            name="Show Bosses Tab"
            description={"Shows the damage dealt by bosses and its skill breakdowns."}
            bind:setting={$settings.general.showBosses} />
        <SettingItem
            name="Show Details Tab"
            description={"Shows an additional tab in meter for raw identity and stagger data."}
            bind:setting={$settings.general.showDetails} />
        <div class="">
            <label class="flex items-center">
                <input
                    type="checkbox"
                    bind:checked={$settings.general.alwaysOnTop}
                    on:change={toggleAlwaysOnTop}
                    class="text-accent-500 size-5 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                <div class="ml-5">
                    <div class="text-gray-100">Always on Top</div>
                    <div class="text-xs text-gray-300">Sets the live meter to always be on top of other windows.</div>
                </div>
            </label>
        </div>
        <SettingItem
            name="Hide Logo in Screenshot"
            description={'Hides the meter name "LOA Logs" in the screenshot.'}
            bind:setting={$settings.general.hideLogo} />
        <div>
            <label class="flex items-center">
                <input
                    type="checkbox"
                    bind:checked={$settings.general.autoIface}
                    on:change={() => {
                        $ifaceChangedStore = true;
                    }}
                    class="text-accent-500 size-5 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                <div class="ml-5">
                    <div class="text-gray-100">Auto Network Selection</div>
                    <div class="text-xs text-gray-300">
                        Automatically select network interface. If using a VPN, turn this off and select the VPN
                        interface.
                    </div>
                </div>
            </label>
        </div>
        {#if !$settings.general.autoIface}
            <div class="relative" on:focusout={handleNetDropdownFocusLoss}>
                <div class="flex items-center">
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
                        role="button"
                        tabindex="0"
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
                        <button
                            class="truncate rounded px-2 py-1 text-left text-sm text-gray-200 hover:bg-gray-700"
                            aria-labelledby="dropdownDefaultButton"
                            on:click={() => {
                                $settings.general.ifDesc = "Default Network Interface";
                                $settings.general.ip = "";
                                networkDropdownOpen = false;
                                $ifaceChangedStore = true;
                            }}>
                            Default Network Interface
                        </button>
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
                                <div class="flex space-x-1">
                                    <div class="w-40 truncate">
                                        {iface[0]}
                                    </div>
                                    <div>
                                        [{iface[1]}]
                                    </div>
                                </div>
                            </button>
                        {/each}
                    </div>
                {/if}
            </div>
            <div>
                <label class="flex items-center">
                    <input
                        type="number"
                        class="h-8 w-24 rounded-md bg-zinc-700 text-sm text-gray-300"
                        bind:value={$settings.general.port}
                        placeholder={$settings.general.port} />
                    <div class="ml-5">
                        <div class="text-gray-100">Port</div>
                        <div class="text-xs text-gray-300">Set custom port if not using default. Default is 6040.</div>
                    </div>
                </label>
            </div>
            <div>
                <label class="flex items-center">
                    <input
                        type="checkbox"
                        bind:checked={$settings.general.rawSocket}
                        on:change={() => {
                            $ifaceChangedStore = true;
                        }}
                        class="text-accent-500 size-5 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                    <div class="ml-5">
                        <div class="text-gray-100">Raw Socket</div>
                        <div class="text-xs text-gray-300">Enables raw socket capture. (manually restart as Admin)</div>
                    </div>
                </label>
            </div>
        {/if}
    </div>
</div>
