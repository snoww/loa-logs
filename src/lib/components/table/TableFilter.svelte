<script lang="ts">
    export let search: string;

</script>
<div class="flex justify-between items-center">
    <div>
        <div class="relative">
            <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
                <svg class="w-5 h-5 text-gray-400" aria-hidden="true" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd"></path></svg>
            </div>            
            <input type="text" bind:value={search} class="block p-2 pl-10 text-sm border rounded-lg w-80 bg-zinc-700 border-gray-600 placeholder-gray-400 text-zinc-300 focus:ring-0 focus:border-accent-500"
                placeholder="Search encounters, names, or classes">
        </div>
    </div>
</div>