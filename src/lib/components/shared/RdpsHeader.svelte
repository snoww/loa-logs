<script lang="ts">
    import { tooltip } from "$lib/utils/tooltip";

</script>

<thead>
    <tr class="bg-zinc-900">
        <th class="w-7 px-2 font-normal" />
        <th class="w-14 px-2 text-left font-normal" />
        <th class="w-full" />
        <th class="w-14 font-normal" use:tooltip={{ content: "Damage dealt without any party synergies + damage given from your synergies" }}>rDMG</th>
        <th class="w-14 font-normal" use:tooltip={{ content: "rDamage per second" }}>rDPS</th>
        <th class="w-14 font-normal" use:tooltip={{ content: "rDamage %" }}>rD%</th>
        <th class="w-14 font-normal" use:tooltip={{ content: "rDamage %" }}>Recv</th>
        <th class="w-14 font-normal" use:tooltip={{ content: "rDamage %" }}>Given</th>
        <th class="w-14 font-normal" use:tooltip={{ content: "% of Damage from all synergies" }}>Syn%</th>
        <th class="w-14 font-normal" use:tooltip={{ content: "% of Damage from Supports" }}>sSyn%</th>
        <th class="w-14 font-normal" use:tooltip={{ content: "% of Damage from Dealers" }}>dSyn%</th>
    </tr>
</thead>
