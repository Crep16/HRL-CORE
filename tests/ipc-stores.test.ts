import { afterEach, beforeEach, test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createPinia, disposePinia, setActivePinia } from "pinia";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { useAuthStore } from "../src/stores/useAuthStore.ts";
import { useRecoveryStore } from "../src/stores/useRecoveryStore.ts";
import { useOrderStore } from "../src/stores/useOrderStore.ts";
import { useWorkshopDataStore } from "../src/stores/useWorkshopDataStore.ts";
import { normalizeIpcError } from "../src/services/api/errors.ts";
import { windowApi } from "../src/services/api/window.ts";
import type {
  CreateOrderPayload, OrderResponse, SessionResponse, StoreError,
  TemporaryAccessPayload, TemporaryAccessResponse, VerifyRecoveryPayload, OrderListItem, OrderHistoryResponse,
} from "../src/types/ipc.ts";

globalThis.window = {} as Window & typeof globalThis;
let pinia: ReturnType<typeof createPinia>;
beforeEach(() => { pinia = createPinia(); setActivePinia(pinia); });
afterEach(() => { clearMocks(); disposePinia(pinia); });

const session: SessionResponse = {
  user: { id: 1, name: "Prueba", loginName: "admin", role: "administrator", accessLevel: "complete", active: true, createdAt: "2026-11-04T00:00:00Z" },
  permissions: ["orders.create", "clients.view"],
};
const login = { username: "admin", password: "test-account-only" };
const input: CreateOrderPayload = {
  client: { id: null, fullName: "Cliente de prueba", phone: "0000000000", email: null },
  equipment: { id: null, item: "Laptop", brand: "Marca", model: "Modelo", serial: "TEST" },
  history: { previousWorkshop: false, previousWorkshopDetail: "", sufferedImpact: false, impactDetail: "", liquidSpill: false, liquidDetail: "" },
  folioPrefix: "ORD-", sequenceStart: 1, status: "En diagnóstico", currency: "MXN", receivedAt: "2026-11-04", estimatedDeliveryAt: null,
  reportedFailure: "No enciende", plannedWork: "Diagnóstico", receptionNotes: "", priceMinor: 150000,
  devicePassword: "test-device-only", initialPayment: { requestId: "fixed-initial-payment-request", amountMinor: 20000, method: "Método libre" },
};
const order: OrderResponse = {
  id: 1, folio: "ORD-001", clientId: 1, equipmentId: 1, receivedBy: 1, status: input.status, currency: "MXN",
  receivedAt: input.receivedAt, estimatedDeliveryAt: null, reportedFailure: input.reportedFailure, plannedWork: input.plannedWork, receptionNotes: "",
  priceMinor: 150000, netMinor: 20000, pendingMinor: 130000, creditMinor: 0, hasDevicePassword: true,
  client: { id: 1, fullName: input.client.fullName, phone: input.client.phone, email: null },
  equipment: { id: 1, item: "Laptop", brand: "Marca", model: "Modelo", serial: "TEST" }, history: input.history,
  revision: 1, delivered: false, deliveredAt: null,
};
const temporary: TemporaryAccessPayload = {
  username: "approver", password: "approver-test-only", scope: "repairs", orderId: 1, operation: "payments.add", reason: "Prueba",
};
const grant: TemporaryAccessResponse = { id: 4, recipientId: 1, approverId: 2, scope: "repairs", orderId: 1, operations: ["payments.add"] };
const recoveryInput: VerifyRecoveryPayload = { challenge: "test-challenge", signature: "00".repeat(64), newMasterPassword: "master-test-only", newAdminPassword: "admin-test-only" };
const hasCode = (code: StoreError["code"]) => (error: unknown) => normalizeIpcError(error).code === code;

test("financial, logbook and public-summary queries reject responses from the previous session", async () => {
  const auth = useAuthStore();
  const data = useWorkshopDataStore();
  const queries = [
    () => data.finances(),
    () => data.logbook({ query: "", operation: "", authorId: "", page: 0 }),
    () => data.summary(1),
  ];
  for (const query of queries) {
    mockIPC(() => session);
    await auth.loginUser(login);
    let release!: (result: unknown) => void;
    mockIPC(() => new Promise(resolve => { release = resolve; }));
    const pending = query();
    auth.clearSession();
    release([]);
    await assert.rejects(pending, hasCode("SESSION_CHANGED"));
    assert.equal(auth.user, null);
  }
  mockIPC(() => session);
  await auth.loginUser(login);
  mockIPC(() => { throw { code: "UNAUTHENTICATED", message: "Sesión revocada" }; });
  await assert.rejects(data.finances(), hasCode("UNAUTHENTICATED"));
  assert.equal(auth.user, null);
});

