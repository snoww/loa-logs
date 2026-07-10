<script lang="ts">
  import { IconCalendar, IconChevronLeft, IconChevronRight, IconX } from "$lib/icons";
  import { CalendarDate, type DateValue } from "@internationalized/date";
  import { createDatePicker, melt } from "@melt-ui/svelte";
  import { onDestroy, untrack } from "svelte";
  import { get, writable } from "svelte/store";
  import { fade } from "svelte/transition";

  type Props = {
    date: string;
    onDateChange: (value: string) => void;
    label?: string;
    placeholder?: string;
    maxDate?: string;
  };

  let { date, onDateChange, label = "Date", placeholder = "Select a date", maxDate = "" }: Props = $props();

  const value = writable<DateValue | undefined>(untrack(() => dateFromString(date)));
  let syncingFromProps = false;

  const {
    elements: { calendar, cell, content, field, grid, heading, nextButton, prevButton, trigger },
    states: { months, open, headingValue, weekdays }
  } = createDatePicker({
    value,
    maxValue: untrack(() => dateFromString(maxDate)),
    fixedWeeks: true,
    numberOfMonths: 1,
    weekStartsOn: 1,
    positioning: {
      placement: "bottom-start",
      fitViewport: true,
      sameWidth: false
    }
  });

  const unsubscribeValue = value.subscribe((selectedDate) => {
    if (syncingFromProps) return;
    const nextDate = dateToString(selectedDate);
    if (nextDate !== date) onDateChange(nextDate);
    if (selectedDate) open.set(false);
  });

  onDestroy(unsubscribeValue);

  $effect(() => {
    const currentDate = dateToString(get(value));
    if (currentDate === date) return;

    syncingFromProps = true;
    value.set(dateFromString(date));
    syncingFromProps = false;
  });

  function dateFromString(dateString: string) {
    const [year, month, day] = dateString.split("-").map(Number);
    if (!year || !month || !day) return undefined;
    return new CalendarDate(year, month, day);
  }

  function dateToString(selectedDate: DateValue | undefined) {
    return selectedDate?.toString() ?? "";
  }

  function clearDate() {
    value.set(undefined);
  }

  function isDateAfterMax(day: DateValue) {
    const maxValue = dateFromString(maxDate);
    return maxValue ? day.compare(maxValue) > 0 : false;
  }

  function dayClasses(day: DateValue, monthValue: DateValue) {
    const props = $cell(day, monthValue);
    const isOutsideMonth = props["data-outside-month"] !== undefined;
    const isDisabled = isDateAfterMax(day);
    return [
      "flex size-8 items-center justify-center rounded-md text-xs outline-none transition-colors",
      "hover:bg-neutral-700 focus-visible:ring-1 focus-visible:ring-accent-500",
      isOutsideMonth ? "text-neutral-600" : "text-neutral-200",
      props["data-selected"] ? "bg-accent-600 text-white hover:bg-accent-500" : "",
      props["data-today"] && !props["data-selected"] ? "text-accent-300" : "",
      isDisabled ? "cursor-not-allowed opacity-30 hover:bg-transparent" : ""
    ].join(" ");
  }
</script>

<div class="relative w-fit">
  <div use:melt={$field} class="flex h-9 items-center rounded-md border border-neutral-700 bg-neutral-800">
    <button
      use:melt={$trigger}
      type="button"
      class="flex h-full items-center gap-2 px-2 text-left text-sm whitespace-nowrap text-neutral-200 outline-none hover:bg-neutral-700/50 focus-visible:ring-1 focus-visible:ring-accent-500"
      aria-label={label}
    >
      <IconCalendar class="size-4 text-neutral-400" />
      <span class="truncate">{date || placeholder}</span>
    </button>

    {#if date}
      <button
        type="button"
        class="grid h-full w-8 place-items-center border-l border-neutral-700 text-neutral-500 hover:bg-neutral-700/60 hover:text-neutral-200"
        aria-label="Clear date"
        onclick={clearDate}
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
                {#each $weekdays as weekday, index (`${weekday}-${index}`)}
                  <th class="h-7 text-center text-xxs font-medium text-neutral-500">{weekday}</th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#each month.weeks as week (week.map((day) => day.toString()).join("-"))}
                <tr>
                  {#each week as day (day.toString())}
                    <td role="gridcell" aria-disabled={isDateAfterMax(day) || undefined} class="p-0.5 text-center">
                      <div
                        use:melt={$cell(day, month.value)}
                        aria-disabled={isDateAfterMax(day) || undefined}
                        data-disabled={isDateAfterMax(day) ? "" : undefined}
                        class={dayClasses(day, month.value)}
                      >
                        {day.day}
                      </div>
                    </td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
        {/each}

        <div class="flex justify-start border-t border-neutral-700 pt-3">
          <button
            type="button"
            class="rounded-md px-2 py-1 text-xs text-neutral-400 hover:bg-neutral-700 hover:text-neutral-100"
            onclick={clearDate}
          >
            Clear
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
