<script lang="ts">
    export let encounterDuration: string;
    export let totalDamageDealt: number;
    export let dps: number;
</script>

<div class="px-2 py-1 flex space-x-2 bg-zinc-900/[.6] text-sm">
    <div>
        {encounterDuration}
    </div>
    <div class="flex space-x-1 tracking-tighter text-gray-400">
        <div>
            Total DMG
        </div>
        <div>
            {totalDamageDealt.toLocaleString()}
        </div>
    </div>
    <div class="flex space-x-1 tracking-tighter text-gray-400">
        <div>
            Total DPS
        </div>
        <div>
            {dps.toLocaleString("en", { minimumFractionDigits: 0, maximumFractionDigits: 0 })}
        </div>
    </div>
</div>