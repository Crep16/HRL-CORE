import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { restoreOrders } from '../utils/restoreOrders';
import { useAccess } from './useAccess';
import { useCompany } from './useCompany';
import type { Order, SearchField } from '../types/order';

export function useOrders() {
  const { company } = useCompany();
  const { can } = useAccess();
  const storageError = ref('');
  const busy = ref(false);
  const ready = ref(false);
  const needsImport = ref(false);
  const orders = ref<Order[]>([]);
  const selectedId = ref('');
  const query = ref('');
  const field = ref<SearchField>('all');
  const ascending = ref(true);
  let legacyRaw: string | null = null;

  async function load() {
    if (busy.value) return;
    busy.value = true; ready.value = false; storageError.value = '';
    try {
      const result = await invoke<{ orders: Order[]; legacyImported: boolean }>('list_orders');
      orders.value = result.orders;
      if (!orders.value.some(order => order.id === selectedId.value)) selectedId.value = orders.value[0]?.id ?? '';
      legacyRaw = localStorage.getItem('hrl.orders.v1');
      needsImport.value = legacyRaw !== null && !result.legacyImported;
      ready.value = !needsImport.value;
    } catch (cause) { storageError.value = typeof cause === 'string' ? cause : 'No se pudieron recuperar las órdenes. Los datos originales no se han modificado.'; }
    finally { busy.value = false; }
  }
  onMounted(load);

  async function importLegacy() {
    if (busy.value || legacyRaw === null || !can('users.manage')) return;
    busy.value = true; storageError.value = '';
    try {
      const legacy = restoreOrders(legacyRaw).map(order => ({ ...order, currency: order.currency ?? company.value.currency }));
      await invoke('import_orders', { orders: legacy });
      needsImport.value = false;
    } catch (cause) { storageError.value = typeof cause === 'string' ? cause : 'No se pudieron importar las órdenes. Conserva el almacenamiento original para su revisión.'; }
    finally { busy.value = false; }
    if (!needsImport.value) await load();
  }

  async function saveOrder(order: Order, original?: Order) {
    if (busy.value || !ready.value) return false;
    busy.value = true; storageError.value = '';
    try {
      order = { ...order, currency: order.currency ?? company.value.currency };
      await invoke('save_order', { order, original: original ?? null });
      const position = orders.value.findIndex(item => item.id === order.id);
      if (position >= 0) orders.value[position] = { ...order }; else orders.value.push({ ...order });
      selectedId.value = order.id;
      return true;
    } catch (cause) { storageError.value = typeof cause === 'string' ? cause : 'No se pudo guardar la orden. Tu borrador sigue en pantalla.'; return false; }
    finally { busy.value = false; }
  }

  async function nextNumber(prefix: string, start: number) {
    return invoke<number>('next_order_number', { prefix, start });
  }
  const sorted = computed(() => [...orders.value].sort((a, b) => ascending.value ? a.id.localeCompare(b.id) : b.id.localeCompare(a.id)));
  const visible = computed(() => {
    const text = query.value.trim().toLocaleLowerCase('es');
    if (!text) return sorted.value;
    return sorted.value.filter(order => ({ all: [order.id, order.client, order.serial], order: [order.id], client: [order.client], serial: [order.serial] })[field.value].some(value => value.toLocaleLowerCase('es').includes(text)));
  });
  const index = computed(() => sorted.value.findIndex(order => order.id === selectedId.value));
  const selected = computed<Order | undefined>(() => sorted.value[index.value]);
  function goTo(position: number) { if (sorted.value.length) selectedId.value = sorted.value[Math.min(Math.max(position, 0), sorted.value.length - 1)].id; }
  function removeSelected() { storageError.value = 'El borrado está deshabilitado para conservar el historial de las órdenes.'; }
  return { storageError, busy, ready, needsImport, load, importLegacy, saveOrder, nextNumber, orders, selectedId, query, field, ascending, visible, index, selected, goTo, removeSelected };
}
