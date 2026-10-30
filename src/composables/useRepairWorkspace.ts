import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { restoreRepair, type Repair } from './useRepair';
import type { PaymentState } from '../types/payments';
type Snapshot = { repair: Repair; revision: number; payment: PaymentState };
const clone = (repair: Repair): Repair => JSON.parse(JSON.stringify(repair));

export function useRepairWorkspace(orderId: string, onSaved?: () => void) {
  const repair = ref<Repair>(restoreRepair('{}'));
  const message = ref('');
  const busy = ref(false);
  const ready = ref(false);
  const payment = ref<PaymentState>();
  let revision = 0;
  let baseline = JSON.stringify(repair.value);
  const dirty = computed(() => JSON.stringify(repair.value) !== baseline);
  async function load() {
    busy.value = true; ready.value = false;
    try {
      let result = await invoke<Snapshot>('load_repair', { orderId, legacy: null });
      if (result.revision === 0) {
        const raw = localStorage.getItem(`hrl.repair.v1.${orderId}`);
        if (raw !== null) {
          const legacy = restoreRepair(raw);
          repair.value = legacy; baseline = JSON.stringify(legacy);
          result = await invoke<Snapshot>('load_repair', { orderId, legacy });
        }
      }
      repair.value = result.repair; revision = result.revision; payment.value = result.payment;
      baseline = JSON.stringify(repair.value); ready.value = true;
    } catch (cause) { message.value = typeof cause === 'string' ? cause : 'No se pudo recuperar el seguimiento. Los datos originales no se han modificado.'; }
    finally { busy.value = false; }
  }
  onMounted(load);
  async function persist(next: Repair) {
    if (!ready.value || busy.value) return false;
    busy.value = true; message.value = '';
    try {
      const result = await invoke<Snapshot>('save_repair', { orderId, repair: next, revision });
      repair.value = result.repair; revision = result.revision; payment.value = result.payment; baseline = JSON.stringify(result.repair);
      onSaved?.();
      message.value = 'Seguimiento guardado en este equipo.'; return true;
    } catch (cause) { message.value = typeof cause === 'string' ? cause : 'No se pudo guardar el seguimiento. Conserva el borrador e inténtalo de nuevo.'; return false; }
    finally { busy.value = false; }
  }
  async function save() { if (busy.value) return false; if (!dirty.value) return true; return persist(clone(repair.value)); }
  async function commit(change: (next: Repair) => void) { const next = clone(repair.value); change(next); return persist(next); }
  return { repair, payment, message, busy, ready, dirty, save, commit, reload:load };
}
