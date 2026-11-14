import type { EquipmentHistory } from '../types/ipc.ts';

export function equipmentHistoryText(history: EquipmentHistory): string {
  if (history.notes != null) return history.notes;
  const fields = [
    [history.previousWorkshop, 'Otro taller', history.previousWorkshopDetail],
    [history.sufferedImpact, 'Caídas o golpes', history.impactDetail],
    [history.liquidSpill, 'Líquidos o humedad', history.liquidDetail],
  ] as const;
  return fields.filter(([flag, , detail]) => flag || detail.trim()).map(([, label, detail]) => detail.trim() ? `${label}: ${detail}` : label).join('\n');
}
