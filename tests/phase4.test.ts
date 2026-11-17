import { test, beforeEach, afterEach } from "node:test";
import assert from "node:assert/strict";
import { createSSRApp, effectScope, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";
import { createPinia, disposePinia, setActivePinia } from "pinia";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { parseMinorUnits, formatMinorUnits } from "../src/utils/minorUnits.ts";
import { createReceptionForm } from "../src/composables/useReceptionForm.ts";
import { useDraftLeaveGuard } from '../src/composables/useDraftLeaveGuard.ts';
import { useAccessForm } from "../src/composables/useAccessForm.ts";
import { useWindowProfile } from "../src/composables/useWindowProfile.ts";
import { useTemporaryAccessForm } from "../src/composables/useTemporaryAccessForm.ts";
import { useAuthStore } from "../src/stores/useAuthStore.ts";
import { useOrderStore } from "../src/stores/useOrderStore.ts";
import { equipmentHistoryText } from '../src/utils/equipmentHistoryText.ts';
import type { CreateOrderPayload, OrderResponse, SessionResponse } from "../src/types/ipc.ts";

globalThis.window = { addEventListener() {}, removeEventListener() {} } as unknown as Window & typeof globalThis;
let pinia: ReturnType<typeof createPinia>;
beforeEach(() => { pinia = createPinia(); setActivePinia(pinia); });
afterEach(() => { clearMocks(); disposePinia(pinia); });
const session: SessionResponse = { user: { id: 1, name: "Prueba", loginName: "admin", role: "administrator", accessLevel: "complete", active: true, createdAt: "2026-11-04" }, permissions: ["orders.create", "clients.view", "payments.add"] };
async function mountModel<T>(factory: () => T): Promise<T> {
  let model!: T;
  await renderToString(createSSRApp({ setup() { model = factory(); return () => h("div"); } }).use(pinia));
  return model;
}
function response(payload: CreateOrderPayload): OrderResponse {
  return { id: 1, folio: "ORD-001", clientId: 1, equipmentId: 1, receivedBy: 1, status: payload.status, currency: payload.currency, receivedAt: payload.receivedAt, estimatedDeliveryAt: null, reportedFailure: payload.reportedFailure, plannedWork: payload.plannedWork, receptionNotes: payload.receptionNotes, priceMinor: payload.priceMinor, netMinor: payload.initialPayment?.amountMinor ?? 0, pendingMinor: payload.priceMinor - (payload.initialPayment?.amountMinor ?? 0), creditMinor: 0, hasDevicePassword: !!payload.devicePassword, client: { ...payload.client, id: 1 }, equipment: { ...payload.equipment, id: 1 }, history: payload.history, revision: 1, delivered: false, deliveredAt: null };
}

test("decimal strings round-trip exactly at cent and safe-integer boundaries", () => {
  for (const [text, amount] of [["150.50", 15050], ["0.29", 29], ["0,01", 1], ["1.2", 120], ["", 0], ["90071992547409.91", Number.MAX_SAFE_INTEGER]] as const) {
    assert.equal(parseMinorUnits(text), amount);
    assert.equal(parseMinorUnits(formatMinorUnits(amount)), amount);
  }
  for (const text of ["1.001", "1,000.00", "-1", "NaN", "Infinity", "1e3", "90071992547409.92"]) assert.throws(() => parseMinorUnits(text));
  assert.equal(formatMinorUnits(-101), "-1.01");
});

test('free-text history preserves previous details and supports explicitly clearing them', () => {
  const history = {previousWorkshop:true,previousWorkshopDetail:'Cambió teclado',sufferedImpact:true,impactDetail:'Golpe en esquina',liquidSpill:true,liquidDetail:'Agua'};
  assert.equal(equipmentHistoryText(history), 'Otro taller: Cambió teclado\nCaídas o golpes: Golpe en esquina\nLíquidos o humedad: Agua');
  assert.equal(equipmentHistoryText({...history,notes:'Reparación previa; se recibe húmedo.'}), 'Reparación previa; se recibe húmedo.');
  assert.equal(equipmentHistoryText({...history,notes:''}), '');
});

test("native window synchronization queues session changes and exposes retry without clearing authentication", async () => {
  Reflect.set(globalThis, "isTauri", true);
  const scope = effectScope();
  const auth = useAuthStore();
  auth.setupComplete = true;
  let release!: () => void;
  let calls = 0;
  mockIPC(async (command) => {
    assert.equal(command, "sync_normalized_window_profile");
    calls += 1;
    if (calls === 1) await new Promise<void>((resolve) => { release = resolve; });
    if (calls === 2) throw { code: "WINDOW_ERROR", message: "Fallo nativo de prueba" };
    return null;
  });
  try {
    const model = scope.run(useWindowProfile)!;
    await Promise.resolve();
    assert.equal(calls, 1);
    auth.user = session.user;
    await nextTick();
    assert.equal(calls, 1);
    release();
    await model.synchronize();
    assert.equal(calls, 3);
    assert.equal(model.error.value, null);
    assert.equal(auth.user?.id, session.user.id);
    assert.equal(model.isLoading.value, false);
  } finally {
    scope.stop();
    Reflect.deleteProperty(globalThis, "isTauri");
  }
});

test("reception sends integer centavos and preserves the exact financial retry including the password", async () => {
  const snapshots: CreateOrderPayload[] = [];
  let fail = true;
  mockIPC((command, args) => {
    if (command === "login_user") return session;
    if (command === "finish_temporary_access") return null;
    assert.equal(command, "create_order");
    const input = (args as { payload: CreateOrderPayload }).payload;
    snapshots.push(structuredClone(input));
    if (fail) { fail = false; throw { code: "RESULT_UNCERTAIN", message: "Prueba incierta" }; }
    return response(input);
  });
  await useAuthStore().loginUser({ username: "admin", password: "test-only" });
  const form = await mountModel(createReceptionForm);
  form.begin();
  form.draft.client.fullName = "Cliente de prueba"; form.draft.client.phone = "0000000000";
  form.draft.reportedFailure = "No enciende"; form.draft.devicePassword = "device-test-only";
  form.draft.history.liquidSpill = true;
  form.price.value = "150.50"; form.deposit.value = "20,05"; form.method.value = "Método libre";
  await form.submit();
  assert.equal(snapshots[0].priceMinor, 15050);
  assert.equal(snapshots[0].initialPayment?.amountMinor, 2005);
  assert.equal(snapshots[0].history.liquidSpill, true);
  assert.equal(form.uncertain.value, true);
  assert.equal(form.canEdit.value, false);
  assert.equal(await form.prepareLeave(), false);
  form.cancel();
  assert.equal(form.uncertain.value, true);
  assert.equal(form.canRetry.value, true);
  await form.submit();
  assert.deepEqual(snapshots[0], snapshots[1]);
  assert.equal(form.editing.value, false);
  assert.equal(form.draft.devicePassword, "");
  assert.equal(useOrderStore().sessionOrders.length, 1);
  assert.equal(await form.prepareLeave(), true);
});

test("new receipt and payment inputs follow independent presentation permissions", async () => {
  mockIPC(() => ({ ...session, permissions: ["orders.create", "clients.view"] }));
  await useAuthStore().loginUser({ username: "limited", password: "test-only" });
  const form = await mountModel(createReceptionForm);
  form.begin(); assert.equal(form.canEdit.value, true); assert.equal(form.canPay.value, false);
  form.draft.client.fullName = "Captura pendiente";
  form.cancel(); assert.equal(form.discardOpen.value, true);
  form.discard(); assert.equal(form.editing.value, false);
  useAuthStore().permissions = [];
  form.begin(); assert.equal(form.editing.value, false);
});

test('draft leave guard preserves failed saves, prevents concurrent actions and cancels on disposal', async () => {
  const scope = effectScope();
  let release!: (saved: boolean) => void;
  let saves = 0, discards = 0;
  const guard = scope.run(() => useDraftLeaveGuard({
    save: () => { saves++; return new Promise<boolean>(resolve => { release = resolve; }); },
    discard: () => { discards++; },
  }))!;
  const leaving = guard.request();
  assert.equal(guard.request(), leaving);
  const saving = guard.save();
  await guard.save(); guard.discard(); guard.keep();
  assert.equal(saves, 1); assert.equal(discards, 0); assert.equal(guard.open.value, true);
  release(false); await saving;
  assert.match(guard.error.value, /No se guardaron/);
  guard.keep(); assert.equal(await leaving, false);
  const retry = guard.request();
  const retriedSave = guard.save(); release(true); await retriedSave;
  assert.equal(await retry, true);
  const discarded = guard.request(); guard.discard();
  assert.equal(await discarded, true); assert.equal(discards, 1);
  const disposed = guard.request(); scope.stop();
  assert.equal(await disposed, false);
});

test('reception navigation offers keeping, discarding and saving without discarding an uncertain receipt', async () => {
  let uncertain = false, calls = 0;
  mockIPC((command, args) => {
    if (command === 'login_user') return session;
    if (command === 'list_reception_orders') return {items: [], total: 0};
    assert.equal(command, 'create_order'); calls++;
    if (uncertain) throw {code: 'RESULT_UNCERTAIN', message: 'Confirma el resultado.'};
    return response((args as {payload: CreateOrderPayload}).payload);
  });
  await useAuthStore().loginUser({username: 'admin', password: 'test-only'});
  const form = await mountModel(createReceptionForm);
  form.begin(); form.draft.client.fullName = 'Borrador conservado';
  const keep = form.prepareLeave(); form.leaveGuard.keep();
  assert.equal(await keep, false); assert.equal(form.draft.client.fullName, 'Borrador conservado');
  const discard = form.prepareLeave(); form.leaveGuard.discard();
  assert.equal(await discard, true); assert.equal(form.editing.value, false); assert.equal(calls, 0);
  form.begin(); form.draft.client.fullName = 'Orden guardada';
  const save = form.prepareLeave(); await form.leaveGuard.save();
  assert.equal(await save, true); assert.equal(form.editing.value, false); assert.equal(calls, 1);
  form.begin(); form.draft.client.fullName = 'Cobro por confirmar';
  form.deposit.value = '1.00'; form.method.value = 'Efectivo'; uncertain = true;
  const blocked = form.prepareLeave(); await form.leaveGuard.save(); form.leaveGuard.discard();
  assert.equal(form.uncertain.value, true); assert.equal(form.leaveGuard.open.value, true);
  assert.match(form.leaveGuard.error.value, /Confirma el resultado/);
  assert.equal(form.draft.client.fullName, 'Cobro por confirmar');
  form.leaveGuard.keep(); assert.equal(await blocked, false);
  assert.equal(await form.prepareLeave(), false);
});

test('login loads the active-name picker without creating a session and submits the selected name',async()=>{
  const calls:string[]=[];
  mockIPC((command,args)=>{
    calls.push(command);
    if(command==='check_setup_status')return {setupComplete:true};
    if(command==='list_login_users')return ['admin','ana','lulu'];
    assert.equal(command,'login_user');
    assert.deepEqual(args,{payload:{username:'lulu',password:'clave-de-prueba'}});
    return {...session,user:{...session.user,name:'lulu',loginName:'lulu'}};
  });
  const form=await mountModel(useAccessForm);
  await form.check();
  assert.deepEqual(form.loginUsers.value,['admin','ana','lulu']);
  assert.equal(form.auth.user,null);
  assert.equal(form.usersLoading.value,false);
  form.name.value='lulu';form.password.value='clave-de-prueba';
  await form.submit();
  assert.equal(form.auth.user?.loginName,'lulu');
  assert.equal(form.password.value,'');
  assert.deepEqual(calls,['check_setup_status','list_login_users','login_user']);
});

test('a failed user list keeps manual login available and can be reloaded',async()=>{
  let fail=true;
  mockIPC(command=>{
    if(command==='check_setup_status')return {setupComplete:true};
    assert.equal(command,'list_login_users');
    if(fail)throw {code:'STORAGE_ERROR',message:'Error de prueba'};
    return ['admin'];
  });
  const form=await mountModel(useAccessForm);
  await form.check();
  assert.match(form.usersError.value,/Puedes escribir tu usuario/);
  assert.equal(form.busy.value,false);
  assert.equal(form.auth.user,null);
  fail=false;await form.loadUsers();
  assert.deepEqual(form.loginUsers.value,['admin']);
  assert.equal(form.usersError.value,'');
});

test("setup is atomic after the two catalog steps, failed login clears personal secrets", async () => {
  let setupPayload: unknown;
  mockIPC((command, args) => {
    if (command === "check_setup_status") return { setupComplete: false };
    if (command === "initialize_setup") { setupPayload = args; return session; }
    throw { code: "INVALID_CREDENTIALS", message: "Prueba inválida" };
  });
  const form = await mountModel(useAccessForm);
  await form.check();
  form.password.value = "master-test-only"; form.confirmation.value = "master-test-only";
  await form.submit(); assert.equal(form.view.value, "first"); assert.equal(form.password.value, "");
  form.name.value = "admin"; form.password.value = "account-test-only"; form.confirmation.value = "account-test-only";
  await form.submit();
  assert.deepEqual(setupPayload, { payload: { name: "admin", username: "admin", password: "account-test-only", confirmation: "account-test-only", masterPassword: "master-test-only", masterConfirmation: "master-test-only" } });
  assert.equal(form.hasSecrets.value, false);
  form.password.value = "wrong-test-only";
  await form.submit(); assert.equal(form.password.value, ""); assert.equal(form.auth.error?.code, "INVALID_CREDENTIALS");
});

test("recovery and temporary forms do not retain credentials after failed submissions", async () => {
  mockIPC((command) => {
    if (command === "generate_recovery_challenge") return { challenge: "test-challenge", adminId: 1, adminUsername: "admin", expiresInSeconds: 900 };
    throw { code: command === "verify_recovery_signature" ? "RECOVERY_REJECTED" : "INVALID_CREDENTIALS", message: "Prueba rechazada" };
  });
  const access = await mountModel(useAccessForm);
  access.toggleSupport(); await access.generateChallenge();
  access.signature.value = "00".repeat(64); access.newMasterPassword.value = "master-test-only"; access.newAdminPassword.value = "admin-test-only";
  await access.verifySignature(); assert.equal(access.hasSecrets.value, false); assert.equal(access.signature.value, "");
  const temporary = await mountModel(useTemporaryAccessForm);
  temporary.username.value = "approver"; temporary.password.value = "wrong-test-only"; temporary.reason.value = "Prueba";
  assert.equal(await temporary.approve({ scope: "clients", orderId: null, operation: "orders.create" }), false);
  assert.equal(temporary.password.value, ""); assert.equal(temporary.auth.isLoading, false);
});

test("Vue renders the catalog fields with native limits and readonly saved orders", async () => {
  const { createServer } = await import("vite");
  const server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, appType: "custom" });
  try {
    const authModule = await server.ssrLoadModule("/src/stores/useAuthStore.ts");
    const ordersModule = await server.ssrLoadModule("/src/stores/useOrderStore.ts");
    const accessModule = await server.ssrLoadModule("/src/views/AccessView.vue");
    const receptionModule = await server.ssrLoadModule("/src/views/ReceptionView.vue");
    const auth = authModule.useAuthStore(pinia);
    auth.setupComplete = false;
    let html = await renderToString(createSSRApp(accessModule.default).use(pinia));
    assert.match(html, /Define la clave maestra/);
    assert.match(html, /id="access-password"[^>]*maxlength="1024"/);
    assert.match(html, /class="access-shell"/);
    assert.match(html, /id="access-confirm"[^>]*type="password"/);
    assert.doesNotMatch(html, />Mostrar<|>Ocultar</);
    auth.setupComplete = true;
    html = await renderToString(createSSRApp(accessModule.default).use(pinia));
    assert.match(html, /id="access-name"[^>]*maxlength="100"/);
    assert.match(html, /Solicitar cambio de clave maestra con soporte/);
    assert.doesNotMatch(html, />Mostrar<|>Ocultar</);
    const appModule = await server.ssrLoadModule("/src/App.vue");
    const appHtml = await renderToString(createSSRApp(appModule.default).use(pinia));
    assert.match(appHtml, /Solicitar cambio de clave maestra con soporte/);
    assert.doesNotMatch(appHtml, /Espacios de trabajo|Taller actual|Recepción nueva/);
    auth.user = session.user; auth.permissions = session.permissions;
    const payload: CreateOrderPayload = {
      client: { id: null, fullName: "Cliente de prueba", phone: "0000000000", email: null },
      equipment: { id: null, item: "Laptop", brand: "", model: "", serial: "" },
      history: { previousWorkshop: false, previousWorkshopDetail: "", sufferedImpact: false, impactDetail: "", liquidSpill: false, liquidDetail: "" },
      folioPrefix: "ORD-", sequenceStart: 1, status: "En diagnóstico", currency: "MXN", receivedAt: "2026-11-04", estimatedDeliveryAt: null,
      reportedFailure: "Falla de prueba", plannedWork: "", receptionNotes: "", priceMinor: 15050, devicePassword: null, initialPayment: null,
    };
    const orders = ordersModule.useOrderStore(pinia);
    orders.currentOrder = response(payload);
    orders.orders = [{ id:99,folio:"ORD-PERSISTENTE",clientName:"Cliente del listado persistente",serial:"SERIE-PERSISTENTE",item:"Laptop",status:"En diagnóstico",receivedAt:"2026-11-05",currency:"MXN",priceMinor:15050,netMinor:0,pendingMinor:15050,creditMinor:0,delivered:false }];
    orders.total = 9;
    html = await renderToString(createSSRApp(receptionModule.default).use(pinia));
    assert.match(html, /Cliente del listado persistente/);
    assert.match(html, /ORD-PERSISTENTE/);
    assert.match(html, /9 en total/);
    assert.match(html, /id="lookup-order"/);
    assert.match(html, /aria-label="Avanzar entre órdenes"/);
    assert.match(html, /aria-label="Buscar en"/);
    assert.match(html, /Cambiar orden/);
    assert.match(html, />admin</);
    assert.doesNotMatch(html, /id="temporary-operation"/);
    assert.doesNotMatch(html, /abiertas en esta sesión/);
    assert.doesNotMatch(html, /Historial de la orden/);
    auth.permissions.push("logbook.view");
    orders.history = [{ id:1,authorId:1,authorName:"Operador de prueba",operation:"payments.add",entity:"abonos",entityId:"1",reason:null,result:"success",createdAt:"2026-11-05T12:00:00Z",changes:[{ field:"amountMinor",oldValue:null,newValue:"2005" }] }];
    orders.historyTotal = 1;
    html = await renderToString(createSSRApp(appModule.default).use(pinia));
    assert.match(html, /Historial de la orden/);
    assert.match(html, /Detalle del movimiento/);
    assert.match(html, /Antes/);
    assert.match(html, /Después/);
    assert.match(html, /20.05 MXN/);
    assert.match(html, /id="falla"[^>]*readonly[^>]*maxlength="16000"/);
    assert.match(html, /id="cliente"[^>]*readonly[^>]*maxlength="100"/);
    assert.match(html, /value="\$150.50"/);
    assert.match(html, /class="app normalized-workshop"/);
    assert.match(html, /class="work reception"/);
    assert.match(html, /class="sheet"/);
    assert.match(html, /class="form"/);
    assert.match(html, /class="side"/);
    assert.match(html, /id="equipment-history"[^>]*readonly[^>]*maxlength="16000"/);
    assert.match(html, /Antecedentes del equipo/);
    assert.doesNotMatch(html, /¿Ha sufrido caída o humedad\?/);
    assert.match(html, /class="repa"/);
    assert.match(html, /class="header-tools"/);
    assert.match(html, /class="reception-details"><summary>Falla reportada/);
    assert.doesNotMatch(html, /type="submit"[^>]*>Editar/);
  } finally { await server.close(); }
});

