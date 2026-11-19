import { afterEach, beforeEach, mock, test } from 'node:test';
import assert from 'node:assert/strict';
import { createSSRApp, effectScope, getCurrentInstance, markRaw, nextTick, ref } from 'vue';
import { renderToString } from 'vue/server-renderer';
import { createPinia, disposePinia, setActivePinia } from 'pinia';
import { useEvidenceCamera } from '../src/composables/useEvidenceCamera.ts';
import { useCompanyClauses } from '../src/composables/useCompanyClauses.ts';
import { defaultCompany } from '../src/composables/useCompany.ts';
import { useRepairScreen } from '../src/composables/useRepairScreen.ts';
import type { Order } from '../src/types/order.ts';

const originalGlobals = new Map(['document', 'navigator', 'HTMLElement'].map(name => [name, Object.getOwnPropertyDescriptor(globalThis, name)]));
const scopes: ReturnType<typeof effectScope>[] = [];
let focused = '';
let targets: Map<string, FocusTarget>;
class FocusTarget {
  disabled = false;
  id: string;
  constructor(id: string) { this.id = id; }
  focus() { focused = this.id; documentStub.activeElement = this; }
}
const documentStub = {
  activeElement: null as FocusTarget | null,
  getElementById(id: string) { return targets.get(id) ?? null; },
  createElement() {
    return { width: 0, height: 0, getContext: () => ({ drawImage() {} }),
      toBlob: (resolve: (value: Blob | null) => void) => resolve(new Blob(['foto'], { type: 'image/jpeg' })) };
  },
};
beforeEach(() => {
  focused = '';
  targets = new Map(['clause-add', 'clause-title', 'clause-text', 'clause-a', 'clause-b', 'clause-a-up'].map(id => [id, new FocusTarget(id)]));
  documentStub.activeElement = targets.get('clause-add')!;
  Object.defineProperty(globalThis, 'document', { configurable: true, value: documentStub });
  Object.defineProperty(globalThis, 'HTMLElement', { configurable: true, value: FocusTarget });
  Object.defineProperty(globalThis, 'navigator', { configurable: true, value: { mediaDevices: {} } });
});
afterEach(() => {
  for (const scope of scopes.splice(0)) scope.stop();
  mock.restoreAll();
  for (const [name, descriptor] of originalGlobals) {
    if (descriptor) Object.defineProperty(globalThis, name, descriptor);
    else Reflect.deleteProperty(globalThis, name);
  }
});

function camera() {
  const scope = effectScope();
  scopes.push(scope);
  const props = { busy: false };
  let closed = 0;
  const model = scope.run(() => useEvidenceCamera(props, () => { closed++; }))!;
  const dialog = markRaw({ open: false, showModal() { this.open = true; }, close() { this.open = false; } });
  model.dialog.value = dialog as HTMLDialogElement;
  model.video.value = markRaw({ srcObject: null, videoWidth: 640, videoHeight: 480, async play() {} }) as unknown as HTMLVideoElement;
  model.enableButton.value = new FocusTarget('enable-camera') as unknown as HTMLButtonElement;
  model.addButton.value = new FocusTarget('add-photo') as unknown as HTMLButtonElement;
  return { model, scope, props, dialog, closed: () => closed };
}

function stream() {
  let stopped = 0;
  const track = { stop() { stopped++; }, getSettings: () => ({ deviceId: 'camera-1' }) };
  return { value: { getTracks: () => [track], getVideoTracks: () => [track] } as unknown as MediaStream, stopped: () => stopped };
}

test('camera reports denied, missing and busy devices while preserving the open dialog', async () => {
  const { model, dialog } = camera();
  await model.open();
  assert.equal(focused, 'enable-camera');
  for (const [name, message] of [['NotAllowedError', 'No se permitió'], ['NotFoundError', 'No encontramos'], ['NotReadableError', 'ocupada']]) {
    Object.assign(navigator.mediaDevices, { getUserMedia: async () => { throw Object.assign(new Error(), { name }); } });
    await model.enable();
    assert.equal(model.phase.value, 'off');
    assert.ok(model.cameraError.value.includes(message!));
    assert.ok(dialog.open);
    assert.equal(model.photo.value, undefined);
  }
});

