<script setup lang="ts">
import { nextTick, ref, watch } from 'vue';
const emit = defineEmits<{ logout: [] }>();
import AppHeader from './components/layout/AppHeader.vue';
import OrderList from './components/orders/OrderList.vue';
import OrderSheet from './components/orders/OrderSheet.vue';
import RepairWorkspace from './components/repairs/RepairWorkspace.vue';
import CompanyWorkspace from './components/company/CompanyWorkspace.vue';
import CompanyNote from './components/company/CompanyNote.vue';
import { cloneCompany, formatFolio, nextAvailableNumber, useCompany } from './composables/useCompany';
import { printCompanyNote } from './utils/printCompanyNote';
import type { NoteKind } from './types/company';
import { useOrders } from './composables/useOrders';
import type { Order } from './types/order';
const { storageError, orders, selectedId, query, field, ascending, visible, index, selected, goTo, removeSelected } = useOrders();
const page = ref<'clients' | 'repairs' | 'company'>('clients');
const { company, saveCompany } = useCompany();
const companyDirty = ref(false);
const repairWorkspace = ref<InstanceType<typeof RepairWorkspace>>();
const printSource = ref<HTMLElement>();
const printKind = ref<NoteKind>('entry');
const printVersion = ref(0);
const reservedNumber = ref<number>();
function navigate(tab: string) {
  const destination = tab === 'Empresa' ? 'company' : 'clients';
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
const notice = ref('');
watch(companyDirty, value => {
  if (!value && (notice.value === 'Guarda o descarta los cambios pendientes antes de cambiar de sección.' || notice.value === 'Guarda o descarta los cambios de Empresa antes de imprimir una orden.')) notice.value='';
});
const deleting = ref(false);
function edit() { if (selected.value) { draft.value = { ...selected.value }; focusClient(); } }
function focusClient() { nextTick(() => document.getElementById('cliente')?.focus()); }
function create() {
  if (draft.value) { notice.value = 'Guarda o cancela la captura actual antes de crear otra orden.'; return; }
  const next = nextAvailableNumber(company.value, orders.value);
  reservedNumber.value = next;
  draft.value = { id: formatFolio(company.value.prefix,next), client: '', phone: '', received: new Date().toLocaleDateString(company.value.locale, { timeZone: company.value.timezone }), status: 'En diagnóstico', serial: '', item: 'Laptop', brand: '', model: '', otherShop: false, otherShopReason: '', dropped: false, dropReason: '', password: '', work: '', cost: '$0.00', deposit: '$0.00', notes: '', paid: false, delivered: false };
  focusClient();
}
function save() {
  if (!draft.value) return;
  const order = { ...draft.value, client: draft.value.client.trim() };
  if (!order.client || !order.phone.trim() || !order.item.trim()) { notice.value = 'Completa cliente, teléfono y artículo.'; return; }
  const position = orders.value.findIndex(item => item.id === order.id);
  if (position >= 0) orders.value[position] = order; else orders.value.push(order);
  if (position < 0 && reservedNumber.value !== undefined) {
    const updated=cloneCompany(company.value); updated.nextNumber=reservedNumber.value+1; saveCompany(updated);
  }
  reservedNumber.value=undefined;
  selectedId.value = order.id;
  draft.value = undefined;
  query.value = '';
  notice.value = 'Orden guardada en este equipo.';
}
function select(id: string) {
  if (draft.value) { notice.value = 'Guarda o cancela los cambios antes de cambiar de orden.'; return; }
  deleting.value = false;
  selectedId.value = id;
}
function requestLogout() {
  if (draft.value || companyDirty.value) { notice.value = 'Guarda o descarta los cambios pendientes antes de cerrar sesión.'; return; }
  if (page.value === 'repairs' && repairWorkspace.value && !repairWorkspace.value.save()) return;
  emit('logout');
}
</script>

<template>
  <div class="app">
    <AppHeader :tabs="['Clientes', 'Bitácora', 'Empresa']" :active="page === 'company' ? 'Empresa' : 'Clientes'" :can-print="!!selected && !draft && !companyDirty" @change="navigate" @print="printOrder" @logout="requestLogout" />
    <div v-if="storageError" class="notice" role="alert">{{ storageError }}</div>
    <div v-if="notice" class="notice" role="status">{{ notice }}<button class="lnk" type="button" aria-label="Cerrar aviso" @click="notice = ''">×</button></div>
    <div v-if="page === 'clients'" class="work reception">
      <OrderList v-model:query="query" v-model:field="field" :orders="visible" :total="orders.length" :selected-id="selectedId" @select="select" @toggle-sort="ascending = !ascending" @create="create" />
      <OrderSheet v-if="draft || selected" :key="draft ? 'draft' : selectedId" :order="(draft || selected)!" :editing="!!draft" :position="index + 1" :total="orders.length" @go-to="draft ? notice = 'Guarda o cancela los cambios antes de navegar.' : goTo($event)" @edit="edit" @save="save" @cancel="draft = undefined" @remove="deleting = true" @repairs="page = 'repairs'" />
      <section v-else class="sheet empty-work"><h1>No hay órdenes</h1><p>Crea una nueva orden para comenzar.</p><button class="btn btn-p" @click="create">Nueva orden</button></section>
      <div v-if="deleting && selected" class="delete-inline" role="alert">¿Eliminar {{ selected.id }}?<button class="btn btn-s" @click="deleting = false">Cancelar</button><button class="btn btn-p" @click="removeSelected(); deleting = false; notice = 'Orden eliminada.'">Eliminar orden</button></div>
    </div>
    <CompanyWorkspace v-else-if="page === 'company'" :orders="orders" :order="selected" @dirty-change="companyDirty=$event" />
    <RepairWorkspace v-else-if="selected" ref="repairWorkspace" :key="selected.id" :order="selected" @back="page = 'clients'" />
    <div ref="printSource" class="sr" aria-hidden="true"><CompanyNote :key="printVersion" :settings="company" :kind="printKind" :order="selected" /></div>
  </div>
</template>
<style src="./assets/styles/orders.css"></style>
<style src="./assets/styles/workspace.css"></style>
<style src="./assets/styles/readability.css"></style>
