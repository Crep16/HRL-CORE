import { after, afterEach, before, beforeEach, test } from 'node:test';
import assert from 'node:assert/strict';
import { createSSRApp } from 'vue';
import { renderToString } from 'vue/server-renderer';
import { createPinia, disposePinia, setActivePinia } from 'pinia';
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { createServer, type ViteDevServer } from 'vite';
import type { Order } from '../src/types/order.ts';

globalThis.window = {} as Window & typeof globalThis;
let server: ViteDevServer;
let pinia: ReturnType<typeof createPinia>;
before(async () => {
  server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, appType: 'custom' });
});
after(async () => { await server.close(); });
beforeEach(() => { pinia = createPinia(); setActivePinia(pinia); });
afterEach(() => { clearMocks(); disposePinia(pinia); });

const order: Order = {
  id: 'ORD-001', databaseId: 1, client: 'Cliente de prueba', phone: '0000000000',
  received: '2026-11-13', status: 'Reparado', serial: 'SERIE', item: 'Laptop',
  brand: 'Marca', model: 'Modelo', otherShop: false, otherShopReason: '', dropped: false,
  dropReason: '', password: 'secreto-que-no-debe-imprimirse', work: 'Trabajo autorizado',
  cost: '600.00', deposit: '750.00', notes: 'Notas de recepción', paid: true, delivered: false,
};

async function renderNote(kind: 'entry' | 'exit') {
  const component = (await server.ssrLoadModule('/src/components/company/CompanyNote.vue')).default;
  const { defaultCompany } = await server.ssrLoadModule('/src/composables/useCompany.ts');
  const settings = defaultCompany();
  settings.name = 'Taller de prueba';
  settings.logoPrint = false;
  settings.clauses = [
    { id: 'entrada', title: 'Condición de entrada', text: 'Texto de entrada', entry: true, exit: false },
    { id: 'salida', title: 'Condición de salida', text: 'Texto de salida', entry: false, exit: true },
  ];
  const prepared = {
    ...component,
    async setup(props: unknown, context: unknown) {
      const model = await component.setup(props, context);
      await model.prepare();
      return model;
    },
  };
  return renderToString(createSSRApp(prepared, { settings, kind, order }).use(pinia));
}

test('exit note renders the public summary and warranty from delivery without printing secrets', async () => {
  mockIPC(command => {
    assert.equal(command, 'normalized_repair_summary');
    return {
      technician: 'Técnico de prueba', publicNote: 'Resumen público de reparación',
      warranties: [{ name: 'Pantalla', days: 90 }], internalNote: 'nota-interna-confidencial',
    };
  });
  const html = await renderNote('exit');
  for (const text of ['Resumen público de reparación', 'Pantalla', '90 días a partir de la entrega', 'Condición de salida', '600.00']) {
    assert.ok(html.includes(text), text);
  }
  for (const text of ['nota-interna-confidencial', order.password, 'Condición de entrada', 'Notas de recepción']) {
    assert.ok(!html.includes(text), text);
  }
});

test('entry note contains its own clauses and reception notes', async () => {
  mockIPC(() => ({ technician: 'Técnico', publicNote: 'Resumen de salida', warranties: [] }));
  const html = await renderNote('entry');
  assert.ok(html.includes('Condición de entrada'));
  assert.ok(html.includes('Notas de recepción'));
  assert.ok(!html.includes('Condición de salida'));
  assert.ok(!html.includes('Resumen de salida'));
  assert.ok(!html.includes(order.password));
});

test('preparing a note rejects an unavailable or forbidden repair summary', async () => {
  mockIPC(() => { throw { code: 'FORBIDDEN', message: 'Acceso denegado de prueba' }; });
  await assert.rejects(renderNote('exit'), error => {
    return typeof error === 'object' && error !== null && 'code' in error && error.code === 'FORBIDDEN';
  });
});
