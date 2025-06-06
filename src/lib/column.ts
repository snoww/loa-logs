import type { Snippet } from "svelte";

export type TooltipContent = string | Snippet | null;

// Column shown for the main damage log interface.
export interface LogColumn<S, E> {
  // Return whether or not the column should be shown.
  show: (state: S) => boolean;
  // Abbreviated name of the column
  headerText: string;
  // Tooltip shown when hovering
  headerTooltip: TooltipContent;
  // Value for a specific player.
  value: Snippet<[E]>;
  // Tooltip shown when hovering over the value
  valueTooltip: null | Snippet<[E]>;
  // width of column
  width?: string;
}
