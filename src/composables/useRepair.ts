import { ref } from 'vue';
type Repair = {
  completed: number; technician: string; publicNote: string; internalNote: string; syncMode: 'auto' | 'manual';
  events: { id:number; date:string; title:string; detail:string; stage:string; public:boolean }[];
  warranties: { name:string; days:number }[]; parts: { name:string; price:number }[];
  payments: { date:string; amount:number }[]; visibility: { label:string; enabled:boolean }[];
};
export function useRepair(id: string) {
  const message = ref('');
  const empty: Repair = { completed:0, technician:'', publicNote:'', internalNote:'', syncMode:'manual', events:[], warranties:[], parts:[], payments:[], visibility:[{label:'Línea de tiempo',enabled:true},{label:'Garantías',enabled:true},{label:'Resumen de pago',enabled:false},{label:'Notas del técnico',enabled:true}] };
  let initial = empty;
  try { const stored = localStorage.getItem(`hrl.repair.v1.${id}`); if (stored) initial = { ...empty, ...JSON.parse(stored) }; } catch { message.value = 'No se pudo recuperar el seguimiento local.'; }
  const repair = ref<Repair>(initial);
  function save() {
    try { localStorage.setItem(`hrl.repair.v1.${id}`, JSON.stringify(repair.value)); message.value = 'Cambios guardados en este equipo.'; return true; }
    catch { message.value = 'No se pudieron guardar los cambios. Mantén esta pantalla abierta e inténtalo de nuevo.'; return false; }
  }
  return { repair, save, message };
}
