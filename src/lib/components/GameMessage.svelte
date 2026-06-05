<script lang="ts">
  import { statType } from "$lib/constants/stat-type";
  import {
    type FormattedString,
    type FormattedStringPart,
    type FormattedStringParts,
    FormattedStringPartType,
    type FormulaNode
  } from "$lib/constants/formatted";

  type ScaleMode = "min" | "max" | "range" | "each";

  const {
    message,
    imageClass,
    newlineHandling = "space",
    scaledVars = {},
    vars = {}
  }: {
    message: FormattedString;
    vars?: Record<string, string>;
    scaledVars?: Record<string, ScaleMode>;
    imageClass?: string;
    newlineHandling?: "space" | "newline";
  } = $props();

  const parts = $derived(message[0]);
  const boundVars = $derived(message[1] || {});

  // number if success, string if error
  function evalFormula(node: FormulaNode): number | { mode: ScaleMode; values: number[] } | string {
    // literal
    if (typeof node === "number") return node;

    // binop
    if (Array.isArray(node)) {
      const [op, left, right] = node;
      const leftVal = evalFormula(left);
      const rightVal = evalFormula(right);
      if (typeof leftVal !== "number") return leftVal;
      if (typeof rightVal !== "number") return rightVal;
      switch (op) {
        case "+":
          return leftVal + rightVal;
        case "-":
          return leftVal - rightVal;
        case "*":
          return leftVal * rightVal;
        case "/":
          return leftVal / rightVal;
        default:
          return `Unknown operator: ${op}`;
      }
    }

    // variable reference
    if (typeof node !== "string") return `Unsupported formula node`;

    if (node in boundVars) {
      // bound, see if we have a value for a selector
      const [selectorVarName, values] = boundVars[node];
      if (selectorVarName in vars) {
        const index = parseInt(vars[selectorVarName], 10);
        if (index in values) {
          return values[index];
        } else {
          return `Index ${index} out of bounds for variable: ${node}`;
        }
      }

      if (!(selectorVarName in scaledVars)) {
        return `Missing selector variable: ${selectorVarName} for bound variable: ${node}`;
      }

      const scaleMode = scaledVars[selectorVarName];
      return { mode: scaleMode, values };
    } else if (node in vars) {
      return evalFormula(vars[node]);
    }

    return `Missing variable: ${node}`;
  }

  function convertNewlines(text: string): string {
    if (newlineHandling === "space") {
      return text.replace(/\n+/g, " ");
    } else {
      return text.replace(/\n/g, "<br/>");
    }
  }
</script>

{#snippet errorNode(msg: string)}
  <span class="underline decoration-dotted" title={msg}>???</span>
{/snippet}

{#snippet messagePart(part: FormattedStringPart)}
  {#if typeof part === "string"}
    <span>{@html convertNewlines(part)}</span>
  {:else if part[0] === FormattedStringPartType.Error}
    {@render errorNode(part[1])}
  {:else if part[0] === FormattedStringPartType.VariableReference}
    {@const varName = part[1]}
    {#if varName in boundVars}
      {@const [selectorVarName, values] = boundVars[varName]}
      {#if selectorVarName in vars}
        {@const index = parseInt(vars[selectorVarName], 10)}
        {#if index in values}
          <span>{values[index]}</span>
        {:else}
          {@render errorNode(`Index ${index} out of bounds for variable: ${varName}`)}
        {/if}
      {:else}
        {@render errorNode(`Missing selector variable: ${selectorVarName} for bound variable: ${varName}`)}
      {/if}
    {:else if varName in vars}
      <span>{vars[varName]}</span>
    {:else}
      {@render errorNode(`Missing variable: ${varName}`)}
    {/if}
  {:else if part[0] === FormattedStringPartType.Colored}
    <span style="color:{part[1]}">
      {@render messageParts(part[2])}
    </span>
  {:else if part[0] === FormattedStringPartType.Image}
    <img src={part[1]["src"]} alt={part[1]["alt"]} class={imageClass} />
  {:else if part[0] === FormattedStringPartType.StatName}
    {@const [statName, isPercentage] = statType[part[1]] ?? ["Unknown Stat", false]}
    <span>{statName}{isPercentage ? " %" : ""}</span>
  {:else if part[0] === FormattedStringPartType.Formula}
    {@const val = evalFormula(part[1])}
    {@const norm = (v: number) => Math.abs(v).toFixed(part[2])}
    {#if typeof val === "number"}
      <!-- Math.abs is needed to avoid double minus signs, looks like lost ark does it like this too (???) -->
      <span>{norm(val)}</span>
    {:else if typeof val === "object" && "mode" in val}
      {#if val.mode === "min"}
        <span>{norm(val.values[0])}</span>
      {:else if val.mode === "max"}
        <span>{norm(val.values[val.values.length - 1])}</span>
      {:else if val.mode === "range"}
        <span>{`${norm(val.values[0])}~${norm(val.values[val.values.length - 1])}`}</span>
      {:else if val.mode === "each"}
        <span class="wrap-break-word">{val.values.map(norm).join("/")}</span>
      {/if}
    {:else}
      {@render errorNode(`Failed to evaluate formula: ${val}`)}
    {/if}
  {:else}
    {@render errorNode(`Unknown formatted string part type: ${part[0]}`)}
  {/if}
{/snippet}

{#snippet messageParts(parts: FormattedStringParts)}
  {#each parts as part}
    {@render messagePart(part)}
  {/each}
{/snippet}

{@render messageParts(parts)}
