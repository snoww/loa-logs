import { ArkGridCore, ArkGridCoreGroup } from "$lib/constants/ArkGrid";
import type { ArkPassiveNode } from "$lib/types";

type ArkGridOrderNames = [string[], string[], string[]];

const ARK_GRID_ORDER_NAMES = new Map<number, ArkGridOrderNames>();

for (const [groupIdText, group] of Object.entries(ArkGridCoreGroup)) {
  const groupId = Number(groupIdText);
  const core = ArkGridCore[groupId];
  if (!core || core[1] !== 0) continue;

  const coreType = core[2];
  if (coreType < 0 || coreType > 2) continue;

  const requiredArkPassiveNodeId = group[3];
  const names = ARK_GRID_ORDER_NAMES.get(requiredArkPassiveNodeId) ?? [[], [], []];
  names[coreType].push(core[0].replace(/^Order (Sun|Moon|Star) Core: /, ""));
  ARK_GRID_ORDER_NAMES.set(requiredArkPassiveNodeId, names);
}

export function getArkGridOrderNames(nodes?: ArkPassiveNode[]): ArkGridOrderNames | undefined {
  for (const node of nodes ?? []) {
    const names = ARK_GRID_ORDER_NAMES.get(node.id);
    if (names) return names;
  }
}
