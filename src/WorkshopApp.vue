<script setup lang="ts">
import { computed, nextTick, onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import TemporaryAccessDialog from './components/access/TemporaryAccessDialog.vue';
import SearchSelect from './components/ui/SearchSelect.vue';
import { useAccess } from './composables/useAccess';
import UsersWorkspace from './components/access/UsersWorkspace.vue';
const { can, baseCan, temporary, requestAccess, endTemporary, setContext } = useAccess();
const tabs = ['Clientes','Reparaciones','Bitácora','Empresa','Usuarios'];
const emit = defineEmits<{ logout: [] }>();
import AppHeader from './components/layout/AppHeader.vue';
import AppNotice from './components/ui/AppNotice.vue';
import OrderList from './components/orders/OrderList.vue';
import OrderSheet from './components/orders/OrderSheet.vue';
import OrderDeleteDialog from './components/orders/OrderDeleteDialog.vue';
import OrderDiscardDialog from './components/orders/OrderDiscardDialog.vue';
import RepairWorkspace from './components/repairs/RepairWorkspace.vue';
import LogbookWorkspace from './components/logbook/LogbookWorkspace.vue';
import CompanyWorkspace from './components/company/CompanyWorkspace.vue';
import CompanyNote from './components/company/CompanyNote.vue';
import { cloneCompany, formatFolio, useCompany } from './composables/useCompany';
import { printCompanyNote } from './utils/printCompanyNote';
import type { NoteKind } from './types/company';
import { useOrders } from './composables/useOrders';
import type { Order } from './types/order';
const { storageError, busy, ready, needsImport, showDeleted, load, refresh: refreshOrders, importLegacy, saveOrder, setDeleted, nextNumber, orders, selectedId, query, field, ascending, visible, index, selected, goTo } = useOrders();
const page = ref<'clients' | 'repairs' | 'company' | 'users' | 'logbook'>('clients');
const { company, companyStorageError, persistCompany, loadPersistedCompany } = useCompany();
const companyReady=ref(false);
onMounted(async()=>{try{await endTemporary();await loadPersistedCompany();}catch(cause){notice.value=String(cause);}finally{companyReady.value=true;}});
const companyDirty = ref(false);
const repairWorkspace = ref<InstanceType<typeof RepairWorkspace>>();
const printSource = ref<HTMLElement>();
const printNote = ref<InstanceType<typeof CompanyNote>>();
const printKind = ref<NoteKind>('entry');
const printVersion = ref(0);
const reservedNumber = ref<number>();
const requestedOperation=ref('orders.edit');
const accessOptions=computed(()=>page.value==='repairs'?[
 {value:'repairs.edit',label:'Editar Reparaciones'},{value:'notes.add',label:'Agregar notas o avances'},{value:'stages.advance',label:'Completar etapas'},{value:'warranties.add',label:'Agregar garantías'},{value:'parts.add',label:'Agregar refacciones'},{value:'publication.edit',label:'Configurar publicación'},{value:'payments.add',label:'Registrar abonos'},{value:'payments.refund',label:'Devolver saldo a favor'},{value:'payments.adjust',label:'Corregir abonos'}
 ]:page.value==='company'?[{value:'company.view',label:'Consultar Empresa'},{value:'company.edit',label:'Editar Empresa'}]:page.value==='logbook'?[{value:'logbook.view',label:'Consultar Bitácora'},{value:'repairs.edit',label:'Registrar costos directos'}]:page.value==='users'?[{value:'users.manage',label:'Administrar usuarios'}]:[{value:'clients.view',label:'Consultar Clientes'},{value:'orders.create',label:'Crear órdenes'},{value:'orders.edit',label:'Editar órdenes'},{value:'orders.delete',label:'Eliminar o restaurar órdenes'},{value:'payments.add',label:'Registrar abono inicial'}]);
const blockedAccessOptions=computed(()=>accessOptions.value.filter(option=>!can(option.value)));
watch(()=>[page.value,selected.value?.id],()=>{setContext(page.value,page.value==='repairs'?selected.value?.id??null:null);},{immediate:true,flush:'sync'});
watch(blockedAccessOptions,options=>{if(!options.some(option=>option.value===requestedOperation.value))requestedOperation.value=options[0]?.value??'';},{immediate:true});
watch(()=>selected.value?.id,id=>{if(page.value==='repairs'&&temporary.value?.recordId&&temporary.value.recordId!==id)void endTemporary().catch(cause=>notice.value=String(cause));});
onBeforeUnmount(()=>{void endTemporary().catch(()=>{});setContext('clients');});
async function loadRepairOrder(id:string){
 const order=await invoke<Order>('load_authorized_repair_order',{orderId:id});
 const position=orders.value.findIndex(item=>item.id===id);if(position<0)orders.value.push(order);else orders.value[position]=order;
 selectedId.value=id;ready.value=true;
}
async function refreshAfterRepair(){if(baseCan('clients.view'))await refreshOrders();else if(selected.value)await loadRepairOrder(selected.value.id);}
async function requestWindowAccess(){
 if(draft.value||companyDirty.value){notice.value='Guarda o descarta los cambios antes de solicitar otra autorización.';return;}
 if(page.value==='repairs'&&repairWorkspace.value&&!await repairWorkspace.value.save())return;
 if(await requestAccess(requestedOperation.value,page.value,page.value==='repairs'?selected.value?.id??null:null,true)){
  if(page.value==='clients')await refreshOrders();else if(page.value==='repairs')await loadRepairOrder(temporary.value?.recordId??selectedId.value);else if(page.value==='company')await loadPersistedCompany();
 }
}
async function finishTemporary(){
 if(draft.value||companyDirty.value){notice.value='Guarda o descarta los cambios antes de finalizar el acceso temporal.';return;}
 if(page.value==='repairs'&&repairWorkspace.value&&!await repairWorkspace.value.save())return;
 try{await endTemporary();if(!baseCan('clients.view')){orders.value=[];selectedId.value='';}page.value='clients';setContext('clients');if(baseCan('clients.view'))await refreshOrders();}
 catch(cause){notice.value=String(cause);}
}
async function navigate(tab: string) {
  const destination = tab === 'Reparaciones' ? 'repairs' : tab === 'Bitácora' ? 'logbook' : tab === 'Usuarios' ? 'users' : tab === 'Empresa' ? 'company' : 'clients';
  if (draft.value || companyDirty.value) { notice.value='Guarda o descarta los cambios pendientes antes de cambiar de sección.'; return; }
  if (page.value === 'repairs' && repairWorkspace.value && !await repairWorkspace.value.save()) return;
  if(destination==='repairs'&&showDeleted.value){notice.value='Restaura la orden para abrir Reparaciones.';return;}
  const operation=destination==='logbook'?'logbook.view':destination==='repairs'?'repairs.view':destination==='users'?'users.manage':destination==='company'?'company.view':'clients.view';
  const record=destination==='repairs'?selected.value?.id??null:null;
  if(!await requestAccess(operation,destination,record,destination==='repairs'&&!record))return;
  const targetId=destination==='repairs'?(record??temporary.value?.recordId??null):null;
  if(temporary.value&&(temporary.value.scope!==destination||(temporary.value.recordId!==null&&temporary.value.recordId!==targetId))){try{await endTemporary();}catch(cause){notice.value=String(cause);return;}}
  setContext(destination,targetId);
  if(destination==='repairs'&&targetId){try{await loadRepairOrder(targetId);}catch(cause){notice.value=String(cause);return;}}
  else if(destination==='clients'){if(!await refreshOrders())return;}
  if(!baseCan('clients.view')&&destination!=='repairs'&&destination!=='clients'){orders.value=[];selectedId.value='';}
  deleteTarget.value=undefined; notice.value=''; page.value=destination;
}
async function printOrder(kind: NoteKind) {
  if (!selected.value || draft.value || showDeleted.value || busy.value) return;
  if (page.value === 'repairs' && repairWorkspace.value && !await repairWorkspace.value.save()) return;
  if (!await refreshOrders()) return;
  if (!selected.value) return;
  const orderId = selected.value.id;
  if (companyDirty.value) { notice.value='Guarda o descarta los cambios de Empresa antes de imprimir una orden.'; return; }
  printKind.value=kind; printVersion.value++; await nextTick();
  try { await printNote.value?.prepare(); await nextTick(); }
  catch (cause) { notice.value = String(cause); return; }
  if (selected.value?.id !== orderId) { notice.value = 'La selección cambió. Vuelve a solicitar la nota de la orden actual.'; return; }
  const note=printSource.value?.querySelector<HTMLElement>('.company-note');
  if (!note) return;
  try { await printCompanyNote(note,company.value); }
  catch(error) {notice.value=error instanceof Error ? error.message : 'No se pudo preparar la impresión.';}
}
const draft = ref<Order>();
const originalOrder = ref<Order>();
const newOrderInitial=ref<Order>();
const discardRequested=ref(false);
function discardDraft(){draft.value=undefined;originalOrder.value=undefined;newOrderInitial.value=undefined;reservedNumber.value=undefined;discardRequested.value=false;}
function cancelDraft(){
 if(busy.value||!draft.value)return;
 if(JSON.stringify(draft.value)!==JSON.stringify(originalOrder.value??newOrderInitial.value)){discardRequested.value=true;return;}
 discardDraft();
}
const notice = ref('');
watch(companyDirty, value => {
  if (!value && (notice.value === 'Guarda o descarta los cambios pendientes antes de cambiar de sección.' || notice.value === 'Guarda o descarta los cambios de Empresa antes de imprimir una orden.')) notice.value='';
});
const deleteTarget = ref<Order>();
async function edit() { if (showDeleted.value || !ready.value || busy.value) { notice.value = 'No puedes editar esta orden en este momento.'; return; } if(!await requestAccess('orders.edit','clients'))return;if (selected.value) { originalOrder.value = { ...selected.value }; draft.value = { ...selected.value }; focusClient(); } }
function focusClient() { nextTick(() => document.getElementById('cliente')?.focus()); }
async function create() {
  if (showDeleted.value) { notice.value = 'Vuelve a las órdenes activas para crear una nueva.'; return; }
  if (!await requestAccess('orders.create','clients')) return;
  if(!ready.value&&!busy.value)await refreshOrders();
  if (!ready.value || busy.value) { notice.value = 'Espera a que se recuperen o importen las órdenes antes de crear otra.'; return; }
  if (draft.value) { notice.value = 'Guarda o cancela la captura actual antes de crear otra orden.'; return; }
  let next: number;
  try { next = await nextNumber(company.value.prefix, company.value.nextNumber); }
  catch (cause) { notice.value = String(cause); return; }
  if (draft.value) return;
  originalOrder.value = undefined;
  reservedNumber.value = next;
  draft.value = { id: formatFolio(company.value.prefix,next), client: '', phone: '', received: new Date().toLocaleDateString(company.value.locale, { timeZone: company.value.timezone }), status: 'En diagnóstico', serial: '', item: 'Laptop', brand: '', model: '', otherShop: false, otherShopReason: '', dropped: false, dropReason: '', password: '', failure: '', work: '', cost: '$0.00', deposit: '$0.00', notes: '', paid: false, delivered: false };
  newOrderInitial.value={...draft.value};
  focusClient();
}
async function save() {
  if (!draft.value) return;
  if (!can(orders.value.some(order => order.id === draft.value!.id) ? 'orders.edit' : 'orders.create')) { notice.value = 'Tu acceso no permite guardar esta orden.'; return; }
  const order = { ...draft.value, client: draft.value.client.trim() };
  if (!order.client || !order.phone.trim() || !order.item.trim()) { notice.value = 'Completa cliente, teléfono y artículo.'; return; }
  if(!originalOrder.value&&!(order.failure??'').trim()){notice.value='Describe la falla reportada antes de guardar la nueva orden.';return;}
  notice.value = '';
  if (!await saveOrder(order, originalOrder.value)) return;
  if (!originalOrder.value && reservedNumber.value !== undefined && can('company.edit')) {
    const updated=cloneCompany(company.value); updated.nextNumber=reservedNumber.value+1; await persistCompany(updated);
  }
  reservedNumber.value=undefined;
  selectedId.value = order.id;
  draft.value = undefined;
  originalOrder.value = undefined;
  newOrderInitial.value = undefined;
  query.value = '';
  notice.value = 'Orden guardada en este equipo.';
}
function select(id: string) {
  if (draft.value) { notice.value = 'Guarda o cancela los cambios antes de cambiar de orden.'; return; }
  deleteTarget.value = undefined;
  selectedId.value = id;
}
async function openRepairs() {
  if (showDeleted.value) { notice.value = 'Restaura la orden para continuar su reparación.'; return; }
  await navigate('Reparaciones');
}
async function requestLogout() {
  if (draft.value || companyDirty.value) { notice.value = 'Guarda o descarta los cambios pendientes antes de cerrar sesión.'; return; }
  if (page.value === 'repairs' && repairWorkspace.value && !await repairWorkspace.value.save()) return;
  await endTemporary();
  emit('logout');
}
async function requestDelete() {
  if(!await requestAccess('orders.delete','clients'))return;
  if (selected.value && can('orders.delete') && ready.value && !busy.value && !draft.value && !showDeleted.value) {
    storageError.value = ''; deleteTarget.value = { ...selected.value };
  }
}
async function confirmDelete() {
  const order = deleteTarget.value;
  if (!order || !await setDeleted(order, true)) return;
  deleteTarget.value = undefined;
  notice.value = `${order.id} se envió a la papelera. Su historial y sus cobros se conservan.`;
  await focusOrderList();
}
async function restoreSelected() {
  if (!showDeleted.value || !selected.value) return;
  if(!await requestAccess('orders.delete','clients'))return;
  const order = { ...selected.value };
  if (await setDeleted(order, false)) { notice.value = `${order.id} se restauró en las órdenes activas.`; await focusOrderList(); }
}
async function focusOrderList() {
  await nextTick();
  (document.querySelector<HTMLElement>('.sheet .num') ?? document.querySelector<HTMLElement>('.rail .h-disp'))?.focus();
}
async function toggleTrash() {
  if (draft.value) { notice.value = 'Guarda o cancela el borrador antes de cambiar de listado.'; return; }
  if (busy.value) return;
  if (!showDeleted.value && (!can('orders.delete') || !can('clients.view'))) return;
  showDeleted.value = !showDeleted.value; query.value = ''; notice.value = ''; await load();
}
</script>

<template>
  <div class="app">
    <AppHeader :tabs="tabs" :active="page === 'repairs' ? 'Reparaciones' : page === 'logbook' ? 'Bitácora' : page === 'users' ? 'Usuarios' : page === 'company' ? 'Empresa' : 'Clientes'" :can-repair="!draft && !showDeleted && !busy" :can-print="!!selected && !draft && !companyDirty && !showDeleted" @change="navigate" @print="printOrder" @logout="requestLogout" />
    <aside v-if="temporary" class="temporary-banner" role="status"><div><strong>Autorización temporal de {{ temporary.approverName }}</strong><p>{{ temporary.recordId?`Solo para ${temporary.recordId}`:'Solo para este apartado' }}. Tu usuario sigue siendo quien realiza las operaciones.</p></div><button class="btn btn-s" @click="finishTemporary">Finalizar acceso temporal</button></aside>
    <div v-if="companyReady && blockedAccessOptions.length" class="temporary-tools"><SearchSelect id="temporary-operation" v-model="requestedOperation" label="Acción para autorizar temporalmente" :options="blockedAccessOptions" /><button class="btn btn-s" :disabled="busy" @click="requestWindowAccess">Solicitar acceso temporal</button></div>
    <AppNotice :message="storageError" tone="error"><button v-if="!draft" class="btn btn-s" :disabled="busy" @click="load">Volver a consultar órdenes</button></AppNotice>
    <AppNotice v-if="busy" message="Consultando o guardando órdenes…" :closable="false" />
    <AppNotice v-if="needsImport" message="Hay órdenes guardadas antes de configurar esta base. Un administrador debe importarlas para continuar; se conservará el original."><button v-if="can('users.manage')" class="btn btn-s" :disabled="busy" @click="importLegacy">Importar órdenes guardadas</button></AppNotice>
    <AppNotice :message="notice" @dismiss="notice=''" />
    <AppNotice v-if="!companyReady" message="Consultando la configuración del taller…" :closable="false" />
    <AppNotice :message="companyStorageError" tone="error" />
    <div v-if="companyReady && page === 'clients'" class="work reception">
      <OrderList v-model:query="query" v-model:field="field" :orders="can('clients.view')?visible:[]" :total="can('clients.view')?orders.length:0" :selected-id="selectedId" :trashed="showDeleted" :busy="busy" @select="select" @toggle-sort="ascending = !ascending" @create="create" @toggle-trash="toggleTrash" />
      <OrderSheet v-if="draft || (can('clients.view')&&selected)" :key="draft ? 'draft' : selectedId" :order="(draft || selected)!" :editing="!!draft" :busy="busy || !ready" :trashed="showDeleted" :initial-payment="!originalOrder" :position="index + 1" :total="orders.length" @go-to="draft ? notice = 'Guarda o cancela los cambios antes de navegar.' : goTo($event)" @edit="edit" @save="save" @cancel="cancelDraft" @remove="requestDelete" @restore="restoreSelected" @repairs="openRepairs" />
      <section v-else class="sheet empty-work"><h1>{{ busy ? 'Recuperando órdenes' : storageError ? 'Órdenes no disponibles' : showDeleted ? 'Papelera vacía' : 'No hay órdenes' }}</h1><p>{{ storageError ? 'Vuelve a consultar el listado para recuperar sus registros.' : showDeleted ? 'Las órdenes eliminadas aparecerán aquí para que puedas restaurarlas.' : 'Crea una nueva orden para comenzar.' }}</p><button v-if="showDeleted" class="btn btn-s" :disabled="busy" @click="toggleTrash">Volver a órdenes</button><button v-else class="btn btn-p" :disabled="busy || !ready" @click="create">Nueva orden</button></section>
    </div>
    <UsersWorkspace v-else-if="companyReady && page === 'users' && can('users.manage')" />
    <LogbookWorkspace v-else-if="companyReady && page === 'logbook' && can('logbook.view')" />
    <CompanyWorkspace v-else-if="companyReady && page === 'company' && can('company.view')" :orders="orders" :order="selected" @dirty-change="companyDirty=$event" />
    <RepairWorkspace v-else-if="companyReady && page === 'repairs' && selected && can('repairs.view')" ref="repairWorkspace" :key="selected.id" :order="selected" @saved="refreshAfterRepair" @back="temporary?.scope==='repairs'&&!baseCan('clients.view')?finishTemporary():navigate('Clientes')" />
    <section v-else-if="companyReady" class="sheet empty-work" role="alert"><h1>Sección no disponible</h1><p>Tu acceso no permite consultar esta sección.</p><button class="btn btn-s" type="button" @click="navigate('Clientes')">Volver a Clientes</button></section>
    <div ref="printSource" class="sr" aria-hidden="true"><CompanyNote ref="printNote" :key="printVersion" :settings="company" :kind="printKind" :order="can('clients.view')||page==='repairs'&&can('repairs.view')?selected:undefined" /></div>
    <OrderDeleteDialog :order="deleteTarget" :busy="busy" :error="storageError" @confirm="confirmDelete" @cancel="busy ? undefined : deleteTarget = undefined" />
    <TemporaryAccessDialog />
    <OrderDiscardDialog :open="discardRequested" :order-id="draft?.id??''" :busy="busy" @keep="discardRequested=false" @discard="discardDraft" />
  </div>
</template>
<style src="./assets/styles/orders.css"></style>
<style src="./assets/styles/workspace.css"></style>
<style src="./assets/styles/readability.css"></style>
<style src="./assets/styles/app-header.css"></style>
<style scoped>
.temporary-banner,.temporary-tools{margin:16px 24px;padding:16px;display:flex;align-items:center;justify-content:space-between;gap:16px;border:1px solid var(--line2);border-radius:10px;background:white}.temporary-banner p{margin:6px 0 0;color:var(--muted);line-height:1.5}.temporary-tools>div{flex:1;max-width:440px}@media(max-width:700px){.temporary-banner,.temporary-tools{margin:12px;align-items:stretch;flex-direction:column}}
</style>