test("order arrows cross pages and the history note retains a guarded draft", async () => {
  const auth = useAuthStore(); auth.user = session.user; auth.permissions = session.permissions;
  const form = await mountModel(createReceptionForm);
  const orders = Array.from({length:7}, (_,index) => ({...response(form.draft),id:index+1,folio:`ORD-${index+1}`}));
  const offsets:number[] = [];
  mockIPC((command,args) => {
    if(command === 'finish_temporary_access') return null;
    if(command === 'list_reception_orders') {
      const payload = (args as {payload:{offset:number;limit:number}}).payload;
      offsets.push(payload.offset);
      return {total:orders.length,items:orders.slice(payload.offset,payload.offset+payload.limit).map(order=>({id:order.id,folio:order.folio,clientName:'Cliente',serial:'',item:'Laptop',status:order.status,receivedAt:order.receivedAt,currency:order.currency,priceMinor:0,netMinor:0,pendingMinor:0,creditMinor:0,delivered:false}))};
    }
    if(command === 'get_order') return orders.find(order=>order.id === (args as {payload:{orderId:number}}).payload.orderId);
    throw new Error(command);
  });
  await form.refreshList();
  assert.equal(form.orderPosition.value,1);
  await form.goToOrder(6);
  assert.equal(useOrderStore().currentOrder?.id,7);
  assert.equal(form.orderPosition.value,7);
  await form.goToOrder(0);
  assert.equal(useOrderStore().currentOrder?.id,1);
  assert.deepEqual(offsets,[0,5,0]);
  form.begin();
  assert.equal(form.draft.history.sufferedImpact,false);
  assert.equal(form.draft.history.liquidSpill,false);
  form.historyNotes.value='Se mojó; ya había sido revisado en otro taller.';
  assert.equal(form.draft.history.notes,'Se mojó; ya había sido revisado en otro taller.');
  assert.equal(form.draft.history.sufferedImpact,false);
  assert.equal(form.hasChanges.value,true);
  const selected=useOrderStore().currentOrder?.id;
  form.draft.client.fullName='Borrador';
  await form.goToOrder(6);
  assert.equal(useOrderStore().currentOrder?.id,selected);
});

