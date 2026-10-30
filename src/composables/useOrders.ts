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
  const showDeleted = ref(false);
  const orders = ref<Order[]>([]);
  const selectedId = ref('');
  const query = ref('');
  const field = ref<SearchField>('all');
  const ascending = ref(true);
  let legacyRaw: string | null = null;

  let pending: Promise<boolean> | null = null;
  function fetchOrders(retain: boolean): Promise<boolean> {
    if (pending) return pending;
    if (busy.value) return Promise.resolve(false);
    pending = readOrders(retain).finally(() => { pending = null; });
    return pending;
  }
  async function readOrders(retain: boolean) {
    busy.value = true; ready.value = false; storageError.value = '';
    if (!retain) orders.value = [];
    try {
      const result = await invoke<{ orders: Order[]; legacyImported: boolean }>('list_orders', { deleted: showDeleted.value });
      orders.value = result.orders;
      if (!orders.value.some(order => order.id === selectedId.value)) selectedId.value = orders.value[0]?.id ?? '';
      legacyRaw = localStorage.getItem('hrl.orders.v1');
      needsImport.value = legacyRaw !== null && !result.legacyImported;
      ready.value = !needsImport.value;
      return ready.value;
    } catch (cause) { storageError.value = typeof cause === 'string' ? cause : 'No se pudieron recuperar las órdenes. Los datos originales no se han modificado.'; return false; }
    finally { busy.value = false; }
  }
  const load = () => fetchOrders(false);
  const refresh = () => fetchOrders(true);
  onMounted(() => load());

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
      const saved = await invoke<Order>('save_order', { order, original: original ?? null });
      const position = orders.value.findIndex(item => item.id === order.id);
      if (position >= 0) orders.value[position] = saved; else orders.value.push(saved);
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
  async function setDeleted(order: Order, deleted: boolean) {
    if (busy.value || !ready.value || !can('orders.delete')) return false;
    busy.value = true; storageError.value = '';
    try {
      await invoke('set_order_deleted', { order, deleted });
      const position = index.value;
      orders.value = orders.value.filter(item => item.id !== order.id);
      selectedId.value = sorted.value[Math.min(Math.max(position, 0), sorted.value.length - 1)]?.id ?? '';
      return true;
    } catch (cause) { storageError.value = typeof cause === 'string' ? cause : 'No se pudo cambiar el estado de la orden. Vuelve a intentar.'; return false; }
    finally { busy.value = false; }
  }
  return { storageError, busy, ready, needsImport, showDeleted, load, refresh, importLegacy, saveOrder, setDeleted, nextNumber, orders, selectedId, query, field, ascending, visible, index, selected, goTo };
}
