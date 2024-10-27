<script lang="ts">
    import { settings } from "$lib/utils/settings";
    import SettingItem from "./SettingItem.svelte";
</script>

<div class="flex flex-col space-y-4 divide-y-[1px]">
    <div class="mt-4 flex flex-col space-y-2 px-2">
        <SettingItem
            name="Offensive Buffs Only"
            description="Only show Damage, Crit, Atk Speed, and Cooldown buffs. Disabling this will show all buffs."
            bind:setting={$settings.buffs.default} />
    </div>
</div>
