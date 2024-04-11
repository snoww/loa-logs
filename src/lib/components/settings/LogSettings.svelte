<script lang="ts">
    import { formatDurationFromS } from "$lib/utils/numbers";
    import { settings } from "$lib/utils/settings";
    import SettingItem from "./SettingItem.svelte";

</script>
<div class="flex flex-col space-y-4 divide-y-[1px]">
    <div class="mt-4 flex flex-col space-y-2 px-2">
        <label class="flex flex-col pb-4 pt-2">
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
            name="Split Party Buffs"
            description="Split players into their respective parties for party buffs"
            bind:setting={$settings.logs.splitPartyBuffs} />
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
            name="Crit Damage"
            description="Show percentage of damage that crit"
            bind:setting={$settings.logs.critDmg} />
        <SettingItem
            name="Front Attack"
            description="Show the front attack percentage"
            bind:setting={$settings.logs.frontAtk} />
        <SettingItem
            name="Back Attack"
            description="Show the back attack percentage"
            bind:setting={$settings.logs.backAtk} />
        <SettingItem
            name="Crit Damage %"
            description="Show crit damage percentage as % of total damage instead of % of hits"
            bind:setting={$settings.logs.critDmgPercent} />
        <SettingItem
            name="Positional Damage %"
            description="Show front/back attack percentage as % of total damage instead of % of hits"
            bind:setting={$settings.logs.positionalDmgPercent} />
        <SettingItem
            name="Support Buff %"
            description="Show the percentage of damage buffed by support attack power buff"
            bind:setting={$settings.logs.percentBuffBySup} />
        <SettingItem
            name="Support Brand %"
            description="Show the percentage of damage buffed by support's brand skill (e.g. Bard's Sound Shock)"
            bind:setting={$settings.logs.percentBrand} />
        <SettingItem
            name="Support Identity %"
            description="Show the percentage of damage buffed by support identity"
            bind:setting={$settings.logs.percentIdentityBySup} />
        <SettingItem
            name="Support Synergy %"
            description="Show the percentage of damage buffed by support identity"
            bind:setting={$settings.logs.ssyn} />
        <SettingItem
            name="Counters"
            description="Show the percentage of damage gained from support synergy"
            bind:setting={$settings.logs.counters} />
    </div>
    <div class="pt-4">
        <div>RDPS</div>
        <div class="mt-4 flex flex-col space-y-2 px-2">
            <SettingItem
                name="RDPS Received"
                description="Total damage received from other players"
                bind:setting={$settings.logs.rdpsDamageReceived} />
            <SettingItem
                name="RDPS Given"
                description="Total damage given to other players"
                bind:setting={$settings.logs.rdpsDamageGiven} />
        </div>
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
                name="Skill Crit Damage"
                description="Show the percentage of damage that crit for the skill"
                bind:setting={$settings.logs.breakdown.critDmg} />
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
                name="Support Identity %"
                description="Show the percentage of damage of the skill buffed by support identity"
                bind:setting={$settings.logs.breakdown.percentIdentityBySup} />
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