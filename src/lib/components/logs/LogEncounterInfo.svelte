<script lang="ts">
    
    export let encounterDuration: string;
    export let totalDamageDealt: number;
    export let dps: number;
</script>

<div class="px-2 pt-2 text-sm" id="header">
    <div class="flex space-x-2">
        <div>
            {encounterDuration}
        </div>
        <div class="flex space-x-1 tracking-tighter text-gray-300">
            <div>
                Total DMG:
            </div>
            <div class="text-white">
                {totalDamageDealt.toLocaleString()}
            </div>
        </div>
        <div class="flex space-x-1 tracking-tighter text-gray-300">
            <div>
                Total DPS:
            </div>
            <div class="text-white">
                {dps.toLocaleString("en", { minimumFractionDigits: 0, maximumFractionDigits: 0 })}
            </div>
        </div>
    </div>
</div>