test('camera stops a permission response that arrives after closing', async () => {
  const { model, dialog, closed } = camera();
  const opened = stream();
  let release!: (value: MediaStream) => void;
  Object.assign(navigator.mediaDevices, { getUserMedia: () => new Promise<MediaStream>(resolve => { release = resolve; }) });
  await model.open();
  const pending = model.enable();
  model.close();
  release(opened.value);
  await pending;
  assert.equal(opened.stopped(), 1);
  assert.equal(model.video.value?.srcObject, null);
  assert.equal(dialog.open, false);
  assert.equal(closed(), 1);
});

test('capture preserves the pending file and description until discard or confirmed save', async () => {
  const { model, props, dialog, closed } = camera();
  const opened = stream();
  const revoked: string[] = [];
  mock.method(URL, 'createObjectURL', () => 'blob:pending-photo');
  mock.method(URL, 'revokeObjectURL', (value: string) => { revoked.push(value); });
  Object.assign(navigator.mediaDevices, {
    getUserMedia: async (constraints: MediaStreamConstraints) => { assert.equal(constraints.audio, false); return opened.value; },
    enumerateDevices: async () => [{ kind: 'videoinput', deviceId: 'camera-1' }, { kind: 'audioinput', deviceId: 'mic' }],
  });
  await model.open();
  await model.enable();
  assert.equal(model.phase.value, 'live');
  assert.equal(model.devices.value.length, 1);
  await model.take();
  assert.equal(model.phase.value, 'review');
  assert.equal(model.photo.value?.type, 'image/jpeg');
  assert.equal(opened.stopped(), 1);
  assert.equal(focused, 'add-photo');
  model.note.value = 'Descripción conservada';
  const pendingFile = model.photo.value;
  model.close();
  assert.ok(model.discard.value);
  assert.ok(dialog.open);
  props.busy = true;
  model.close(true);
  assert.equal(model.photo.value, pendingFile);
  assert.equal(model.note.value, 'Descripción conservada');
  assert.equal(closed(), 0);
  props.busy = false;
  model.saved();
  assert.deepEqual(revoked, ['blob:pending-photo']);
  assert.equal(model.photo.value, undefined);
  assert.equal(dialog.open, false);
  assert.equal(closed(), 1);
});

test('disposing a camera rejects a late capture and releases the active stream', async () => {
  const { model, scope, dialog } = camera();
  const opened = stream();
  Object.assign(navigator.mediaDevices, { getUserMedia: async () => opened.value, enumerateDevices: async () => [] });
  let release!: (value: Blob) => void;
  mock.method(documentStub, 'createElement', () => ({ width: 0, height: 0, getContext: () => ({ drawImage() {} }),
    toBlob: (resolve: (value: Blob) => void) => { release = resolve; } }));
  await model.open();
  await model.enable();
  const pending = model.take();
  scope.stop();
  release(new Blob(['captura'], { type: 'image/jpeg' }));
  await pending;
  assert.equal(opened.stopped(), 1);
  assert.equal(dialog.open, false);
  assert.equal(model.photo.value, undefined);
});

test('clause drafts validate before applying and restore keyboard focus', async () => {
  const scope = effectScope();
  scopes.push(scope);
  const settings = ref(defaultCompany());
  const editing: boolean[] = [];
  const model = scope.run(() => useCompanyClauses(settings, value => editing.push(value)))!;
  model.edit();
  await nextTick();
  assert.equal(focused, 'clause-title');
  model.apply();
  assert.equal(settings.value.clauses.length, 0);
  assert.ok(model.error.value);
  model.editor.value!.title = ' Condición ';
  model.editor.value!.text = ' Texto acordado ';
  model.apply();
  await nextTick();
  assert.equal(settings.value.clauses[0]?.title, 'Condición');
  assert.equal(settings.value.clauses[0]?.text, 'Texto acordado');
  assert.equal(focused, 'clause-add');
  assert.deepEqual(editing, [true, false]);
  model.edit(settings.value.clauses[0]);
  model.editor.value!.text = 'Borrador cancelado';
  model.cancel();
  assert.equal(settings.value.clauses[0]?.text, 'Texto acordado');
});