const listedOrder: OrderListItem = { id:order.id,folio:order.folio,clientName:order.client.fullName,serial:order.equipment.serial,item:order.equipment.item,status:order.status,receivedAt:order.receivedAt,currency:order.currency,priceMinor:order.priceMinor,netMinor:order.netMinor,pendingMinor:order.pendingMinor,creditMinor:order.creditMinor,delivered:order.delivered };
const historyResponse: OrderHistoryResponse = { items:[{ id:1,authorId:1,authorName:"Prueba",operation:"orders.create",entity:"ordenes",entityId:"1",reason:null,result:"success",createdAt:"2026-11-05T12:00:00Z",changes:[] }],total:1 };

test("persistent list, reference lookup and history use exact normalized IPC envelopes", async () => {
  const calls: string[] = [];
  mockIPC((command, args) => {
    calls.push(command);
    if (command === "list_reception_orders") { assert.deepEqual(args,{ payload:{ query:"SERIE",offset:5,limit:5,field:"serial",oldestFirst:true } }); return { items:[listedOrder],total:6 }; }
    if (command === "find_reception_order") { assert.deepEqual(args,{ payload:{ reference:order.folio } }); return order; }
    assert.equal(command,"get_reception_order_history");
    assert.deepEqual(args,{ payload:{ orderId:order.id,offset:0,limit:50 } });
    return historyResponse;
  });
  const store = useOrderStore();
  await store.listOrders({ query:"SERIE",offset:5,limit:5,field:"serial",oldestFirst:true });
  assert.equal(store.total,6); assert.deepEqual(store.orders,[listedOrder]);
  await store.findOrder({ reference:order.folio });
  await store.getHistory({ orderId:order.id,offset:0,limit:50 });
  assert.deepEqual(store.history,historyResponse.items);
  assert.deepEqual(calls,["list_reception_orders","find_reception_order","get_reception_order_history"]);
});

test("late searches and previous sessions cannot replace the latest persistent list", async () => {
  let release!: (value: unknown) => void;
  mockIPC((_command,args) => {
    const query = (args as { payload:{query:string} }).payload.query;
    if (query === "slow") return new Promise(resolve => { release = resolve; });
    return { items:[listedOrder],total:1 };
  });
  const store = useOrderStore();
  const slow = store.listOrders({ query:"slow",offset:0,limit:5 });
  await store.listOrders({ query:"latest",offset:0,limit:5 });
  release({ items:[],total:0 }); await slow;
  assert.equal(store.orders[0].id,order.id);
  const previousSession = store.listOrders({ query:"slow",offset:0,limit:5 });
  useAuthStore().clearSession();
  release({ items:[listedOrder],total:1 }); await previousSession;
  assert.deepEqual(store.orders,[]); assert.equal(store.total,0); assert.equal(store.isListing,false);
});

test("denied persistent queries clear cached data and changed orders reject stale history", async () => {
  let deny = false;
  let release!: (value: unknown) => void;
  mockIPC((command) => {
    if (command === "find_reception_order") return order;
    if (command === "get_reception_order_history") return new Promise(resolve => { release = resolve; });
    if (deny) throw { code:"FORBIDDEN",message:"Prueba sin permiso" };
    return { items:[listedOrder],total:1 };
  });
  const store = useOrderStore();
  await store.findOrder({ reference:order.folio });
  const history = store.getHistory({ orderId:order.id,offset:0,limit:50 });
  store.clearOrder(); release(historyResponse);
  await assert.rejects(history,hasCode("SESSION_CHANGED"));
  assert.deepEqual(store.history,[]);
  await store.listOrders({ query:"",offset:0,limit:5 });
  deny = true;
  await assert.rejects(store.listOrders({ query:"",offset:0,limit:5 }),hasCode("FORBIDDEN"));
  assert.deepEqual(store.orders,[]); assert.equal(store.total,0); assert.equal(store.currentOrder,null); assert.deepEqual(store.sessionOrders,[]);
});

test("window synchronization delegates its profile to the normalized backend without frontend arguments", async () => {
  mockIPC((command, args) => {
    assert.equal(command, "sync_normalized_window_profile");
    assert.deepEqual(args, {});
    return null;
  });
  await windowApi.synchronizeProfile();
});

