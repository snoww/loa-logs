export const enum FormattedStringPartType {
  Error = 0,
  Image = 1,
  Colored = 2,
  VariableReference = 3,
  Formula = 4,
  StatName = 5
}

/// Single component of a formatted string
export type FormattedStringPart =
  | string // literal string
  | [FormattedStringPartType.Error, string] // error
  | [FormattedStringPartType.Image, Record<string, string>] // image with attributes
  | [FormattedStringPartType.Colored, string, FormattedStringParts] // push colored text
  | [FormattedStringPartType.VariableReference, string] // reference to a variable
  | [FormattedStringPartType.Formula, FormulaNode, number] // math formula formatted with the given number of decimals
  | [FormattedStringPartType.StatName, number]; // stat name for the given ID

/// Node in a formula expression tree
export type FormulaNode =
  | number // literal number
  | string // variable name
  | [string, FormulaNode, FormulaNode]; // [operator, left, right]

/// Sequence of formatted string parts
export type FormattedStringParts = FormattedStringPart[];

/// A "bound" argument, where there's a list of possible values, and we pick
/// which index to use based on the value of the selector variable.
export type BoundArg = [string, number[]];

/// A formatted string, consisting of parts and a list of bound arguments.
export type FormattedString = [FormattedStringParts, Record<string, BoundArg>?];
