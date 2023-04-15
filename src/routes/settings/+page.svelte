<script lang="ts">
    import LogSidebar from '$lib/components/logs/LogSidebar.svelte';
    import { Tabs, TabItem } from 'flowbite-svelte';
    import { settings } from '$lib/utils/settings';
    import SettingItem from '$lib/components/settings/SettingItem.svelte';
    import { formatDurationFromS } from '$lib/utils/numbers';

    let hidden: boolean = true;
</script>

<svelte:window on:contextmenu|preventDefault/>
<LogSidebar bind:hidden={hidden} />
<div class="bg-zinc-800 h-screen pb-8 overflow-y-scroll custom-scroll">
        <div class="flex justify-between py-5 px-8 sticky top-0 bg-zinc-800 shadow-md">
            <div class="flex space-x-2 ml-2">
                <div class="">
                    <button on:click={() => (hidden = false)} class="block mt-px">
                        <svg class="fill-gray-300 w-6 h-6 hover:fill-pink-500" xmlns="http://www.w3.org/2000/svg" viewBox="0 96 960 960"><path d="M107 841v-91.5h746.5V841H107Zm0-219.5V530h746.5v91.5H107Zm0-219V310h746.5v92.5H107Z"/></svg>
                    </button>
                </div>
                <div class="text-xl font-bold text-gray-300 pl-2">
                    Settings
                </div>
            </div>
        </div>
        <div class="px-8">
            <Tabs style="underline" contentClass="" defaultClass="flex flex-wrap space-x-2">
                <TabItem title="General" activeClasses="p-4 text-pink-500 border-b border-pink-500" inactiveClasses="p-4 hover:text-gray-200 text-gray-400">
                    <div class="mt-4 px-2">
                        <div>
                            WORK IN PROGRESS
                        </div>
                        <div>
                            show log db stats
                        </div>
                    </div>
                </TabItem>
                <TabItem open title="Live Meter" activeClasses="p-4 text-pink-500 border-b border-pink-500" inactiveClasses="p-4 hover:text-gray-200 text-gray-400">
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
                            <div>
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
                <TabItem title="Logs" activeClasses="p-4 text-pink-500 border-b border-pink-500" inactiveClasses="p-4 hover:text-gray-200 text-gray-400">
                    <div class="flex flex-col space-y-4 divide-y-[1px]">
                        <div class="mt-4 px-2 flex flex-col space-y-2">
                            <label class="font-medium flex flex-col pt-2 pb-4">
                                <div class="flex justify-between px-2 items-center">
                                    <div class="pb-2">
                                        <div class="text-gray-100">Minimum Encounter Duration</div>
                                        <div class="text-xs text-gray-300">Set the minimum time for an encounter for it to be saved to the database</div>
                                    </div>
                                    <div class="bg-zinc-700 px-2 py-1 rounded">
                                        {formatDurationFromS($settings.logs.minEncounterDuration)}
                                    </div>
                                </div>
                                <input type="range" bind:value={$settings.logs.minEncounterDuration} class="accent-pink-700" list="markers" min=0 max=300 step=10/>
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
                <TabItem title="Shortcuts" activeClasses="p-4 text-pink-500 border-b border-pink-500" inactiveClasses="p-4 hover:text-gray-200 text-gray-400">

                </TabItem>
            </Tabs>
        </div>
        <!-- <div>
            <div>
                Show Boss HP
            </div>
            <input type="checkbox" bind:checked={$settings.meter.bossHp} />
        </div> -->
</div>