test("auth sends exact envelopes and loading excludes duplicate IPC; rejected login preserves session", async () => {
  let resolve!: (value: SessionResponse) => void;
  const calls: string[] = [];
  mockIPC((command, args) => {
    calls.push(command);
    if (command === "check_setup_status") return { setupComplete: true };
    assert.equal(command, "login_user");
    assert.deepEqual(args, { payload: login });
    return new Promise<SessionResponse>((done) => { resolve = done; });
  });
  const auth = useAuthStore();
  assert.equal(auth.setupComplete, null);
  await auth.checkSetupStatus();
  const pending = auth.loginUser(login);
  assert.equal(auth.isLoading, true);
  await assert.rejects(auth.loginUser(login), hasCode("BUSY"));
  resolve(session);
  await pending;
  assert.equal(auth.isLoading, false);
  assert.equal(auth.error, null);
  assert.deepEqual(auth.user, session.user);
  assert.deepEqual(auth.permissions, session.permissions);
  assert.deepEqual(calls, ["check_setup_status", "login_user"]);
  const denied = { code: "INVALID_CREDENTIALS", message: "Usuario o clave incorrectos." };
  mockIPC(() => { throw denied; });
  await assert.rejects(auth.loginUser(login), hasCode("INVALID_CREDENTIALS"));
  assert.deepEqual(auth.error, denied);
  assert.equal(auth.user?.id, 1);
  assert.equal(auth.isLoading, false);
  assert.ok(!JSON.stringify(auth.$state).includes(login.password));
});

test("temporary permission stays contextual and never alters permanent permissions", async () => {
  mockIPC((command, args) => {
    if (command === "login_user") return session;
    assert.equal(command, "request_temporary_access");
    assert.deepEqual(args, { payload: temporary });
    return grant;
  });
  const auth = useAuthStore();
  await auth.loginUser(login);
  await auth.requestTemporaryAccess(temporary);
  assert.deepEqual(auth.temporaryAccess, grant);
  assert.deepEqual(auth.permissions, session.permissions);
  assert.equal(auth.can("payments.add", "repairs", 1), true);
  assert.equal(auth.can("payments.add", "repairs", 2), false);
  assert.equal(auth.can("payments.add", "clients", 1), false);
  mockIPC((command) => { assert.equal(command, "finish_temporary_access"); throw { code: "STORAGE_ERROR", message: "Error de prueba" }; });
  await assert.rejects(auth.finishTemporaryAccess(), hasCode("STORAGE_ERROR"));
  assert.equal(auth.temporaryAccess, null);
  assert.equal(auth.user?.id, 1);
  assert.ok(!JSON.stringify(auth.$state).includes(temporary.password));
});

test("initial setup maps its full payload and requires no legacy first_setup command", async () => {
  const payload = { name: "Prueba", username: "admin", password: "test-account-only", confirmation: "test-account-only", masterPassword: "test-master-only", masterConfirmation: "test-master-only" };
  mockIPC((command, args) => {
    assert.equal(command, "initialize_setup");
    assert.deepEqual(args, { payload });
    return session;
  });
  const auth = useAuthStore();
  await auth.initializeSetup(payload);
  assert.equal(auth.setupComplete, true);
  assert.equal(auth.isAuthenticated, true);
});

test("recovery uses flat arguments, clears sessions and cached orders, and stores no replacement passwords", async () => {
  mockIPC((command, args) => {
    if (command === "login_user") return session;
    if (command === "create_order") return order;
    if (command === "generate_recovery_challenge") return { challenge: "test-challenge", adminId: 1, adminUsername: "admin", expiresInSeconds: 900 };
    assert.equal(command, "verify_recovery_signature");
    assert.deepEqual(args, recoveryInput);
    return { adminId: 1, requiresLogin: true };
  });
  const auth = useAuthStore();
  const orders = useOrderStore();
  const recovery = useRecoveryStore();
  await auth.loginUser(login);
  await orders.createOrder(input);
  const before = Date.now();
  await recovery.generateRecoveryChallenge();
  assert.ok(recovery.expiresAt! >= before + 900000);
  await recovery.verifyRecoverySignature(recoveryInput);
  assert.equal(recovery.result?.requiresLogin, true);
  assert.equal(recovery.challenge, null);
  assert.equal(recovery.expiresAt, null);
  assert.equal(auth.user, null);
  assert.deepEqual(auth.permissions, []);
  assert.equal(orders.currentOrder, null);
  assert.equal(recovery.isLoading, false);
  assert.ok(!JSON.stringify(recovery.$state).includes(recoveryInput.newMasterPassword));
});

