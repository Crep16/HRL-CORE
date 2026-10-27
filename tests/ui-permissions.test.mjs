import assert from 'node:assert/strict';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import vue from '@vitejs/plugin-vue';
import { createSSRApp, h } from 'vue';
import { renderToString } from 'vue/server-renderer';

test('header renders the active account and order actions reflect backend permissions', async () => {
  const previousWindow = globalThis.window;
  const previousTauri = globalThis.isTauri;
  let role = 'complete';
  const permissions = {
    complete: ['clients.view', 'orders.create', 'orders.edit', 'orders.delete', 'payments.add', 'repairs.view'],
    limited: ['clients.view', 'orders.create', 'repairs.view', 'notes.add'],
    custom: ['clients.view', 'orders.delete'],
  };
  globalThis.isTauri = true;
  globalThis.window = { __TAURI_INTERNALS__: { async invoke(command) {
    if (command === 'access_status') return { setupComplete: true, user: { id: 'fixture', name: '<Usuario de prueba>', accessLevel: role } };
    if (command === 'access_permissions') return permissions[role];
    throw new Error(`Unexpected command: ${command}`);
  } } };
  const server = await createServer({ root: fileURLToPath(new URL('..', import.meta.url)), configFile: false, plugins: [vue()], server: { middlewareMode: true, hmr: false }, appType: 'custom', logLevel: 'error' });
  try {
    const { useAccess } = await server.ssrLoadModule('/src/composables/useAccess.ts');
    const { default: Header } = await server.ssrLoadModule('/src/components/layout/AppHeader.vue');
    const { default: Sheet } = await server.ssrLoadModule('/src/components/orders/OrderSheet.vue');
    const { default: Workshop } = await server.ssrLoadModule('/src/WorkshopApp.vue');
    const order = { id: 'TEST-001', client: 'Cliente de prueba', phone: '', received: '', status: 'Diagnóstico', serial: '', item: 'Laptop', brand: '', model: '', otherShop: false, otherShopReason: '', dropped: false, dropReason: '', password: '', work: '', cost: '$0.00', deposit: '$0.00', notes: '', paid: false, delivered: false };
    const render = (component, props) => renderToString(createSSRApp({ render: () => h(component, props) }));
    const buttons = html => [...html.matchAll(/<button\b([^>]*)>([\s\S]*?)<\/button>/g)];
    for (const [level, label] of Object.entries({ complete: 'Acceso completo', limited: 'Acceso limitado', custom: 'Acceso personalizado' })) {
      role = level;
      await useAccess().refresh();
      const header = await render(Header, { tabs: ['Clientes', 'Empresa'], active: 'Clientes' });
      assert.ok(header.includes('&lt;Usuario de prueba&gt;'));
      assert.ok(header.includes(label));
      assert.ok(header.indexOf(label) < header.indexOf('Cerrar sesión'));
      const sheet = await render(Sheet, { order, position: 1, total: 1, editing: false });
      const remove = buttons(sheet).find(button => button[2].includes('Eliminar'));
      assert.ok(remove);
      assert.equal(/\bdisabled\b/.test(remove[1]), level === 'limited');
      const trash = await render(Sheet, { order, position: 1, total: 1, editing: false, trashed: true });
      assert.ok(buttons(trash).some(button => button[2].includes('Restaurar orden')));
      assert.ok(!buttons(trash).some(button => button[2].includes('Eliminar') || /\bEditar\b/.test(button[2])));
      if (level === 'limited') {
        const workshop = await render(Workshop, {});
        assert.ok(buttons(workshop).some(button => button[2].includes('Reparaciones')), 'Limited access must have the Repairs menu');
      }
    }
  } finally {
    await server.close();
    if (previousWindow === undefined) delete globalThis.window; else globalThis.window = previousWindow;
    if (previousTauri === undefined) delete globalThis.isTauri; else globalThis.isTauri = previousTauri;
  }
});
