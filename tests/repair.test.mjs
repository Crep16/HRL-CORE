import assert from 'node:assert/strict';
import { after, test } from 'node:test';
import { restoreRepair, useRepair } from '../src/composables/useRepair.ts';

const previousStorage = Object.getOwnPropertyDescriptor(globalThis, 'localStorage');
after(() => {
  if (previousStorage) Object.defineProperty(globalThis, 'localStorage', previousStorage);
  else delete globalThis.localStorage;
});

function storage(raw = null) {
  const values = new Map(raw === null ? [] : [['hrl.repair.v1.A', raw]]);
  const state = { values, writes: 0, failRead: false, failWrite: false };
  Object.defineProperty(globalThis, 'localStorage', { configurable: true, value: {
    getItem(key) { if (state.failRead) throw new Error('Unavailable'); return values.get(key) ?? null; },
    setItem(key, value) { if (state.failWrite) throw new Error('Full'); state.writes++; values.set(key, value); },
  } });
  return state;
}

test('repair restores partial legacy records and retains all tracking data on reopen', () => {
  storage(JSON.stringify({ technician: 'Ana', publicNote: 'Diagnóstico' }));
  const { repair, save } = useRepair('A');
  assert.equal(repair.value.technician, 'Ana');
  assert.deepEqual(repair.value.payments, []);
  Object.assign(repair.value, {
    completed: 2, internalNote: 'Solo taller', syncMode: 'auto',
    events: [{ id: 1, date: '2026-10-27T09:00', title: 'Revisión', detail: '', stage: 'Diagnóstico', public: false }],
    warranties: [{ name: 'Pantalla', days: 90 }], parts: [{ name: 'Panel', price: 500 }],
    payments: [{ date: '2026-10-27T09:00', amount: 100 }],
  });
  assert.equal(save(), true);
  assert.deepEqual(useRepair('A').repair.value, repair.value);
  assert.deepEqual(useRepair('B').repair.value.events, []);
});

test('corrupt repair records never overwrite their original storage', () => {
  for (const raw of ['', '{', 'null', '[]', '42', '{"events":null}', '{"payments":[{"amount":"100"}]}', '{"completed":8}']) {
    const state = storage(raw);
    const { repair, save, message } = useRepair('A');
    assert.match(message.value, /No se pudo recuperar/);
    repair.value.publicNote = 'No sobrescribir';
    assert.equal(save(), false);
    assert.equal(save(), false);
    assert.equal(state.writes, 0);
    assert.equal(state.values.get('hrl.repair.v1.A'), raw);
  }
});

test('read failure blocks saves for the lifetime of the repair session', () => {
  const state = storage('{"publicNote":"Original"}');
  state.failRead = true;
  const session = useRepair('A');
  state.failRead = false;
  assert.equal(session.save(), false);
  assert.equal(state.writes, 0);
  assert.equal(useRepair('A').repair.value.publicNote, 'Original');
});

test('failed writes retain the draft and allow retry without affecting another order', () => {
  const state = storage();
  const session = useRepair('A');
  session.repair.value.internalNote = 'Borrador';
  state.failWrite = true;
  assert.equal(session.save(), false);
  assert.equal(session.repair.value.internalNote, 'Borrador');
  assert.equal(state.writes, 0);
  state.failWrite = false;
  assert.equal(session.save(), true);
  assert.equal(useRepair('A').repair.value.internalNote, 'Borrador');
  assert.equal(useRepair('B').repair.value.internalNote, '');
});

test('invalid tracking fields and nonfinite amounts are rejected before storage', () => {
  for (const value of [
    { completed: -1 }, { completed: 1.5 }, { technician: null }, { syncMode: 'unknown' },
    { events: [null] }, { warranties: [{ name: 'Panel', days: -1 }] },
    { parts: [{ name: 'Panel', price: '500' }] }, { visibility: [{ label: 'Notas', enabled: 'true' }] },
    { payments: [{ date: '', amount: 0 }] },
  ]) assert.throws(() => restoreRepair(JSON.stringify(value)));
  const state = storage();
  const session = useRepair('A');
  session.repair.value.payments.push({ date: '', amount: Infinity });
  assert.equal(session.save(), false);
  assert.equal(state.writes, 0);
});
