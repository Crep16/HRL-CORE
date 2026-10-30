import { ref } from 'vue';
export type Repair = {
  stages: string[];
  completed: number; technician: string; publicNote: string; internalNote: string; syncMode: 'auto' | 'manual';
  events: { id:number; date:string; title:string; detail:string; stage:string; public:boolean }[];
  warranties: { name:string; days:number }[]; parts: { name:string; price:number }[];
  payments: { date:string; amount:number; method?:string }[]; visibility: { label:string; enabled:boolean }[];
};
export const defaultStageOptions = ['Recibido', 'Diagnóstico', 'Autorización', 'Refacciones', 'Reparación', 'Pruebas', 'Entregado'];
const legacyStages = [...defaultStageOptions.slice(0, -1), 'Entrega'];
export function includeRepairStage(repair: Repair, name: string): string {
  const normalized = name.trim();
  const existing = repair.stages.find(item => item.toLocaleLowerCase('es') === normalized.toLocaleLowerCase('es'));
  if (existing) return existing;
  if (repair.completed === repair.stages.length) throw new Error('La entrega ya está completada. No se pueden incorporar etapas intermedias.');
  if (!normalized || new TextEncoder().encode(normalized).length > 100 || repair.stages.length >= 200) throw new Error('La etapa debe tener entre 1 y 100 bytes y la orden admite hasta 200 etapas.');
  repair.stages.splice(repair.stages.length - 1, 0, normalized);
  return normalized;
}
function emptyRepair(): Repair {
  return { stages:['Recibido', 'Entregado'], completed:0, technician:'', publicNote:'', internalNote:'', syncMode:'manual', events:[], warranties:[], parts:[], payments:[], visibility:[{label:'Línea de tiempo',enabled:true},{label:'Garantías',enabled:true},{label:'Resumen de pago',enabled:false},{label:'Notas del técnico',enabled:true}] };
}

export function restoreRepair(raw: string): Repair {
  const value: unknown = JSON.parse(raw);
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Invalid repair');
  const repair = { ...emptyRepair(), stages:[...legacyStages], ...value };
  const record = (item: unknown): item is Record<string, unknown> => !!item && typeof item === 'object' && !Array.isArray(item);
  const text = (item: unknown) => typeof item === 'string';
  const nonnegative = (item: unknown): item is number => typeof item === 'number' && Number.isFinite(item) && item >= 0;
  const list = (items: unknown, valid: (item: Record<string, unknown>) => boolean) => Array.isArray(items) && items.every(item => record(item) && valid(item));
  if (!Array.isArray(repair.stages) || repair.stages.length < 2 || repair.stages.length > 200
    || repair.stages.some(item => typeof item !== 'string' || !item.trim() || new TextEncoder().encode(item).length > 100)
    || new Set(repair.stages.map(item => item.trim().toLocaleLowerCase('es'))).size !== repair.stages.length
    || !Number.isInteger(repair.completed) || repair.completed < 0 || repair.completed > repair.stages.length
    || ![repair.technician, repair.publicNote, repair.internalNote].every(text)
    || !['auto', 'manual'].includes(repair.syncMode)
    || !list(repair.events, item => nonnegative(item.id) && Number.isInteger(item.id) && [item.date, item.title, item.detail, item.stage].every(text) && typeof item.public === 'boolean')
    || !list(repair.warranties, item => text(item.name) && nonnegative(item.days) && Number.isInteger(item.days))
    || !list(repair.parts, item => text(item.name) && nonnegative(item.price))
    || !list(repair.payments, item => text(item.date) && nonnegative(item.amount) && item.amount > 0)
    || !list(repair.visibility, item => text(item.label) && typeof item.enabled === 'boolean')) throw new Error('Invalid repair fields');
  return repair;
}

export function useRepair(id: string) {
  const message = ref('');
  const recoveryMessage = 'No se pudo recuperar el seguimiento local. No captures cambios hasta recuperar el almacenamiento y volver a abrir la aplicación; los datos originales no se han modificado.';
  let recoveryFailed = false;
  let initial = emptyRepair();
  try {
    const stored = localStorage.getItem(`hrl.repair.v1.${id}`);
    if (stored !== null) initial = restoreRepair(stored);
  } catch { recoveryFailed = true; message.value = recoveryMessage; }
  const repair = ref<Repair>(initial);
  function save() {
    if (recoveryFailed) { message.value = recoveryMessage; return false; }
    try {
      const serialized = JSON.stringify(repair.value);
      restoreRepair(serialized);
      localStorage.setItem(`hrl.repair.v1.${id}`, serialized);
      message.value = 'Cambios guardados en este equipo.'; return true;
    }
    catch { message.value = 'No se pudieron guardar los cambios. Mantén esta pantalla abierta e inténtalo de nuevo.'; return false; }
  }
  return { repair, save, message };
}
