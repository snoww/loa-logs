<script lang="ts">
  import { IconCalendar, IconChevronLeft, IconChevronRight, IconX } from "$lib/icons";
  import { CalendarDate, type DateValue } from "@internationalized/date";
  import { createDateRangePicker, melt, type DateRange } from "@melt-ui/svelte";
  import { onDestroy } from "svelte";
  import { get, writable } from "svelte/store";
  import { fade } from "svelte/transition";

  type Props = {
    startDate: string;
    endDate: string;
    onStartDateChange: (value: string) => void;
    onEndDateChange: (value: string) => void;
    label?: string;
  };

  let { startDate, endDate, onStartDateChange, onEndDateChange, label = "Date range" }: Props = $props();

  const value = writable<DateRange>({ start: undefined, end: undefined });
  let syncingFromProps = false;
  let hasSyncedValue = false;

  const {
    elements: { calendar, cell, content, field, grid, heading, nextButton, prevButton, trigger },
    states: { months, open, headingValue, weekdays },
    helpers: { isDateDisabled, isDateUnavailable }
  } = createDateRangePicker({
    value,
    fixedWeeks: true,
    numberOfMonths: 1,
    weekStartsOn: 0,
    positioning: {
      placement: "bottom-start",
      fitViewport: true,
      sameWidth: false
    }
  });

  const unsubscribeValue = value.subscribe((range) => {
    if (syncingFromProps || !hasSyncedValue) return;
    const nextStartDate = dateToString(range?.start);
    const nextEndDate = dateToString(range?.end);
    if (nextStartDate !== startDate) onStartDateChange(nextStartDate);
    if (nextEndDate !== endDate) onEndDateChange(nextEndDate);
  });

  onDestroy(unsubscribeValue);

  $effect(() => {
    const current = get(value);
    const nextStartDate = dateToString(current?.start);
    const nextEndDate = dateToString(current?.end);
    if (nextStartDate === startDate && nextEndDate === endDate) {
      hasSyncedValue = true;
      return;
    }

    syncingFromProps = true;
    value.set(dateRangeFromStrings(startDate, endDate));
    syncingFromProps = false;
    hasSyncedValue = true;
  });

  let displayLabel = $derived(
    startDate || endDate ? `${startDate || "Any start"} - ${endDate || "Any end"}` : "All time"
  );

  function clearRange() {
    value.set({ start: undefined, end: undefined });
    onStartDateChange("");
    onEndDateChange("");
  }

  function closePicker() {
    open.set(false);
  }

  function dateRangeFromStrings(start: string, end: string): DateRange {
    return {
      start: dateFromString(start),
      end: dateFromString(end)
    };
  }

  function dateFromString(value: string) {
    const [year, month, day] = value.split("-").map(Number);
    if (!year || !month || !day) return undefined;
    return new CalendarDate(year, month, day);
  }

  function dateToString(value: DateRange["start"]) {
    return value?.toString() ?? "";
  }

  function dayClasses(date: DateValue, monthValue: DateValue) {
    const props = $cell(date, monthValue);
    return [
      "flex size-8 items-center justify-center rounded-md text-xs outline-none transition-colors",
      "hover:bg-neutral-700 focus-visible:ring-1 focus-visible:ring-accent-500",
      props["data-outside-month"] ? "text-neutral-600" : "text-neutral-200",
      props["data-highlighted"] ? "bg-accent-500/20 text-accent-100" : "",
      props["data-selected"] ? "bg-accent-600 text-white hover:bg-accent-500" : "",
      props["data-today"] && !props["data-selected"] ? "text-accent-300" : "",
      props["data-disabled"] || props["data-unavailable"] ? "cursor-not-allowed opacity-30 hover:bg-transparent" : ""
    ].join(" ");
  }
</script>

<div class="relative">
  <div use:melt={$field} class="flex h-9 items-center rounded-md border border-neutral-700 bg-neutral-800">
    <button
      use:melt={$trigger}
      type="button"
      class="flex h-full min-w-64 items-center gap-2 px-2 text-left text-sm text-neutral-200 outline-none hover:bg-neutral-700/50 focus-visible:ring-1 focus-visible:ring-accent-500"
      aria-label={label}
    >
      <IconCalendar class="size-4 text-neutral-400" />
      <span class="truncate">{displayLabel}</span>
    </button>

    {#if startDate || endDate}
      <button
        type="button"
        class="grid h-full w-8 place-items-center border-l border-neutral-700 text-neutral-500 hover:bg-neutral-700/60 hover:text-neutral-200"
        aria-label="Clear date range"
        onclick={clearRange}
      >
        <IconX class="size-4" />
      </button>
    {/if}
  </div>

  {#if $open}
    <div
      use:melt={$content}
      transition:fade={{ duration: 100 }}
      class="z-50 mt-1 w-[18rem] rounded-md border border-neutral-700 bg-neutral-800 p-3 text-neutral-100 shadow-xl"
    >
      <div use:melt={$calendar} class="flex flex-col gap-3">
        <div class="flex items-center justify-between">
          <button
            use:melt={$prevButton}
            type="button"
            class="grid size-8 place-items-center rounded-md text-neutral-300 hover:bg-neutral-700 disabled:cursor-default disabled:opacity-40 disabled:hover:bg-transparent"
          >
            <IconChevronLeft class="size-4" />
          </button>

          <div use:melt={$heading} class="text-sm font-medium text-neutral-100">{$headingValue}</div>

          <button
            use:melt={$nextButton}
            type="button"
            class="grid size-8 place-items-center rounded-md text-neutral-300 hover:bg-neutral-700 disabled:cursor-default disabled:opacity-40 disabled:hover:bg-transparent"
          >
            <IconChevronRight class="size-4" />
          </button>
        </div>

        {#each $months as month (month.value.toString())}
          <table use:melt={$grid} class="w-full table-fixed border-collapse">
            <thead aria-hidden="true">
              <tr>
                {#each $weekdays as day, index (`${day}-${index}`)}
                  <th class="h-7 text-center text-[0.7rem] font-medium text-neutral-500">{day}</th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#each month.weeks as week (week.map((date) => date.toString()).join("-"))}
                <tr>
                  {#each week as date (date.toString())}
                    <td
                      role="gridcell"
                      aria-disabled={$isDateDisabled(date) || $isDateUnavailable(date)}
                      class="p-0.5 text-center"
                    >
                      <div use:melt={$cell(date, month.value)} class={dayClasses(date, month.value)}>
                        {date.day}
                      </div>
                    </td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
        {/each}

        <div class="flex justify-between border-t border-neutral-700 pt-3">
          <button
            type="button"
            class="rounded-md px-2 py-1 text-xs text-neutral-400 hover:bg-neutral-700 hover:text-neutral-100"
            onclick={clearRange}
          >
            Clear
          </button>
          <button
            type="button"
            class="rounded-md bg-accent-600 px-2 py-1 text-xs font-medium text-white hover:bg-accent-500"
            onclick={closePicker}
          >
            Apply
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
