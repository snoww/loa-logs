export type DamageAttributeType = "NONE" | "FIRE" | "ICE" | "ELECTRICITY" | "WIND" | "EARTH" | "DARK" | "HOLY";

export type StatDataGroup =
  | "weapon_power_"
  | "weapon_dam_x_"
  | "attack_power_base_multiplier_"
  | "attack_power_rate_"
  | "str_stat_"
  | "dex_stat_"
  | "int_stat_"
  | "str_multiplier_stat_"
  | "dex_multiplier_stat_"
  | "int_multiplier_stat_"
  | "critical_hit_stat_"
  | "move_speed_rate_"
  | "attack_speed_rate_"
  | "ally_identity_damage_power_"
  | "ally_attack_power_power_"
  | "ally_brand_power_"
  | "evolution_damage_"
  | "modify_damage_combat_effect_"
  | "spec_bonus_identity_1_"
  | "spec_bonus_identity_2_"
  | "spec_bonus_identity_3_"
  | "critical_hit_rate_"
  | "critical_damage_rate_"
  | "critical_damage_rate_2_"
  | "attack_power_addend_"
  | "attack_power_addend_2_"
  | "attack_power_sub_rate_1_"
  | "attack_power_sub_rate_2_"
  | "skill_damage_sub_rate_1_"
  | "skill_damage_sub_rate_2_"
  | "skill_damage_rate_"
  | "ultimate_awakening_damage_rate_"
  | "move_speed_to_damage_rate_"
  | "physical_defense_break_"
  | "magical_defense_break_"
  | "outgoing_dmg_stat_amp_"
  | "skill_damage_amplify_"
  | "front_attack_amplify_"
  | "back_attack_amplify_"
  | "physical_critical_damage_amplify_"
  | "magical_critical_damage_amplify_"
  | "critical_hit_to_damage_rate_"
  | "evolution_damage_bonus_from_blunt_thorn_"
  | "evolution_damage_bonus_from_supersonic_breakthrough_"
  | `damage_attr_amplifications_[${DamageAttributeType}]`
  | `damage_attr_rates_[${DamageAttributeType}]`;

// either a direct description, or a dynamic description that depends on class/spec
export type StatDataDescription = string | ((spec: string) => string);

export type StatDataDescriptor = {
  title: StatDataDescription;
  help: StatDataDescription;
  type: "additive" | "multiplicative" | "singular";
};

const simpleStat = (name: string, extra?: string): StatDataDescriptor => ({
  title: `Stat - ${name}`,
  help: `Damage directly influenced by the amount of ${name} a character has.${extra ? ` ${extra}` : ""}`,
  type: "additive"
});

const unused = (name: string): StatDataDescriptor => ({
  title: `UNUSED - Please Report If You See This!`,
  help: name,
  type: "additive"
});

const attrAmplification = (typeName: string): StatDataDescriptor => ({
  title: `${typeName} Damage Dealt Increase %`,
  help: `Damage directly influenced by effects that increase all ${typeName}-type damage on the target.`,
  type: "additive"
});

const attrRates = (typeName: string): StatDataDescriptor => ({
  title: `${typeName} Damage Increase %`,
  help: `Damage directly influenced by effects that increase all ${typeName}-type damage dealt by the character.`,
  type: "additive"
});

