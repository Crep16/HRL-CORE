import { computed, inject, onBeforeUnmount, onMounted, provide, reactive, ref, watch } from "vue";
import type { InjectionKey } from "vue";
import { useAuthStore } from "../stores/useAuthStore.ts";
import { useOrderStore } from "../stores/useOrderStore.ts";
import { formatMinorUnits, parseMinorUnits } from "../utils/minorUnits.ts";
import type { CreateOrderPayload, OrderResponse, OrderSearchField } from "../types/ipc.ts";

import { useCompany } from './useCompany.ts';
import { legacyOrder } from '../utils/legacyOrder.ts';
function emptyDraft(): CreateOrderPayload {
  return {
    client: { id: null, fullName: "", phone: "", email: null },
    equipment: { id: null, item: "Laptop", brand: "", model: "", serial: "" },
    history: { previousWorkshop: false, previousWorkshopDetail: "", sufferedImpact: false, impactDetail: "", liquidSpill: false, liquidDetail: "" },
    folioPrefix: "ORD-", sequenceStart: 1, status: "En diagnóstico", currency: "MXN",
    receivedAt: new Date().toLocaleDateString("en-CA"), estimatedDeliveryAt: null,
    reportedFailure: "", plannedWork: "", receptionNotes: "", priceMinor: 0, devicePassword: "", initialPayment: null,
  };
}
export function createReceptionForm() {
  const auth = useAuthStore(); const store = useOrderStore();
  const { company } = useCompany();
  const draft = reactive(emptyDraft());
  const editExisting=ref(false), trashed=ref(false), delivered=ref(false), changePassword=ref(false), deleteOpen=ref(false);
  const deleteTarget=computed(()=>deleteOpen.value&&store.currentOrder?legacyOrder(store.currentOrder):undefined);
  const price = ref("0.00"); const deposit = ref("0.00"); const method = ref("");
  const editing = ref(false); const showPassword = ref(false); const message = ref("");
  const discardOpen = ref(false); const query = ref(""); const lookupId = ref("");
  const searchField = ref<OrderSearchField>("all");
  const oldestFirst = ref(false);
  const uncertain = ref(false);
  const listPage = ref(0);
  const pageSize = 5;
  const listPages = computed(() => Math.max(1, Math.ceil(store.total / pageSize)));
  const historyOpen = ref(false);
  const historyPage = ref(0);
  const historyPages = computed(() => Math.max(1, Math.ceil(store.historyTotal / 50)));
  const canViewHistory = computed(() => !!store.currentOrder && auth.can("logbook.view", "logbook", store.currentOrder.id));
  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  let pending: CreateOrderPayload | null = null;
  let baseline = "";
  const canEdit = computed(() => editing.value && !store.isLoading && !uncertain.value && auth.can(editExisting.value ? "orders.edit" : "orders.create", "clients"));
  const canPay = computed(() => canEdit.value && !editExisting.value && auth.can("payments.add", "clients"));
  const folio = computed(() => editing.value && !editExisting.value ? "Se asignará al guardar" : store.currentOrder?.folio ?? "Sin orden seleccionada");
  const errorMessage = computed(() => message.value || store.error?.message || "");
  const visibleOrders = computed(() => store.orders);
  const damageDetailsOpen = ref(false);
  const hasDamage = computed(() => draft.history.sufferedImpact || draft.history.liquidSpill);
  function showDamageDetails(value: boolean) {
    damageDetailsOpen.value = value;
    if (!value) {
      draft.history.sufferedImpact = false; draft.history.liquidSpill = false;
      draft.history.impactDetail = ''; draft.history.liquidDetail = '';
    }
  }
  const receivedLabel = computed(() => {
    const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(draft.receivedAt);
    return match ? `${match[3]}/${match[2]}/${match[1]}` : draft.receivedAt;
  });
  const priceLabel = computed(() => new Intl.NumberFormat(company.value.locale, {style:'currency',currency:draft.currency}).format(draft.priceMinor / 100));
  const depositLabel = computed(() => new Intl.NumberFormat(company.value.locale, {style:'currency',currency:draft.currency}).format((store.currentOrder?.netMinor ?? 0) / 100));
  const orderPosition = computed(() => {
    const index = store.orders.findIndex(order => order.id === store.currentOrder?.id);
    return index < 0 ? 0 : listPage.value * pageSize + index + 1;
  });
  async function goToOrder(index: number) {
    if (hasChanges.value || editing.value || store.isLoading || store.isListing || index < 0 || index >= store.total) return;
    const page = Math.floor(index / pageSize);
    if (page !== listPage.value) { listPage.value = page; await refreshList(false); }
    const order = store.orders[index % pageSize];
    if (order) await select(order.id);
  }
  async function refreshList(selectFirst = true) {
    clearTimeout(searchTimer);
    searchTimer = undefined;
    if (!auth.can("clients.view", "clients")) { store.clearList(); return; }
    try {
      const response = await store.listOrders({ trashed: trashed.value, query: query.value, field: searchField.value, oldestFirst: oldestFirst.value, offset: listPage.value * pageSize, limit: pageSize });
      if (response && listPage.value >= listPages.value) { listPage.value = listPages.value - 1; await refreshList(false); return; }
      if (response && selectFirst && !editing.value && !store.currentOrder && response.items[0]) await select(response.items[0].id);
    } catch { /* El listado publica su error sin alterar una captura ni un guardado confirmado. */ }
  }
  function changeListPage(page: number) {
    if (!Number.isInteger(page) || page < 0 || page >= listPages.value || store.isListing) return;
    listPage.value = page;
    void refreshList(false);
  }
  watch([query, searchField, oldestFirst], () => {
    listPage.value = 0;
    clearTimeout(searchTimer);
    searchTimer = setTimeout(() => { void refreshList(false); }, 250);
  });
  watch(() => auth.can("clients.view", "clients"), (allowed) => {
    if (allowed) void refreshList();
    else { store.clearList(); store.clearOrder(); clearSecret(); }
  });
  onMounted(() => { void refreshList(); });
  async function loadHistory(page = 0) {
    if (!canViewHistory.value || !store.currentOrder || store.isHistoryLoading || !Number.isInteger(page) || page < 0 || page >= historyPages.value) return;
    historyPage.value = page;
    try { await store.getHistory({ orderId: store.currentOrder.id, offset: page * 50, limit: 50 }); }
    catch { historyPage.value = 0; /* El historial conserva su error y permite volver a consultar. */ }
  }
  function openHistory() { if (!canViewHistory.value) return; store.clearHistory(); historyPage.value = 0; historyOpen.value = true; void loadHistory(); }
  function closeHistory() { historyOpen.value = false; }
  watch(() => store.currentOrder?.id, closeHistory);
  watch(canViewHistory, (allowed) => { if (!allowed) { closeHistory(); store.clearHistory(); } });
  function snapshot() { return JSON.stringify({ draft, delivered:delivered.value, changePassword:changePassword.value, price: price.value, deposit: deposit.value, method: method.value }); }
  const hasChanges = computed(() => uncertain.value || (editing.value && snapshot() !== baseline));
  function clearSecret() { draft.devicePassword = ""; showPassword.value = false; }
  function begin() {
    if (store.isLoading || uncertain.value || hasChanges.value) { message.value = "Guarda o descarta la captura actual antes de crear otra orden."; return; }
    if (!auth.can("orders.create", "clients")) { message.value = "Solicita autorización para crear órdenes."; return; }
    editExisting.value=false;trashed.value=false;delivered.value=false;changePassword.value=false;
    damageDetailsOpen.value=false;
    Object.assign(draft, emptyDraft(), {folioPrefix:company.value.prefix,sequenceStart:company.value.nextNumber,currency:company.value.currency}); price.value = deposit.value = "0.00"; method.value = "";
    editing.value = true; message.value = ""; store.error = null; showPassword.value = false;
    baseline = snapshot();
  }
  function display(order: OrderResponse) {
    damageDetailsOpen.value = false;
    Object.assign(draft, {
      client: { ...order.client }, equipment: { ...order.equipment }, history: { ...order.history },
      status: order.status, currency: order.currency, receivedAt: order.receivedAt,
      estimatedDeliveryAt: order.estimatedDeliveryAt, reportedFailure: order.reportedFailure,
      plannedWork: order.plannedWork, receptionNotes: order.receptionNotes, priceMinor: order.priceMinor,
      devicePassword: "", initialPayment: null,
    });
    delivered.value=order.delivered;editExisting.value=false;changePassword.value=false;
    price.value = formatMinorUnits(order.priceMinor); deposit.value = formatMinorUnits(order.netMinor);
    method.value = ""; editing.value = false; clearSecret();
  }
  async function submit() {
    if (store.isLoading) return;
    if (!canEdit.value && !pending) return;
    message.value = "";
    try {
      if (!pending) {
        const priceMinor = parseMinorUnits(price.value);
        const amountMinor = parseMinorUnits(deposit.value);
        const payload: CreateOrderPayload = JSON.parse(JSON.stringify(draft)) as CreateOrderPayload;
        payload.priceMinor = priceMinor;
        payload.devicePassword = draft.devicePassword || null;
        payload.initialPayment = !editExisting.value && amountMinor > 0 ? { requestId: crypto.randomUUID(), amountMinor, method: method.value } : null;
        pending = payload;
      }
      const result = editExisting.value && store.currentOrder ? await store.editOrder({orderId:store.currentOrder.id,revision:store.currentOrder.revision,value:pending,delivered:delivered.value,changePassword:changePassword.value}) : await store.createOrder(pending);
      pending = null; uncertain.value = false; display(result);
      listPage.value = 0;
      await refreshList(false);
    } catch (cause) {
      const code = typeof cause === "object" && cause !== null && "code" in cause ? cause.code : "";
      uncertain.value = ["RESULT_UNCERTAIN", "IPC_ERROR", "SESSION_CHANGED"].includes(String(code));
      if (!uncertain.value) { if (pending) pending.devicePassword = null; pending = null; }
      if (cause instanceof Error) message.value = cause.message;
    }
  }
  const canRetry = computed(() => uncertain.value && !!pending?.initialPayment && !store.isLoading);
  function cancel() {
    if (uncertain.value) { message.value = "Confirma el resultado antes de descartar. Mantén esta ventana abierta para conservar la solicitud."; return; }
    if (hasChanges.value) { discardOpen.value = true; return; }
    discard();
  }
  function discard() {
    clearSecret(); editing.value = false; discardOpen.value = false; message.value = "";
    if (store.currentOrder) display(store.currentOrder);
  }
  async function select(id: number) {
    if (hasChanges.value || store.isLoading) { message.value = "Guarda o descarta los cambios antes de consultar otra orden."; return; }
    try { display(await store.getOrder({ orderId: id, scope: "clients", includeDeleted:trashed.value })); message.value = ""; } catch { /* Error disponible en el store. */ }
  }
  function edit() {
    if(!store.currentOrder||store.isLoading||trashed.value||!auth.can('orders.edit','clients',store.currentOrder.id))return;
    display(store.currentOrder);editExisting.value=true;editing.value=true;baseline=snapshot();
  }
  async function toggleTrash(){
    if(hasChanges.value||store.isLoading)return;
    trashed.value=!trashed.value;store.clearOrder();listPage.value=0;query.value='';await refreshList();
  }
  async function setDeleted(deleted:boolean){
    const order=store.currentOrder;if(!order||store.isLoading||hasChanges.value)return;
    try{await store.trashOrder({orderId:order.id,revision:order.revision,deleted});deleteOpen.value=false;await refreshList();}catch{/* El store conserva el error. */}
  }
  async function lookup() {
    if (hasChanges.value || store.isLoading) { message.value = "Guarda o descarta los cambios antes de consultar otra orden."; return; }
    const reference = lookupId.value.trim();
    if (!reference || reference.length > 100) { message.value = "Escribe un folio o identificador de orden válido."; return; }
    try { display(await store.findOrder({ reference })); message.value = ""; }
    catch { /* El store publica el error. */ }
  }
  async function togglePassword() {
    if (showPassword.value) { showPassword.value = false; if (!editing.value) draft.devicePassword = ""; return; }
    if (!editing.value && store.currentOrder?.hasDevicePassword) {
      try {
        const response = await store.getOrder({ orderId: store.currentOrder.id, scope: "clients", includeDeleted:trashed.value, includeDevicePassword: true });
        draft.devicePassword = response.devicePassword ?? "";
      } catch { return; }
    }
    showPassword.value = true;
  }
  async function prepareLeave() {
    if (store.isLoading || hasChanges.value) { message.value = "Guarda o descarta la captura antes de salir de Recepción."; return false; }
    clearSecret();
    return true;
  }
  function beforeUnload(event: BeforeUnloadEvent) { if (hasChanges.value) { event.preventDefault(); event.returnValue = ""; } }
  window.addEventListener("beforeunload", beforeUnload);
  watch(() => auth.sessionVersion, () => { clearSecret(); });
  onBeforeUnmount(() => { clearTimeout(searchTimer); clearSecret(); if (pending) pending.devicePassword = null; pending = null; window.removeEventListener("beforeunload", beforeUnload); });
  if (store.currentOrder) display(store.currentOrder);
  return { damageDetailsOpen,hasDamage,showDamageDetails,receivedLabel,priceLabel,depositLabel,orderPosition,goToOrder,editExisting,trashed,delivered,changePassword,deleteOpen,deleteTarget,edit,toggleTrash,setDeleted,auth, store, draft, price, deposit, method, editing, canEdit, canPay, folio, showPassword, message, errorMessage, hasChanges, query, searchField, oldestFirst, visibleOrders, lookupId, discardOpen, uncertain, canRetry, begin, submit, cancel, discard, select, lookup, togglePassword, prepareLeave, listPage, listPages, refreshList, changeListPage, historyOpen, historyPage, historyPages, canViewHistory, loadHistory, openHistory, closeHistory };
}
export type ReceptionForm = ReturnType<typeof createReceptionForm>;
const key: InjectionKey<ReceptionForm> = Symbol("reception-form");
export function provideReceptionForm() { const form = createReceptionForm(); provide(key, form); return form; }
export function useReceptionForm() { const form = inject(key); if (!form) throw new Error("El formulario requiere la vista de Recepción."); return form; }
