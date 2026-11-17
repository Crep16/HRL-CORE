<script setup lang="ts">
import DraftLeaveDialog from '../components/ui/DraftLeaveDialog.vue';
import { nextTick, ref, watch } from "vue";
import OrderDeleteDialog from '../components/orders/OrderDeleteDialog.vue';
import AppHeader from "../components/layout/AppHeader.vue";
import AppIcon from "../components/ui/AppIcon.vue";
import OrderForm from "../components/orders/OrderForm.vue";
import OrderSidePanel from "../components/orders/OrderSidePanel.vue";
import TemporaryAccessDialog from "../components/access/TemporaryAccessDialog.vue";
import OrderHistoryDialog from "../components/orders/OrderHistoryDialog.vue";
import { provideReceptionForm } from "../composables/useReceptionForm";
import type { TemporaryAccessContext } from "../composables/useTemporaryAccessForm";
import { statusTone } from "../data/orderStatuses";
withDefaults(defineProps<{embedded?:boolean}>(),{embedded:false});
const emit=defineEmits<{repairs:[]}>();
const { leaveGuard, receivedLabel,orderPosition,goToOrder,editExisting,trashed,deleteOpen,deleteTarget,edit,toggleTrash,setDeleted,auth, store, draft, editing, folio, errorMessage, hasChanges, query, searchField, oldestFirst, visibleOrders, lookupId, discardOpen, uncertain, canRetry, begin, submit, cancel, discard, select, lookup, prepareLeave, listPage, listPages, refreshList, changeListPage, canViewHistory, openHistory } = provideReceptionForm();
const lookupDialog = ref<HTMLDialogElement>();
function showLookup() { lookupDialog.value?.showModal(); }
async function lookupAndClose() { await lookup(); if (!errorMessage.value) lookupDialog.value?.close(); }
const searchFields = [{ value: 'all', label: 'Todo' }, { value: 'order', label: 'Orden' }, { value: 'client', label: 'Cliente' }, { value: 'serial', label: 'Serie' }] as const;
const request = ref<TemporaryAccessContext | null>(null);
watch(editing, async (active) => { if (active) { await nextTick(); document.getElementById("cliente")?.focus(); } });
const discardDialog = ref<HTMLDialogElement>();
let previousFocus: HTMLElement | null = null;
watch(discardOpen, async (open) => {
  await nextTick();
  if (open) { previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null; discardDialog.value?.showModal(); }
  else { discardDialog.value?.close(); previousFocus?.focus(); }
});
async function logout() { if (await prepareLeave()) { try { await auth.logoutUser(); } catch { /* El store publica el error. */ } } }
async function finishTemporary() { try { await auth.finishTemporaryAccess(); } catch { /* El store publica el error. */ } }
defineExpose({ prepareLeave,editing,hasChanges,trashed,showLookup,openHistory,canViewHistory,refreshList });
</script>
<template>
  <div :class="embedded ? 'normalized-reception' : 'app normalized-workshop'">
    <AppHeader v-if="!embedded" :tabs="[]" active="Clientes" heading="Clientes · Recepción" :user="auth.user"
      :print-actions="false" temporary-access :busy="auth.isLoading || store.isLoading"
      @request-access="request = { operation: 'orders.create', scope: 'clients', orderId: null }" @logout="logout" />
    <div v-if="!embedded && auth.temporaryAccess" role="status" class="notice"><span>Autorización temporal activa. Tu identidad como operador se conserva.</span><button type="button" class="btn btn-s" :disabled="auth.isLoading || store.isLoading" @click="finishTemporary">Finalizar acceso temporal</button></div>
    <p v-if="errorMessage || auth.error" role="alert" class="notice workshop-error">{{ errorMessage || auth.error?.message }}</p>
    <p v-if="store.isLoading" role="status" class="notice">Consultando o guardando la orden…</p>
    <main class="work reception">
      <aside class="rail" aria-label="Órdenes del taller">
        <div class="rail-h"><div><h1 tabindex="-1" class="h-disp">{{ trashed ? 'Papelera' : 'Órdenes' }}</h1><p class="subt">{{ store.total }} en total</p></div><button v-if="!trashed" type="button" class="btn btn-p" :disabled="store.isLoading || uncertain || !auth.can('orders.create', 'clients')" @click="begin"><AppIcon name="add" />Nueva orden</button></div>
        <label for="q" class="sr">Buscar órdenes por folio, cliente o serie</label>
        <div class="search"><AppIcon name="search" /><input id="q" v-model="query" type="search" placeholder="Orden, cliente o serie" maxlength="100" :disabled="store.isLoading"></div>
        <div class="seg" role="group" aria-label="Buscar en">
          <button v-for="field in searchFields" :key="field.value" type="button" :class="{ on: searchField === field.value }" :aria-pressed="searchField === field.value" :disabled="!auth.can('clients.view', 'clients')" @click="searchField = field.value">{{ field.label }}</button>
        </div>
        <div class="tools"><button v-if="trashed || auth.can('orders.delete','clients')" class="lnk trash-toggle" type="button" :disabled="hasChanges || store.isLoading" @click="toggleTrash"><AppIcon :name="trashed ? 'clients' : 'trash'" />{{ trashed ? 'Ver órdenes' : 'Papelera' }}</button><button type="button" class="lnk" :aria-pressed="oldestFirst" :disabled="!auth.can('clients.view', 'clients')" :title="oldestFirst ? 'Primero las más antiguas' : 'Primero las más recientes'" @click="oldestFirst = !oldestFirst"><svg class="i sm sort-icon" :class="{ reversed: oldestFirst }" viewBox="0 0 24 24" aria-hidden="true"><path d="m21 16-4 4-4-4M17 20V4M3 8l4-4 4 4M7 4v16" /></svg>Cambiar orden</button></div>
        <p v-if="store.isListing" role="status" class="help">Buscando órdenes…</p>
        <p v-if="store.listError" role="alert" class="help workshop-list-error">{{ store.listError.message }}</p>
        <ul class="olist" :aria-busy="store.isListing"><li v-for="order in visibleOrders" :key="order.id"><button type="button" class="orow" :class="{ sel: !editing && store.currentOrder?.id === order.id }" :aria-current="!editing && store.currentOrder?.id === order.id ? 'true' : undefined" :disabled="store.isLoading || hasChanges" @click="select(order.id)"><span class="o-top"><strong class="o-name">{{ order.clientName }}</strong><span class="o-id">{{ order.folio }}</span></span><span class="o-bot"><span class="st"><i class="dot" :class="statusTone(order.status)" aria-hidden="true"></i>{{ order.status }}</span><span class="o-sn">{{ order.serial || 'Sin serie' }}</span></span></button></li></ul>
        <p v-if="!store.isListing && !store.listError && !visibleOrders.length" class="empty-list">{{ auth.can('clients.view', 'clients') ? 'No hay órdenes que coincidan.' : 'Solicita autorización para consultar órdenes.' }}</p>
        <div class="order-pagination" aria-label="Páginas de órdenes"><div class="pagination-summary"><strong>Página {{ listPage + 1 }} de {{ listPages }}</strong><span>{{ store.total ? listPage * 5 + 1 : 0 }}–{{ Math.min((listPage + 1) * 5, store.total) }} de {{ store.total }} órdenes</span></div><div class="pagination-buttons"><button type="button" class="btn btn-s" :disabled="store.isListing || listPage === 0" @click="changeListPage(listPage - 1)"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6" /></svg>Anterior</button><button type="button" class="btn btn-s" :disabled="store.isListing || listPage + 1 >= listPages" @click="changeListPage(listPage + 1)">Siguiente<svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg></button></div></div>
      </aside>
      <section class="sheet" aria-label="Ficha de recepción">
        <template v-if="editing || store.currentOrder">
          <header class="sh-h">
            <div><h2 class="num">{{ editing && !editExisting ? 'Nueva orden' : folio }}</h2><p v-if="trashed" class="subt">En la papelera · historial y cobros conservados</p><p class="who">{{ draft.client.fullName }}</p><p class="subt">Recibida el {{ receivedLabel }}</p></div>
            <div class="sh-act">
              <template v-if="editing"><button type="submit" form="reception-form" class="btn btn-p" :disabled="store.isLoading || uncertain || !auth.can(editExisting ? 'orders.edit' : 'orders.create','clients')">Guardar cambios</button><button type="button" class="btn btn-s" :disabled="store.isLoading || uncertain" @click="cancel">Cancelar</button></template>
              <template v-else>
                <div class="navgrp" role="group" aria-label="Avanzar entre órdenes">
                  <button type="button" aria-label="Primera orden" :disabled="store.isLoading || orderPosition <= 1" @click="goToOrder(0)"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m11 17-5-5 5-5m7 10-5-5 5-5" /></svg></button>
                  <button type="button" aria-label="Orden anterior" :disabled="store.isLoading || orderPosition <= 1" @click="goToOrder(orderPosition-2)"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6" /></svg></button>
                  <span>{{ orderPosition || 'Fuera del filtro' }} de {{ store.total }}</span>
                  <button type="button" aria-label="Orden siguiente" :disabled="store.isLoading || !orderPosition || orderPosition >= store.total" @click="goToOrder(orderPosition)"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg></button>
                  <button type="button" aria-label="Última orden" :disabled="store.isLoading || orderPosition >= store.total" @click="goToOrder(store.total-1)"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m6 17 5-5-5-5m7 10 5-5-5-5" /></svg></button>
                </div>
                <button v-if="!trashed" type="button" class="btn btn-s" :disabled="store.isLoading || !auth.can('orders.edit','clients')" @click="edit"><AppIcon name="edit" />Editar</button>
                <button type="button" class="btn" :class="trashed ? 'btn-p' : 'btn-s'" :disabled="store.isLoading || !auth.can('orders.delete','clients')" @click="trashed ? setDeleted(false) : deleteOpen=true"><AppIcon :name="trashed ? 'restore' : 'trash'" />{{ trashed ? 'Restaurar orden' : 'Eliminar' }}</button>
              </template>
            </div>
          </header>
          <div class="perf" aria-hidden="true"></div>
          <div v-if="uncertain" role="alert" class="notice workshop-warning"><p>No se confirmó el resultado. Mantén esta ventana abierta y conserva la solicitud antes de realizar otra operación.</p><button v-if="canRetry" type="button" class="btn btn-s" @click="submit">Reintentar la misma solicitud</button><p v-else>Consulta el resultado con soporte antes de volver a crear esta orden.</p></div>
          <div class="sh-b" :aria-busy="store.isLoading"><OrderForm /><OrderSidePanel @repairs="emit('repairs')" /></div>
        </template>
        <div v-else class="empty-work">
          <h2 class="h-disp">{{ store.isListing ? 'Consultando órdenes…' : !auth.can('clients.view', 'clients') ? 'Consulta de órdenes restringida' : store.listError ? 'No se pudo cargar el listado' : query ? 'Sin coincidencias' : store.total ? 'Selecciona una orden' : 'No hay órdenes' }}</h2>
          <p>{{ !auth.can('clients.view', 'clients') ? 'Solicita acceso temporal para consultar las órdenes del taller.' : store.listError ? 'Vuelve a consultar el listado para ver las órdenes guardadas.' : store.isListing ? 'Cargando el listado del taller.' : query ? 'Prueba otro folio, nombre de cliente o número de serie.' : store.total ? 'Elige una orden del listado o consulta su folio o identificador.' : 'Crea una nueva orden para comenzar.' }}</p>
          <button v-if="store.listError && auth.can('clients.view', 'clients')" type="button" class="btn btn-p" :disabled="store.isListing" @click="refreshList()">Volver a consultar</button>
          <button v-else-if="!trashed && auth.can('orders.create', 'clients')" type="button" class="btn btn-p" :disabled="store.isLoading || uncertain" @click="begin">Nueva orden</button>
        </div>
      </section>
    </main>
    <DraftLeaveDialog id="reception-leave" area="Recepción" :open="leaveGuard.open.value" :busy="leaveGuard.busy.value" :error="leaveGuard.error.value" @save="leaveGuard.save" @discard="leaveGuard.discard" @keep="leaveGuard.keep" />
    <TemporaryAccessDialog :request="request" choose-operation @close="request = null" @approved="request = null" />
    <OrderHistoryDialog /><OrderDeleteDialog :order="deleteTarget" :busy="store.isLoading" :error="errorMessage" @confirm="setDeleted(true)" @cancel="deleteOpen=false" />
    <dialog ref="lookupDialog" class="workshop-dialog" aria-labelledby="lookup-title"><header><h2 id="lookup-title">Consultar orden</h2><button type="button" class="btn btn-s" @click="lookupDialog?.close()">Cerrar</button></header><form @submit.prevent="lookupAndClose"><label for="lookup-order">Folio o identificador</label><input id="lookup-order" v-model="lookupId" maxlength="100" required :disabled="store.isLoading || hasChanges"><p v-if="errorMessage" role="alert">{{ errorMessage }}</p><footer><button type="submit" class="btn btn-p" :disabled="store.isLoading || hasChanges || !auth.can('clients.view','clients')">Consultar orden</button></footer></form></dialog>
    <dialog ref="discardDialog" aria-labelledby="discard-title" class="workshop-dialog" @cancel.prevent="discardOpen = false"><h2 id="discard-title">Descartar la captura</h2><p>Los datos sin guardar se perderán.</p><footer><button type="button" autofocus class="btn btn-s" @click="discardOpen = false">Seguir capturando</button><button type="button" class="btn workshop-danger" @click="discard">Descartar cambios</button></footer></dialog>
  </div>
</template>
<style src="../assets/styles/orders.css"></style>
<style src="../assets/styles/workspace.css"></style>
<style src="../assets/styles/app-header.css"></style>
<style src="../assets/styles/readability.css"></style>
<style src="../assets/styles/normalized-workshop.css"></style>