export const statDataDescriptors: Record<StatDataGroup, StatDataDescriptor> = {
  weapon_power_: simpleStat("Weapon Power"),
  weapon_dam_x_: simpleStat("Weapon Power %", "This stat is provided primarily by Earring accessory lines."),
  attack_power_base_multiplier_: simpleStat(
    "Basic Atk. Power (Gems)",
    "This stat represents the additional Atk. Power percentage provided by tier 4 gems."
  ),
  attack_power_rate_: simpleStat(
    "Atk. Power %",
    "This stat comes from percentage Atk. Power sources, such as accessories, astrogems, and strength orbs."
  ),
  str_stat_: simpleStat("Strength"),
  dex_stat_: simpleStat("Dexterity"),
  int_stat_: simpleStat("Intelligence"),
  str_multiplier_stat_: simpleStat(
    "Strength %",
    "This stat comes from tier 1 weapons and some rare other class-specific effects."
  ),
  dex_multiplier_stat_: simpleStat(
    "Dexterity %",
    "This stat comes from tier 1 weapons and some rare other class-specific effects."
  ),
  int_multiplier_stat_: simpleStat(
    "Intelligence %",
    "This stat comes from tier 1 weapons and some rare other class-specific effects."
  ),
  critical_hit_stat_: simpleStat("Crit"),
  move_speed_rate_: simpleStat(
    "Move Speed %",
    "This stat represents all bonuses to movement speed, capped at 40%. This stat is derived from the swiftness stat, as well as additional bonus effects from skills and (party) buffs."
  ),
  attack_speed_rate_: simpleStat(
    "Attack Speed %",
    "This stat represents all bonuses to attack speed, capped at 40%. This stat is derived from the swiftness stat, as well as additional bonus effects from skills and (party) buffs."
  ),
  ally_identity_damage_power_: unused("ally_identity_damage_power_"),
  ally_attack_power_power_: simpleStat("Ally Atk. Power Enhancement Effect %"),
  ally_brand_power_: simpleStat("Brand Power %"),
  evolution_damage_: {
    title: "Evolution-Type Damage Increases",
    help: 'Damage directly influenced for effects that increase "Evolution-Type Damage", which contains all tier 5 evolution Ark Passive nodes, as well as evolution karma progress.',
    type: "additive"
  },
  modify_damage_combat_effect_: {
    title: "On-Hit Damage Multipliers",
    help: 'Collection group for all flat multiplicative damage modifiers. Most of these effects come from Ark Grid core lines and bracelets. The game usually describes these effects as "[Outgoing/Back Attack/Frontal Attack/Skill] Damage +X%".',
    type: "multiplicative"
  },
  spec_bonus_identity_1_: unused("spec_bonus_identity_1_"),
  spec_bonus_identity_2_: unused("spec_bonus_identity_2_"),
  spec_bonus_identity_3_: unused("spec_bonus_identity_3_"),
  critical_hit_rate_: {
    title: "Crit Rate %",
    help: "Damage directly influenced by additional crit rate sources, such as accessories, synergies, Precise Dagger, and flash orbs.",
    type: "additive"
  },
  critical_damage_rate_: {
    title: "Flat Crit Damage %",
    help: 'Damage directly influenced by additional crit damage sources, such as accessories, synergies, and Keen Blunt Weapon. Multiplicative with effects classified as "on-hit" crit damage increases',
    type: "additive"
  },
  critical_damage_rate_2_: {
    title: "On-Hit Crit Damage %",
    help: 'Damage directly influenced by sources that are classified as "on-hit" crit damage increases. The game usually describes these effects as "On Crit Hit, Damage (to foes) +X%". These effects are multiplicative with flat crit damage increases.',
    type: "additive"
  },
  attack_power_addend_: {
    title: "Support Bonus Atk. Power",
    help: "Damage directly influenced by the bonus Atk. Power provided by support attack power buffs.",
    type: "additive"
  },
  attack_power_addend_2_: {
    title: "Bonus Atk. Power (Ark Grid)",
    help: "Damage directly influenced by the bonus Atk. Power granted by the Attack Ark Grid core.",
    type: "additive"
  },
  attack_power_sub_rate_1_: {
    title: "Atk. Power % (Buffs)",
    help: "Damage directly influenced by increases in Atk. Power. Primarily comes from status effects.",
    type: "additive"
  },
  attack_power_sub_rate_2_: {
    title: "Atk. Power % (Synergies)",
    help: "Damage directly influenced by increases in Atk. Power. Primarily comes from party synergy effects.",
    type: "additive"
  },
  skill_damage_sub_rate_1_: {
    title: "Damage To Foes % (Base)",
    help: 'Damage directly influenced by effects that increase damage to foes. The game usually describes these effects as "Damage to Foes +X%". Multiplicative with support "Damage to Foes" increases.',
    type: "additive"
  },
  skill_damage_sub_rate_2_: {
    title: "Damage To Foes % (Supports)",
    help: 'Damage directly influenced by effects that increase damage to foes. The game usually describes these effects as "Damage to Foes +X%". This group only contains support buffs. Multiplicative with base "Damage to Foes" increases.',
    type: "additive"
  },
  skill_damage_rate_: simpleStat("Additional Damage %"),
  ultimate_awakening_damage_rate_: simpleStat("Hyper Awakening Skill Damage %"),
  move_speed_to_damage_rate_: {
    title: "Raid Captain",
    help: "Damage directly influenced by the Raid Captain engraving.",
    type: "singular"
  },
  physical_defense_break_: {
    title: "Physical Defense Reduction %",
    help: "Damage directly influenced by physical defense reduction debuffs on the boss.",
    type: "additive"
  },
  magical_defense_break_: {
    title: "Magical Defense Reduction %",
    help: "Damage directly influenced by magical defense reduction debuffs on the boss.",
    type: "additive"
  },
  outgoing_dmg_stat_amp_: {
    title: "Outgoing Damage Amplification %",
    help: 'Damage directly influenced by any effects that increase incoming damage on the boss. The game usually describes these effects as "Incoming Damage +X%" or "Damage from foes +X%", though it is not very consistent with this.',
    type: "additive"
  },
  skill_damage_amplify_: {
    title: "Damage Dealt Increase %",
    help: 'Damage directly influenced by any effects that increase damage received. This group includes support brand effects. The game usually describes these effects as "Incoming Damage +X%" or "Damage from foes +X%", though it is not very consistent with this.',
    type: "additive"
  },
  front_attack_amplify_: {
    title: "Front Attack Bonus Damage %",
    help: "Damage directly influenced by any effects that increase front attack damage.",
    type: "additive"
  },
  back_attack_amplify_: {
    title: "Back Attack Bonus Damage %",
    help: "Damage directly influenced by any effects that increase back attack damage.",
    type: "additive"
  },
  physical_critical_damage_amplify_: {
    title: "Physical Crit Damage Increase %",
    help: "Damage directly influenced by any physical crit damage increase effects on the boss.",
    type: "additive"
  },
  magical_critical_damage_amplify_: {
    title: "Magical Crit Damage Increase %",
    help: "Damage directly influenced by any magical crit damage increase effects on the boss.",
    type: "additive"
  },
  critical_hit_to_damage_rate_: {
    title: (spec) => {
      if (spec === "Asura's Path") return "Asura's Path Basic Attack Crit Conversion";
      if (spec === "Lone Knight") return "Gunlance Skill Crit Conversion";
      if (spec === "Shining Knight") return "Shining Knight Crit Conversion";
      if (spec === "Rage Hammer" || spec === "Gravity Training") return "Destroyer State Crit Conversion";
      return "UNUSED - Please Report If You See This!";
    },
    help: (spec) => {
      if (spec === "Asura's Path")
        return "Damage directly influenced by the crit-to-damage conversion given by the Lethal Fist Ark Passive node.";
      if (spec === "Lone Knight")
        return "Damage directly influenced by the crit-to-damage conversion given to all Gunlance Skills by the Gunlance Training Ark Passive node.";
      if (spec === "Shining Knight")
        return "Damage directly influenced by the crit-to-damage conversion given by the Holy Sword Unleashed Ark Passive node.";
      if (spec === "Rage Hammer" || spec === "Gravity Training")
        return "Damage directly influenced by the crit-to-damage conversion given by the Gravity Conversion Ark Passive node.";
      return "critical_hit_to_damage_rate_";
    },
    type: "singular"
  },
  evolution_damage_bonus_from_blunt_thorn_: {
    title: "Blunt Thorn Evolution Damage",
    help: "Damage directly influenced by the evolution damage bonus provided by the excess-crit-to-damage conversion of the Blunt Thorn Ark Passive node.",
    type: "singular"
  },
  evolution_damage_bonus_from_supersonic_breakthrough_: {
    title: "Supersonic Breakthrough Evolution Damage",
    help: "Damage directly influenced by the evolution damage bonus provided by the excess-speed-to-damage conversion of the Supersonic Breakthrough Ark Passive node.",
    type: "singular"
  },
  "damage_attr_amplifications_[NONE]": unused("damage_attr_amplifications_[NONE]"),
  "damage_attr_amplifications_[FIRE]": attrAmplification("Fire"),
  "damage_attr_amplifications_[ICE]": attrAmplification("Water"),
  "damage_attr_amplifications_[ELECTRICITY]": attrAmplification("Lightning"),
  "damage_attr_amplifications_[WIND]": unused("damage_attr_amplifications_[WIND]"),
  "damage_attr_amplifications_[EARTH]": attrAmplification("Earth"),
  "damage_attr_amplifications_[DARK]": attrAmplification("Dark"),
  "damage_attr_amplifications_[HOLY]": attrAmplification("Holy"),
  "damage_attr_rates_[NONE]": unused("damage_attr_rates_[NONE]"),
  "damage_attr_rates_[FIRE]": attrRates("Fire"),
  "damage_attr_rates_[ICE]": attrRates("Water"),
  "damage_attr_rates_[ELECTRICITY]": attrRates("Lightning"),
  "damage_attr_rates_[WIND]": unused("damage_attr_rates_[WIND]"),
  "damage_attr_rates_[EARTH]": attrRates("Earth"),
  "damage_attr_rates_[DARK]": attrRates("Dark"),
  "damage_attr_rates_[HOLY]": attrRates("Holy")
};
