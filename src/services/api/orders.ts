import type { EditOrderPayload, TrashOrderPayload, CreateOrderPayload, GetOrderPayload, OrderResponse, ListOrdersPayload, OrderListResponse, FindOrderPayload, OrderHistoryPayload, OrderHistoryResponse } from "../../types/ipc.ts";
import { request } from "./transport.ts";

export const ordersApi = {
  editOrder: (payload:EditOrderPayload)=>request<OrderResponse>("edit_normalized_order",{payload}),
  trashOrder: (payload:TrashOrderPayload)=>request<void>("trash_normalized_order",{payload}),
  createOrder: (payload: CreateOrderPayload) => request<OrderResponse>("create_order", { payload }),
  getOrder: (payload: GetOrderPayload) => request<OrderResponse>("get_order", { payload }),
  listOrders: (payload: ListOrdersPayload) => request<OrderListResponse>("list_reception_orders", { payload }),
  findOrder: (payload: FindOrderPayload) => request<OrderResponse>("find_reception_order", { payload }),
  getHistory: (payload: OrderHistoryPayload) => request<OrderHistoryResponse>("get_reception_order_history", { payload }),
};
