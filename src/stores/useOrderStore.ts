import { defineStore } from "pinia";
import { ref, watch } from "vue";
import { ordersApi } from "../services/api/orders.ts";
import { useIpcTask } from "../composables/useIpcTask.ts";
import { useAuthStore } from "./useAuthStore.ts";
import { normalizeIpcError } from "../services/api/errors.ts";
import type { ClientError, EditOrderPayload, TrashOrderPayload, CreateOrderPayload, GetOrderPayload, OrderResponse, OrderListItem, ListOrdersPayload, FindOrderPayload, OrderHistoryPayload, OrderHistoryEvent, StoreError } from "../types/ipc.ts";

export const useOrderStore = defineStore("orders", () => {
  const currentOrder = ref<OrderResponse | null>(null);
  const sessionOrders = ref<OrderResponse[]>([]);
  const orders = ref<OrderListItem[]>([]);
  const total = ref(0);
  const isListing = ref(false);
  const listError = ref<StoreError | null>(null);
  const history = ref<OrderHistoryEvent[]>([]);
  const historyTotal = ref(0);
  const historyTask = useIpcTask(() => { history.value = []; historyTotal.value = 0; });
  let listVersion = 0;
  const auth = useAuthStore();
  const task = useIpcTask((error) => {
    if (error.code === "FORBIDDEN") clearOrder();
    if (error.code === "UNAUTHENTICATED") auth.clearSession();
  });
  function clearOrder() { currentOrder.value = null; }
  function clearList() { listVersion += 1; orders.value = []; total.value = 0; isListing.value = false; listError.value = null; }
  function clearHistory() { history.value = []; historyTotal.value = 0; historyTask.error.value = null; }
  watch(() => currentOrder.value?.id, clearHistory, { flush: "sync" });
  watch(() => auth.sessionVersion, () => { clearOrder(); clearList(); clearHistory(); sessionOrders.value = []; }, { flush: "sync" });

  async function listOrders(payload: ListOrdersPayload) {
    const requestVersion = ++listVersion;
    const sessionVersion = auth.sessionVersion;
    isListing.value = true;
    listError.value = null;
    try {
      const response = await ordersApi.listOrders(payload);
      if (requestVersion !== listVersion || sessionVersion !== auth.sessionVersion) return;
      orders.value = response.items;
      total.value = response.total;
      return response;
    } catch (cause) {
      const error = normalizeIpcError(cause);
      if (requestVersion === listVersion && sessionVersion === auth.sessionVersion) {
        orders.value = []; total.value = 0; listError.value = error;
        if (error.code === "UNAUTHENTICATED") auth.clearSession();
        if (error.code === "FORBIDDEN") { clearOrder(); sessionOrders.value = []; }
      }
      throw error;
    } finally {
      if (requestVersion === listVersion) isListing.value = false;
    }
  }

  function acceptOrder(order: OrderResponse, version: number) {
    if (version !== auth.sessionVersion) {
      throw { code: "SESSION_CHANGED", message: "La sesión cambió durante la consulta de la orden." } satisfies ClientError;
    }
    // El secreto solicitado se devuelve al llamador; no se conserva en el estado global.
    const { devicePassword: _secret, ...publicOrder } = order;
    currentOrder.value = publicOrder;
    const index = sessionOrders.value.findIndex((item) => item.id === order.id);
    if (index < 0) sessionOrders.value.push(publicOrder);
    else sessionOrders.value[index] = publicOrder;
    return order;
  }
  function createOrder(payload: CreateOrderPayload) {
    return task.run(async () => {
      const version = auth.sessionVersion;
      // No se modifica el id financiero ni el contenido: un reintento debe ser idéntico.
      const order = await ordersApi.createOrder(payload);
      return acceptOrder(order, version);
    });
  }
  function editOrder(payload:EditOrderPayload) {return task.run(async()=>{const version=auth.sessionVersion;return acceptOrder(await ordersApi.editOrder(payload),version);});}
  function trashOrder(payload:TrashOrderPayload) {return task.run(async()=>{await ordersApi.trashOrder(payload);clearOrder();});}
  function getOrder(payload: GetOrderPayload) {
    return task.run(async () => {
      const version = auth.sessionVersion;
      const order = await ordersApi.getOrder(payload);
      return acceptOrder(order, version);
    });
  }
  function findOrder(payload: FindOrderPayload) {
    return task.run(async () => {
      const version = auth.sessionVersion;
      return acceptOrder(await ordersApi.findOrder(payload), version);
    });
  }
  function getHistory(payload: OrderHistoryPayload) {
    return historyTask.run(async () => {
      const version = auth.sessionVersion;
      const response = await ordersApi.getHistory(payload);
      if (version !== auth.sessionVersion || currentOrder.value?.id !== payload.orderId) {
        throw { code: "SESSION_CHANGED", message: "La orden o la sesión cambió. Vuelve a consultar el historial." } satisfies ClientError;
      }
      history.value = response.items;
      historyTotal.value = response.total;
      return response;
    });
  }
  return {
    currentOrder, sessionOrders, isLoading: task.isLoading, error: task.error,
    orders, total, isListing, listError, history, historyTotal,
    isHistoryLoading: historyTask.isLoading, historyError: historyTask.error,
    createOrder, editOrder, trashOrder, getOrder, findOrder, listOrders, getHistory, clearOrder, clearList, clearHistory,
  };
});
