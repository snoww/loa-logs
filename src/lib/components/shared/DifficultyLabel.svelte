<script lang="ts">
    export let difficulty: string | undefined;
    export let hover: boolean = false;

</script>
{#if difficulty === "Normal"}
    <span>[Normal]</span>
{:else if difficulty === "Hard"}
    <span class="text-yellow-300 {hover ? 'group-hover:text-accent-500' : ''}">[{difficulty}]</span>
{:else if difficulty === "Inferno" || difficulty === "Challenge" || difficulty === "Trial"}
    <span class="text-amber-600 {hover ? 'group-hover:text-accent-500' : ''}">[{difficulty}]</span>
{:else if difficulty === "Special"}
    <span class="text-cyan-400 {hover ? 'group-hover:text-accent-500' : ''}">[Special]</span>
{:else if difficulty === "Extreme" || difficulty === "The First"}
    <span class="text-purple-500 {hover ? 'group-hover:text-accent-500' : ''}">[{difficulty}]</span>
{:else}
    &nbsp;
{/if}