test("restored workspace renders every original section through the normalized container", async () => {
  const { createServer } = await import('vite');
  const server=await createServer({server:{middlewareMode:true,hmr:false,ws:false},appType:'custom'});
  mockIPC(command=>{if(command==='normalized_repair_summary')return {technician:'',publicNote:'',warranties:[]};return null;});
  try {
    const authModule=await server.ssrLoadModule('/src/stores/useAuthStore.ts');
    const workspaceModule=await server.ssrLoadModule('/src/stores/useWorkspaceStore.ts');
    const companyModule=await server.ssrLoadModule('/src/stores/useCompanyStore.ts');
    const settingsModule=await server.ssrLoadModule('/src/composables/useCompany.ts');
    const ordersModule=await server.ssrLoadModule('/src/stores/useOrderStore.ts');
    const appModule=await server.ssrLoadModule('/src/WorkshopApp.vue');
    const auth=authModule.useAuthStore(pinia),workspace=workspaceModule.useWorkspaceStore(pinia),company=companyModule.useCompanyStore(pinia),orders=ordersModule.useOrderStore(pinia);
    auth.user=session.user;auth.permissions=['clients.view','orders.create','orders.edit','orders.delete','repairs.view','repairs.edit','notes.add','stages.advance','parts.add','warranties.add','publication.edit','company.view','company.edit','users.manage','logbook.view'];
    company.settings=settingsModule.defaultCompany();company.loaded=true;
    orders.currentOrder=response({client:{id:null,fullName:'Cliente de prueba',phone:'0000000000',email:null},equipment:{id:null,item:'Laptop',brand:'Marca',model:'Modelo',serial:'PRUEBA'},history:{previousWorkshop:false,previousWorkshopDetail:'',sufferedImpact:false,impactDetail:'',liquidSpill:false,liquidDetail:''},folioPrefix:'ORD-',sequenceStart:1,status:'En diagnóstico',currency:'MXN',receivedAt:'2026-11-05',estimatedDeliveryAt:null,reportedFailure:'No enciende',plannedWork:'Diagnóstico',receptionNotes:'',priceMinor:15050,devicePassword:null,initialPayment:null});
    for(const [page,checks] of [
      ['clients',['Nueva orden','Papelera','>Editar<','Abrir gestión','Evidencias']],
      ['repairs',['Notas del técnico','Línea de tiempo','Garantías','Refacciones','Publicación en la web','Movimientos de dinero']],
      ['company',['Datos generales','Notas impresas','Cláusulas','Folios y garantías','Así sale en la nota']],
      ['users',['Cuentas','Nivel de acceso','Clave maestra','Registrar abonos']],
      ['logbook',['Historial de cambios','Resumen financiero de órdenes','Detalle del movimiento']],
    ] as const){
      workspace.page=page;
      const html=await renderToString(createSSRApp(appModule.default).use(pinia));
      for(const text of checks)assert.ok(html.includes(text),`${page}: falta ${text}`);
      assert.match(html,/aria-label="Secciones"/);
      assert.doesNotMatch(html,/Espacios de trabajo|Taller actual|Recepción nueva/);
    }
  }finally{await server.close();}
});