test('clause ordering and undo preserve identifiers, position and focused controls', async () => {
  const scope = effectScope();
  scopes.push(scope);
  const settings = ref(defaultCompany());
  settings.value.clauses = [
    { id: 'a', title: 'A', text: 'Primera', entry: true, exit: false },
    { id: 'b', title: 'B', text: 'Segunda', entry: false, exit: false },
  ];
  const model = scope.run(() => useCompanyClauses(settings, () => {}))!;
  assert.equal(model.activeCount.value, 1);
  model.move(0, 1);
  await nextTick();
  assert.deepEqual(settings.value.clauses.map(clause => clause.id), ['b', 'a']);
  assert.equal(focused, 'clause-a');
  model.move(1, 1);
  assert.deepEqual(settings.value.clauses.map(clause => clause.id), ['b', 'a']);
  model.remove(1);
  await nextTick();
  assert.equal(focused, 'clause-b');
  model.undo();
  model.undo();
  await nextTick();
  assert.deepEqual(settings.value.clauses.map(clause => clause.id), ['b', 'a']);
  assert.equal(focused, 'clause-a');
});

test('finances renders payment and delivery breakdown without audit controls', async () => {
  const { createServer } = await import('vite');
  const server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, appType: 'custom' });
  const pinia = createPinia();
  setActivePinia(pinia);
  try {
    const component = (await server.ssrLoadModule('/src/views/FinancesView.vue')).default;
    const { financialTotals } = await import('../src/models/finances.ts');
    const order = { id:'ORD-009',client:'Cliente',equipment:'Laptop',status:'Reparado',currency:'MXN',
      totalMinor:150000,collectedMinor:150000,balanceMinor:0,costsMinor:60000,profitMinor:90000,
      delivered:false,paid:true,classification:'paid_uncollected',excluded:false,
      received:'2026-11-14',deliveredAt:null,settledAt:null,payments:[],expenses:[] };
    const filters = {query:'',classification:'all',dateBasis:'received',from:'',to:'',scope:'active'};
    const prepared = { ...component, setup(props: unknown, context: unknown) {
      const model = component.setup(props, context);
      return {...model,totals:ref(financialTotals([order],filters,'America/Cancun')),
        filtered:ref([order]),visible:ref([order]),selected:ref(order)};
    } };
    const html = await renderToString(createSSRApp(prepared).use(pinia));
    for (const text of ['Finanzas', 'ORD-009', 'Ganancia estimada de trabajos pagados',
      'Ganancia estimada de trabajos con deuda', 'Desglose por pago y entrega',
      'Pagadas y entregadas', 'Pagadas sin entregar', 'Entregadas con saldo pendiente',
      'Sin liquidar ni entregar', '900.00', 'Detalle financiero']) {
      assert.ok(html.includes(text), text);
    }
    assert.doesNotMatch(html,/Historial de cambios|Historial de esta orden|Detalle del movimiento|Antes|Después|Bitácora/);
  } finally { disposePinia(pinia); await server.close(); }
});

test('repair warranty suggestions follow the selected company policy in the viewmodel', async () => {
  const pinia = createPinia();
  setActivePinia(pinia);
  try {
    await renderToString(createSSRApp({ setup() {
      const model = useRepairScreen({ order: { databaseId: 1 } as Order }, () => {});
      model.selectWarrantyType('labor');
      assert.equal(model.days.value, model.company.value.laborDays);
      assert.equal(model.warrantyType.value, 'labor');
      model.selectWarrantyType('parts');
      assert.equal(model.days.value, model.company.value.partsDays);
      assert.equal(model.warrantyType.value, 'parts');
      return () => null;
    } }).use(pinia));
  } finally { disposePinia(pinia); }
});

