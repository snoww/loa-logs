<script lang="ts">
    import { settings } from "$lib/utils/settings";
    import SettingItem from "./SettingItem.svelte";
</script>

<div class="flex flex-col space-y-4 divide-y-[1px]">
    <div class="mt-4 flex flex-col space-y-2 px-2">
        <SettingItem
            name="Boss HP"
            description="Show the HP bar for the current boss"
            bind:setting={$settings.meter.bossHp} />
        <SettingItem
            name="Boss HP Bars"
            description="Show boss HP bars (e.g. x65), turn off to show HP percentage for all bosses"
            bind:setting={$settings.meter.bossHpBar} />
        <SettingItem
            name="Split Boss HP Bar"
            description="Add vertical bars to the boss hp at 25%, 50%, and 75% intervals."
            bind:setting={$settings.meter.splitBossHpBar} />
        <SettingItem
            name="Abbreviate Header"
            description="Abbreviates the Total DMG and Total DPS numbers in the header"
            bind:setting={$settings.meter.abbreviateHeader} />
        <SettingItem
            name="Show Time To Kill"
            description="Shows approximate time until Boss HP reaches 0"
            bind:setting={$settings.meter.showTimeUntilKill} />
        <SettingItem
            name="Show Class Colors"
            description="Shows class colors in the meter. Width of the bar shows relative % damage dealt."
            bind:setting={$settings.meter.showClassColors} />
        <SettingItem
            name="Split Party Buffs"
            description="Split players into their respective parties for party buffs"
            bind:setting={$settings.meter.splitPartyBuffs} />
        <SettingItem
            name="Pin Player Party"
            description="Pin the local player's party to the top of the meter in the when party buffs are split."
            bind:setting={$settings.meter.pinSelfParty} />
        <SettingItem
            name="Death Time"
            description="Show how long a party member has died"
            bind:setting={$settings.meter.deathTime} />
        <SettingItem
            name="Damage"
            description="Show the damage dealt by player in the current encounter"
            bind:setting={$settings.meter.damage} />
        <SettingItem name="DPS" description="Show the current damage per second" bind:setting={$settings.meter.dps} />
        <SettingItem
            name="Crit Rate"
            description="Show the critical strike rate"
            bind:setting={$settings.meter.critRate} />
        <SettingItem
            name="Crit Damage"
            description="Show percentage of damage that crit"
            bind:setting={$settings.meter.critDmg} />
        <SettingItem
            name="Front Attack"
            description="Show the front attack percentage"
            bind:setting={$settings.meter.frontAtk} />
        <SettingItem
            name="Back Attack"
            description="Show the back attack percentage"
            bind:setting={$settings.meter.backAtk} />
        <SettingItem
            name="Positional Damage %"
            description="Show front/back attack percentage as % of total damage instead of % of hits"
            bind:setting={$settings.meter.positionalDmgPercent} />
        <SettingItem
            name="Support Buff %"
            description="Show the percentage of damage buffed by support attack power buff"
            bind:setting={$settings.meter.percentBuffBySup} />
        <SettingItem
            name="Support Brand %"
            description="Show the percentage of damage buffed by support's brand skill (e.g. Bard's Sound Shock)"
            bind:setting={$settings.meter.percentBrand} />
        <SettingItem
            name="Support Identity %"
            description="Show the percentage of damage buffed by support identity"
            bind:setting={$settings.meter.percentIdentityBySup} />
        <SettingItem
            name="Support Synergy %"
            description="Show the percentage of damage gained from support synergy"
            bind:setting={$settings.meter.ssyn} />
        <SettingItem
            name="Counters"
            description="Show the number of counters hit"
            bind:setting={$settings.meter.counters} />
    </div>
    <div class="pt-4">
        <div>RDPS</div>
        <div class="mt-4 flex flex-col space-y-2 px-2">
            <SettingItem
                name="RDPS Received"
                description="Total damage received from other players"
                bind:setting={$settings.meter.rdpsDamageReceived} />
            <SettingItem
                name="RDPS Given"
                description="Total damage given to other players"
                bind:setting={$settings.meter.rdpsDamageGiven} />
        </div>
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
                name="Skill Crit Damage"
                description="Show the percentage of damage that crit for the skill"
                bind:setting={$settings.meter.breakdown.critDmg} />
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
                description="Show the percentage of damage of the skill buffed by support attack buff"
                bind:setting={$settings.meter.breakdown.percentBuffBySup} />
            <SettingItem
                name="Support Brand %"
                description="Show the percentage of damage of the skill buffed by support's brand skill (e.g. Bard's Sound Shock)"
                bind:setting={$settings.meter.breakdown.percentBrand} />
            <SettingItem
                name="Support Identity %"
                description="Show the percentage of damage of the skill buffed by support identity"
                bind:setting={$settings.meter.breakdown.percentIdentityBySup} />
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
