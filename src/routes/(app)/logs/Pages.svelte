<script lang="ts">
  import { IconChevronFirst, IconChevronLast, IconChevronLeft, IconChevronRight } from "$lib/icons";
  import { settings } from "$lib/stores.svelte";

  let { page = $bindable(), total }: { page: number; total?: number } = $props();
  let logsPerPage = $derived(settings.app.general.logsPerPage);
  let from = $derived(total === 0 ? 0 : (page - 1) * logsPerPage + 1);
  let to = $derived(Math.min((page - 1) * logsPerPage + logsPerPage, total || 0));
</script>

<div class="flex items-center justify-between p-2 text-sm text-neutral-300">
  <div class="flex items-center gap-2">
    <label for="rowsPerPage">Rows per page:</label>
    <select
      id="rowsPerPage"
      class="focus:border-accent-500 inline rounded-lg border border-neutral-700 bg-neutral-800 p-1 text-sm focus:ring-0"
      onchange={(e) => {
        settings.app.general.logsPerPage = parseInt((e.target as HTMLSelectElement).value);
      }}
    >
      <option selected={logsPerPage === 10}>10</option>
      <option selected={logsPerPage === 25}>25</option>
      <option selected={logsPerPage === 50}>50</option>
      <option selected={logsPerPage === 100}>100</option>
    </select>
    <div class="">
      Showing
      <span class="font-semibold text-white">
        {from}
      </span>
      -
      <span class="font-semibold text-white">
        {to}
      </span>
      of
      <span class="font-semibold text-white">
        {total || 0}
      </span>
    </div>
  </div>
  <div class="flex items-center gap-4 px-1">
    <button
      onclick={() => {
        page = 1;
      }}
    >
      <IconChevronFirst class="hover:text-accent-500 size-5" />
    </button>
    <button
      onclick={() => {
        if (page > 1) {
          page--;
        }
      }}
    >
      <IconChevronLeft class="hover:text-accent-500 size-5" />
    </button>
    <button
      onclick={() => {
        if (page * logsPerPage < (total || 0)) {
          page++;
        }
      }}
    >
      <IconChevronRight class="hover:text-accent-500 size-5" />
    </button>
    <button
      onclick={() => {
        page = Math.ceil((total || 0) / logsPerPage);
      }}
    >
      <IconChevronLast class="hover:text-accent-500 size-5" />
    </button>
  </div>
</div>
