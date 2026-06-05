import type { ContributionSplit } from "$lib/types";

export const enum StatSourceType {
  Base = 0,
  Inspect = 1,
  InspectStatDerived = 2,
  InspectDerived = 3,
  InspectDeferred = 4,
  Contribution = 5,
  StatDiff = 6,
  BackAttack = 7,
  Roster = 8,
  Pet = 9,
  Skin = 10,
  SkillTripods = 11,
  BaseAP = 12,
  AttackPowerBaseMultiplier = 13,
  Clamp = 14,
  AbilityFeature = 15,
  SkillBuff = 16,
  Ability = 17,
  Composite = 18
}

export type StatSource =
  | [StatSourceType.InspectStatDerived, number | string] // StatType (enum discrim)
  | [StatSourceType.AbilityFeature, string] // AbilityFeature enum
  | [StatSourceType.SkillBuff, number | string] // SkillBuff id (as string)
  | [StatSourceType.Ability, number | string] // Ability id (as string)
  | [StatSourceType.Composite, StatSource, StatSource] // converter, original
  | [
      Exclude<
        StatSourceType,
        | StatSourceType.InspectStatDerived
        | StatSourceType.AbilityFeature
        | StatSourceType.SkillBuff
        | StatSourceType.Ability
        | StatSourceType.Composite
      >
    ];

// convert a $$id::<rest> string into a StatSource tuple, or null if not a stat source
export function extractStatSource(name: string): StatSource | null {
  if (!name.startsWith("$$") && !name.startsWith("$@")) return null;

  // new format: JSON-encoded
  if (name.startsWith("$@")) {
    return JSON.parse(name.slice(2)) as StatSource;
  }

  // old format: $$<type>::<rest>
  const parts = name.split("::");
  const kind = +parts[0].slice(2) as StatSourceType;
  return [kind, parts[1] ?? ""] as StatSource;
}

export function hasStatSources(split: ContributionSplit): boolean {
  return (
    Object.values(split.damageDoneByEntitySkillGroup).some((group) =>
      Object.keys(group).some((name) => extractStatSource(name.split("/").at(-1)!) !== null)
    ) ||
    Object.values(split.damageIncreaseByEntitySkillGroup).some((group) =>
      Object.keys(group).some((name) => extractStatSource(name.split("/").at(-1)!) !== null)
    )
  );
}
