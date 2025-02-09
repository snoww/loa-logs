<script lang="ts">
    interface Props {
        difficulty: string | undefined;
        hover?: boolean;
    }

    let { difficulty, hover = false }: Props = $props();
</script>

{#if difficulty === "Normal"}
    <span>[Normal]</span>
{:else if difficulty === "Hard"}
    <span class="text-yellow-300 {hover ? 'group-hover:text-accent-500' : ''}">[{difficulty}]</span>
{:else if difficulty === "Inferno" || difficulty === "Challenge" || difficulty === "Trial"}
    <span class="text-amber-600 {hover ? 'group-hover:text-accent-500' : ''}">[{difficulty}]</span>
{:else if difficulty === "Solo"}
    <span class="text-cyan-400 {hover ? 'group-hover:text-accent-500' : ''}">[Solo]</span>
{:else if difficulty === "Extreme" || difficulty === "The First"}
    <span class="text-purple-500 {hover ? 'group-hover:text-accent-500' : ''}">[{difficulty}]</span>
{:else}
    &nbsp;
{/if}
