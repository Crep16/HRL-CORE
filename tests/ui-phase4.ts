import { mockIPC } from "@tauri-apps/api/mocks";
import type { CreateOrderPayload, ListOrdersPayload, OrderResponse, SessionResponse } from "../src/types/ipc";
const session: SessionResponse = {
  user: { id: 1, name: "Administrador de prueba", loginName: "admin", role: "administrator", accessLevel: "complete", active: true, createdAt: "2026-11-04" },
  permissions: ["orders.create", "clients.view", "payments.add", "logbook.view"],
};
const orders = new Map<number, OrderResponse>();
const seen = new Map<string, OrderResponse>();
const summary = (order: OrderResponse) => ({ id:order.id,folio:order.folio,clientName:order.client.fullName,serial:order.equipment.serial,item:order.equipment.item,status:order.status,receivedAt:order.receivedAt,currency:order.currency,priceMinor:order.priceMinor,netMinor:order.netMinor,pendingMinor:order.pendingMinor,creditMinor:order.creditMinor,delivered:order.delivered });
let nextId = 1;
const loginRequests: unknown[] = [];
Object.assign(window,{loginRequests});
mockIPC((command, args) => {
  if (command === 'load_normalized_company') return null;
  if (command === 'list_login_users') {
    if (new URLSearchParams(location.search).has('usersError')) throw {code:'STORAGE_ERROR',message:'Fallo de lista de prueba'};
    return ['admin','ana','hector','lulu'];
  }
  if (command === "check_setup_status") return { setupComplete: !new URLSearchParams(location.search).has('setup') };
  if (command === "access_status") return { setupComplete: true, user: null };
  if (command === "login_user") { loginRequests.push(structuredClone(args)); return session; }
  if (command === "initialize_setup") return session;
  if (command === "logout_user" || command === "finish_temporary_access" || command === "sync_normalized_window_profile") return null;
  if (command === "list_reception_orders") {
    const { query, offset, limit, field = 'all', oldestFirst = false } = (args as { payload: ListOrdersPayload }).payload;
    const matches = [...orders.values()].filter(order => {
      const fields = { order:order.folio,client:order.client.fullName,serial:order.equipment.serial };
      return (field === 'all' ? Object.values(fields) : [fields[field]]).some(value => value.toLocaleLowerCase('es').includes(query.toLocaleLowerCase('es')));
    }).sort((a,b) => oldestFirst ? a.id-b.id : b.id-a.id);
    return { items:matches.slice(offset,offset+limit).map(summary),total:matches.length };
  }
  if (command === "find_reception_order") {
    const { reference } = (args as { payload: { reference:string } }).payload;
    const order = [...orders.values()].find(order => order.folio === reference) ?? orders.get(Number(reference));
    if (!order) throw { code:"NOT_FOUND",message:"Orden de prueba no encontrada." };
    return order;
  }
  if (command === "get_reception_order_history") {
    const { orderId } = (args as { payload: { orderId:number } }).payload;
    if (!orders.has(orderId)) throw { code:"NOT_FOUND",message:"Orden de prueba no encontrada." };
    return { items:[{ id:1,authorId:1,authorName:"Administrador de prueba",operation:"orders.create",entity:"ordenes",entityId:String(orderId),reason:null,result:"success",createdAt:"2026-11-05T12:00:00Z",changes:[] }],total:1 };
  }
  if (command === "generate_recovery_challenge") return { challenge: "HRL-CORE:master-recovery:v1:1:desafio-ficticio-para-prueba", adminId: 1, adminUsername: "admin", expiresInSeconds: 900 };
  if (command === "verify_recovery_signature") return { adminId: 1, requiresLogin: true };
  if (command === "request_temporary_access") return { id: 1, recipientId: 1, approverId: 2, scope: "clients", orderId: null, operations: ["orders.create", "clients.view", "payments.add"] };
  if (command === "create_order") {
    const payload = (args as { payload: CreateOrderPayload }).payload;
    if (!Number.isSafeInteger(payload.priceMinor) || !Number.isSafeInteger(payload.initialPayment?.amountMinor ?? 0)) throw { code: "VALIDATION_ERROR", message: "Centavos inválidos en la prueba." };
    const requestId = payload.initialPayment?.requestId;
    if (requestId && seen.has(requestId)) return seen.get(requestId);
    const id = nextId++;
    const net = payload.initialPayment?.amountMinor ?? 0;
    const response: OrderResponse = {
      id, folio: `ORD-${String(id).padStart(3, "0")}`, clientId: id, equipmentId: id, receivedBy: 1,
      status: payload.status, currency: payload.currency, receivedAt: payload.receivedAt, estimatedDeliveryAt: payload.estimatedDeliveryAt,
      reportedFailure: payload.reportedFailure, plannedWork: payload.plannedWork, receptionNotes: payload.receptionNotes,
      priceMinor: payload.priceMinor, netMinor: net, pendingMinor: Math.max(0, payload.priceMinor - net), creditMinor: 0,
      hasDevicePassword: !!payload.devicePassword, client: { ...payload.client, id }, equipment: { ...payload.equipment, id }, history: payload.history,
      revision: 1, delivered: false, deliveredAt: null,
    };
    orders.set(id, response);
    if (requestId) seen.set(requestId, response);
    document.getElementById("test-ipc-received")!.textContent = `Prueba IPC: costo=${payload.priceMinor}; abono=${net} centavos`;
    return response;
  }
  if (command === "get_order") {
    const payload = (args as { payload: { orderId: number; includeDevicePassword?: boolean } }).payload;
    const order = orders.get(payload.orderId);
    if (!order) throw { code: "NOT_FOUND", message: "Orden de prueba no encontrada." };
    return payload.includeDevicePassword ? { ...order, devicePassword: "pin-ficticio" } : order;
  }
  throw { code: "VALIDATION_ERROR", message: `Comando no simulado: ${command}` };
});
await import("../src/main.ts");
