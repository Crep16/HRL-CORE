<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useAccess } from './composables/useAccess';
import UsersWorkspace from './components/access/UsersWorkspace.vue';
const { can } = useAccess();
const tabs = computed(() => ['Clientes', 'Bitácora', 'Empresa', ...(can('users.manage') ? ['Usuarios'] : [])]);
const emit = defineEmits<{ logout: [] }>();
import AppHeader from './components/layout/AppHeader.vue';
import OrderList from './components/orders/OrderList.vue';
import OrderSheet from './components/orders/OrderSheet.vue';
import RepairWorkspace from './components/repairs/RepairWorkspace.vue';
import CompanyWorkspace from './components/company/CompanyWorkspace.vue';
import CompanyNote from './components/company/CompanyNote.vue';
import { cloneCompany, formatFolio, useCompany } from './composables/useCompany';
import { printCompanyNote } from './utils/printCompanyNote';
import type { NoteKind } from './types/company';
import { useOrders } from './composables/useOrders';
import type { Order } from './types/order';
const { storageError, busy, ready, needsImport, load, importLegacy, saveOrder, nextNumber, orders, selectedId, query, field, ascending, visible, index, selected, goTo, removeSelected } = useOrders();
const page = ref<'clients' | 'repairs' | 'company' | 'users'>('clients');
const { company, saveCompany } = useCompany();
const companyDirty = ref(false);
const repairWorkspace = ref<InstanceType<typeof RepairWorkspace>>();
const printSource = ref<HTMLElement>();
const printKind = ref<NoteKind>('entry');
const printVersion = ref(0);
const reservedNumber = ref<number>();
function navigate(tab: string) {
  const destination = tab === 'Usuarios' ? 'users' : tab === 'Empresa' ? 'company' : 'clients';
  if (destination === 'users' && !can('users.manage')) return;
  if (destination === 'company' && !can('company.view')) { notice.value = 'Tu acceso no permite consultar Empresa.'; return; }
  if (destination === page.value) return;
  if (draft.value || companyDirty.value) { notice.value='Guarda o descarta los cambios pendientes antes de cambiar de sección.'; return; }
  if (page.value === 'repairs' && repairWorkspace.value && !repairWorkspace.value.save()) return;
  deleting.value=false; notice.value=''; page.value=destination;
}
async function printOrder(kind: NoteKind) {
  if (!selected.value || draft.value) return;
  if (companyDirty.value) { notice.value='Guarda o descarta los cambios de Empresa antes de imprimir una orden.'; return; }
  printKind.value=kind; printVersion.value++; await nextTick();
  const note=printSource.value?.querySelector<HTMLElement>('.company-note');
  if (!note) return;
  try { await printCompanyNote(note,company.value); }
  catch(error) {notice.value=error instanceof Error ? error.message : 'No se pudo preparar la impresión.';}
}
const draft = ref<Order>();
const originalOrder = ref<Order>();
const notice = ref('');
watch(companyDirty, value => {
  if (!value && (notice.value === 'Guarda o descarta los cambios pendientes antes de cambiar de sección.' || notice.value === 'Guarda o descarta los cambios de Empresa antes de imprimir una orden.')) notice.value='';
});
const deleting = ref(false);
function edit() { if (!can('orders.edit') || !ready.value || busy.value) { notice.value = 'No puedes editar esta orden en este momento.'; return; } if (selected.value) { originalOrder.value = { ...selected.value }; draft.value = { ...selected.value }; focusClient(); } }
function focusClient() { nextTick(() => document.getElementById('cliente')?.focus()); }
async function create() {
  if (!can('orders.create')) { notice.value = 'Tu acceso no permite crear órdenes.'; return; }
  if (!ready.value || busy.value) { notice.value = 'Espera a que se recuperen o importen las órdenes antes de crear otra.'; return; }
  if (draft.value) { notice.value = 'Guarda o cancela la captura actual antes de crear otra orden.'; return; }
  let next: number;
  try { next = await nextNumber(company.value.prefix, company.value.nextNumber); }
  catch (cause) { notice.value = String(cause); return; }
  if (draft.value) return;
  originalOrder.value = undefined;
  reservedNumber.value = next;
  draft.value = { id: formatFolio(company.value.prefix,next), client: '', phone: '', received: new Date().toLocaleDateString(company.value.locale, { timeZone: company.value.timezone }), status: 'En diagnóstico', serial: '', item: 'Laptop', brand: '', model: '', otherShop: false, otherShopReason: '', dropped: false, dropReason: '', password: '', work: '', cost: '$0.00', deposit: '$0.00', notes: '', paid: false, delivered: false };
  focusClient();
}
async function save() {
  if (!draft.value) return;
  if (!can(orders.value.some(order => order.id === draft.value!.id) ? 'orders.edit' : 'orders.create')) { notice.value = 'Tu acceso no permite guardar esta orden.'; return; }
  const order = { ...draft.value, client: draft.value.client.trim() };
  if (!order.client || !order.phone.trim() || !order.item.trim()) { notice.value = 'Completa cliente, teléfono y artículo.'; return; }
  if (!await saveOrder(order, originalOrder.value)) return;
  if (!originalOrder.value && reservedNumber.value !== undefined && can('company.edit')) {
    const updated=cloneCompany(company.value); updated.nextNumber=reservedNumber.value+1; saveCompany(updated);
  }
  reservedNumber.value=undefined;
  selectedId.value = order.id;
  draft.value = undefined;
  originalOrder.value = undefined;
  query.value = '';
  notice.value = 'Orden guardada en este equipo.';
}
function select(id: string) {
  if (draft.value) { notice.value = 'Guarda o cancela los cambios antes de cambiar de orden.'; return; }
  deleting.value = false;
  selectedId.value = id;
}
function openRepairs() {
  if (!can('repairs.view')) { notice.value = 'Tu acceso no permite consultar Reparaciones.'; return; }
  if (!can('users.manage')) { notice.value = 'El seguimiento está disponible por ahora para el acceso Completo.'; return; }
  page.value = 'repairs';
}
function requestLogout() {
  if (draft.value || companyDirty.value) { notice.value = 'Guarda o descarta los cambios pendientes antes de cerrar sesión.'; return; }
  if (page.value === 'repairs' && repairWorkspace.value && !repairWorkspace.value.save()) return;
  emit('logout');
}
</script>

