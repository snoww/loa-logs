import * as rawData from "./StatOrigin.json";

export type StatOrigin =
  | { t: "s"; i: number } // skill
  | { t: "b" } // bracelet
  | { t: "c"; i: number } // card
  | { t: "ag"; i: number } // ark grid
  | { t: "ap"; i: number } // ark passive
  | { t: "a"; i: number } // ability
  | { t: "it"; i: number } // item (battle items only for now)
  | { t: "gr"; i: number }; // guardian raid NPC mechanic

const data: {
  ability_feature_roots: Record<string, number[]>;
  buff_roots: Record<number, number[]>;
  origins: StatOrigin[];
} = rawData as any;

// order in which stat origins should be normalized; an entry earlier
// in this list is more likely to be shown as the "primary" origin for a stat
const originSortList: StatOrigin["t"][] = ["s", "b", "c", "ag", "ap", "a", "it", "gr"];

export function compareOrigins(a: StatOrigin, b: StatOrigin): number {
  if (a.t !== b.t) return originSortList.indexOf(a.t) - originSortList.indexOf(b.t);
  if (!("i" in a) || !("i" in b)) return 0;
  return a.i - b.i;
}

export function normalizeOrigins(origins: StatOrigin[]): StatOrigin[] {
  return origins.sort(compareOrigins);
}

export function getAbilityFeatureOrigin(feature: string): StatOrigin[] {
  const indices = data.ability_feature_roots[feature] || [];
  return indices.map((i) => data.origins[i]).filter((origin): origin is StatOrigin => origin !== undefined);
}

export function getSkillBuffOrigin(buffId: number): StatOrigin[] {
  const indices = data.buff_roots[buffId] || [];
  return indices.map((i) => data.origins[i]).filter((origin): origin is StatOrigin => origin !== undefined);
}