test('the moved container retains company drafts and blocks leaving during an operation', async () => {
  const { createServer } = await import('vite');
  const server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, appType: 'custom' });
  const pinia = createPinia();
  setActivePinia(pinia);
  try {
    const component = (await server.ssrLoadModule('/src/WorkshopApp.vue')).default;
    const prepared = { ...component, async setup(props: unknown, context: unknown) {
      const model = component.setup(props, context);
      const instance = getCurrentInstance()!;
      model.workspace.page = 'company';
      model.companyDirty.value = true;
      assert.equal(await model.prepareLeave(), false);
      await model.navigate('Usuarios');
      assert.equal(model.workspace.page, 'company');
      assert.match(model.notice.value, /Guarda o descarta/);
      model.companyDirty.value = false;
      model.printing.value = true;
      assert.equal(await model.prepareLeave(), false);
      model.printing.value = false;
      assert.equal(await model.prepareLeave(), true);
      let attempts = 0;
      instance.refs.repair = {async save() {attempts++; return false;}};
      model.workspace.page = 'repairs';
      assert.equal(await model.prepareLeave(), false);
      await model.navigate('Clientes');
      assert.equal(model.workspace.page, 'repairs');
      assert.equal(attempts, 2);
      return model;
    } };
    await renderToString(createSSRApp(prepared).use(pinia));
  } finally { disposePinia(pinia); await server.close(); }
});

test('company leave guard preserves invalid and rejected drafts and confirms save or discard', async () => {
  const { createServer } = await import('vite');
  const server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, appType: 'custom' });
  const pinia = createPinia(); setActivePinia(pinia);
  const previousWindow = Object.getOwnPropertyDescriptor(globalThis, 'window');
  Object.defineProperty(globalThis, 'window', {configurable: true, value: {addEventListener() {}, removeEventListener() {}}});
  try {
    const {useCompanyWorkspace} = await server.ssrLoadModule('/src/composables/useCompanyWorkspace.ts');
    const {useAuthStore} = await server.ssrLoadModule('/src/stores/useAuthStore.ts');
    const {useCompanyStore} = await server.ssrLoadModule('/src/stores/useCompanyStore.ts');
    const auth = useAuthStore(pinia), store = useCompanyStore(pinia);
    auth.user = {id: 1, name: 'Admin', accessLevel: 'complete'};
    auth.permissions = ['company.edit', 'company.view'];
    store.settings = defaultCompany(); store.loaded = true;
    let fail = true, saves = 0;
    store.save = async (value: ReturnType<typeof defaultCompany>) => {
      saves++;
      if (fail) throw new Error('Fallo de prueba');
      store.settings = {...value, revision: 2};
    };
    let model!: ReturnType<typeof useCompanyWorkspace>;
    await renderToString(createSSRApp({setup() {model = useCompanyWorkspace({orders: []}, () => {}); return () => null;}}).use(pinia));
    model.draft.value.name = 'Empresa pendiente';
    const kept = model.prepareLeave(); model.leaveGuard.keep();
    assert.equal(await kept, false); assert.equal(model.draft.value.name, 'Empresa pendiente');
    model.clauseEditing.value = true;
    assert.equal(await model.prepareLeave(), false); assert.match(model.message.value, /cláusula/);
    model.clauseEditing.value = false;
    model.readingLogo.value = true; assert.equal(await model.prepareLeave(), false); model.readingLogo.value = false;
    const saved = model.prepareLeave();
    model.draft.value.name = ''; await model.leaveGuard.save();
    assert.equal(saves, 0); assert.equal(model.leaveGuard.open.value, true);
    model.draft.value.name = 'Empresa pendiente'; await model.leaveGuard.save();
    assert.equal(saves, 1); assert.equal(model.dirty.value, true); assert.match(model.leaveGuard.error.value, /No se guardaron/);
    fail = false; await model.leaveGuard.save();
    assert.equal(await saved, true); assert.equal(model.dirty.value, false); assert.equal(store.settings.name, 'Empresa pendiente');
    model.draft.value.name = 'Cambio descartado';
    const discarded = model.prepareLeave(); model.leaveGuard.discard();
    assert.equal(await discarded, true); assert.equal(model.draft.value.name, 'Empresa pendiente');
  } finally {
    if (previousWindow) Object.defineProperty(globalThis, 'window', previousWindow); else Reflect.deleteProperty(globalThis, 'window');
    disposePinia(pinia); await server.close();
  }
});