<template>
  <div class="app">
    <AppHeader :tabs="tabs" :active="page === 'users' ? 'Usuarios' : page === 'company' ? 'Empresa' : 'Clientes'" :can-print="!!selected && !draft && !companyDirty" @change="navigate" @print="printOrder" @logout="requestLogout" />
    <div v-if="storageError" class="notice" role="alert">{{ storageError }}</div>
    <div v-if="busy" class="notice" role="status">Consultando o guardando órdenes…</div>
    <div v-if="needsImport" class="notice" role="status">Hay órdenes guardadas antes de configurar esta base. Un administrador debe importarlas para continuar; se conservará el original.<button v-if="can('users.manage')" class="btn btn-s" :disabled="busy" @click="importLegacy">Importar órdenes guardadas</button></div>
    <button v-if="storageError && !draft" class="btn btn-s" :disabled="busy" @click="load">Volver a consultar órdenes</button>
    <div v-if="notice" class="notice" role="status">{{ notice }}<button class="lnk" type="button" aria-label="Cerrar aviso" @click="notice = ''">×</button></div>
    <div v-if="page === 'clients'" class="work reception">
      <OrderList v-model:query="query" v-model:field="field" :orders="visible" :total="orders.length" :selected-id="selectedId" @select="select" @toggle-sort="ascending = !ascending" @create="create" />
      <OrderSheet v-if="draft || selected" :key="draft ? 'draft' : selectedId" :order="(draft || selected)!" :editing="!!draft" :busy="busy" :initial-payment="!originalOrder" :position="index + 1" :total="orders.length" @go-to="draft ? notice = 'Guarda o cancela los cambios antes de navegar.' : goTo($event)" @edit="edit" @save="save" @cancel="busy ? undefined : draft = undefined" @remove="removeSelected" @repairs="openRepairs" />
      <section v-else class="sheet empty-work"><h1>No hay órdenes</h1><p>Crea una nueva orden para comenzar.</p><button class="btn btn-p" @click="create">Nueva orden</button></section>
      <div v-if="deleting && selected" class="delete-inline" role="alert">¿Eliminar {{ selected.id }}?<button class="btn btn-s" @click="deleting = false">Cancelar</button><button class="btn btn-p" @click="removeSelected(); deleting = false; notice = 'Orden eliminada.'">Eliminar orden</button></div>
    </div>
    <UsersWorkspace v-else-if="page === 'users' && can('users.manage')" />
    <CompanyWorkspace v-else-if="page === 'company' && can('company.view')" :orders="orders" :order="selected" @dirty-change="companyDirty=$event" />
    <RepairWorkspace v-else-if="selected" ref="repairWorkspace" :key="selected.id" :order="selected" @back="page = 'clients'" />
    <div ref="printSource" class="sr" aria-hidden="true"><CompanyNote :key="printVersion" :settings="company" :kind="printKind" :order="selected" /></div>
  </div>
</template>
<style src="./assets/styles/orders.css"></style>
<style src="./assets/styles/workspace.css"></style>
<style src="./assets/styles/readability.css"></style>