test("order requests retain exact retry id and payload, and secrets are excluded from global state", async () => {
  let calls = 0;
  const snapshots: unknown[] = [];
  mockIPC((command, args) => {
    snapshots.push(structuredClone(args));
    if (command === "create_order") {
      calls += 1;
      if (calls === 1) throw { code: "RESULT_UNCERTAIN", message: "Confirmación incierta" };
      return order;
    }
    assert.equal(command, "get_order");
    assert.deepEqual(args, { payload: { orderId: 1, scope: "clients", includeDevicePassword: true } });
    return { ...order, devicePassword: "decrypted-test-only" };
  });
  const store = useOrderStore();
  const original = structuredClone(input);
  await assert.rejects(store.createOrder(input), hasCode("RESULT_UNCERTAIN"));
  assert.equal(store.isLoading, false);
  await store.createOrder(input);
  assert.deepEqual(input, original);
  assert.deepEqual(snapshots[0], snapshots[1]);
  const response = await store.getOrder({ orderId: 1, scope: "clients", includeDevicePassword: true });
  assert.equal(response.devicePassword, "decrypted-test-only");
  assert.equal(store.currentOrder?.devicePassword, undefined);
  assert.ok(!JSON.stringify(store.$state).includes("decrypted-test-only"));
  assert.ok(!JSON.stringify(store.$state).includes(input.devicePassword!));
});

test("unsafe and fractional integers fail before IPC; unsafe responses preserve an uncertain outcome", async () => {
  let calls = 0;
  mockIPC(() => { calls += 1; return { ...order, pendingMinor: Number.MAX_SAFE_INTEGER + 1 }; });
  const store = useOrderStore();
  for (const value of [1.5, NaN, Infinity, Number.MAX_SAFE_INTEGER + 1]) {
    await assert.rejects(store.createOrder({ ...input, priceMinor: value }), hasCode("INVALID_INTEGER"));
    assert.equal(store.isLoading, false);
  }
  assert.equal(calls, 0);
  await assert.rejects(store.createOrder(input), hasCode("IPC_ERROR"));
  assert.equal(calls, 1);
  assert.equal(store.currentOrder, null);
});

test("session changes cannot restore stale login or order responses", async () => {
  let resolveLogin!: (value: SessionResponse) => void;
  mockIPC(() => new Promise<SessionResponse>((done) => { resolveLogin = done; }));
  const auth = useAuthStore();
  const pendingLogin = auth.loginUser(login);
  auth.clearSession();
  resolveLogin(session);
  await assert.rejects(pendingLogin, hasCode("SESSION_CHANGED"));
  assert.equal(auth.user, null);
  let resolveOrder!: (value: OrderResponse) => void;
  mockIPC(() => new Promise<OrderResponse>((done) => { resolveOrder = done; }));
  const store = useOrderStore();
  const pendingOrder = store.createOrder(input);
  auth.clearSession();
  resolveOrder(order);
  await assert.rejects(pendingOrder, hasCode("SESSION_CHANGED"));
  assert.equal(store.currentOrder, null);
});

test("logout and uncertain recovery fail closed while Rust errors are normalized without leaking unknown errors", async () => {
  mockIPC((command) => command === "login_user" ? session : order);
  const auth = useAuthStore();
  const orders = useOrderStore();
  await auth.loginUser(login);
  await orders.createOrder(input);
  mockIPC(() => { throw new Error("sensitive transport internals"); });
  await assert.rejects(auth.logoutUser(), hasCode("IPC_ERROR"));
  assert.equal(auth.user, null);
  assert.equal(orders.currentOrder, null);
  assert.ok(!auth.error?.message.includes("sensitive"));
  mockIPC(() => session);
  await auth.loginUser(login);
  const recovery = useRecoveryStore();
  mockIPC(() => { throw JSON.stringify({ code: "RESULT_UNCERTAIN", message: "Confirmación incierta" }); });
  await assert.rejects(recovery.verifyRecoverySignature(recoveryInput), hasCode("RESULT_UNCERTAIN"));
  assert.equal(auth.user, null);
  assert.equal(recovery.error?.code, "RESULT_UNCERTAIN");
});

test("all serialized Rust AppError codes are represented in TypeScript", () => {
  const rust = readFileSync(new URL("../src-tauri/src/error.rs", import.meta.url), "utf8");
  const ts = readFileSync(new URL("../src/types/ipc.ts", import.meta.url), "utf8");
  const errorType = ts.split("export type AppErrorCode =")[1].split(";")[0];
  const rustCodes = [...rust.matchAll(/=> "([A-Z_]+)"/g)].map((match) => match[1]);
  const tsCodes = [...errorType.matchAll(/"([A-Z_]+)"/g)].map((match) => match[1]);
  assert.deepEqual([...rustCodes].sort(), [...tsCodes].sort());
  for (const code of rustCodes) assert.equal(normalizeIpcError({ code, message: "Prueba" }).code, code);
});
