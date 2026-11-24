import supportBuffWeights from "./supportBuffWeights.json" with { type: "json" };

export const SUPPORT_SPECS = ["Desperate Salvation", "Full Bloom", "Blessed Aura", "Liberator"];

export const isSupportSpec = (spec?: string): boolean => !!spec && SUPPORT_SPECS.includes(spec);

export const SUPPORT_CLASS_IDS = [105, 113, 204, 602];

export const isSupportClassId = (classId?: number): boolean => !!classId && SUPPORT_CLASS_IDS.includes(classId);

export type PlayerRole = "support" | "dps";

export const resolveLocalPlayerRole = (entity: { spec?: string | null; classId?: number }): PlayerRole | null => {
  if (entity.spec && entity.spec !== "Unknown") {
    return isSupportSpec(entity.spec) ? "support" : "dps";
  }
  if (!entity.classId || isSupportClassId(entity.classId)) {
    return null;
  }
  return "dps";
};

export const SUPPORT_BUFF_WEIGHTS: {
  readonly ap: number;
  readonly brand: number;
  readonly identity: number;
} = supportBuffWeights;

export function supportBuffPerformance(ap: number, brand: number, identity: number): number {
  return SUPPORT_BUFF_WEIGHTS.ap * ap + SUPPORT_BUFF_WEIGHTS.brand * brand + SUPPORT_BUFF_WEIGHTS.identity * identity;
}
