import assert from 'node:assert/strict';
import { test } from 'node:test';
import { restoreOrders } from '../src/utils/restoreOrders.ts';

const order = Object.fromEntries(['id', 'client', 'phone', 'received', 'status', 'serial', 'item', 'brand', 'model', 'dropReason', 'password', 'work', 'cost', 'deposit', 'notes'].map(key => [key, key]));
Object.assign(order, {otherShop:false, dropped:false, paid:false, delivered:false});

test('restore preserves orders and migrates the optional shop reason', () => {
  assert.deepEqual(restoreOrders(JSON.stringify([order])), [{...order, otherShopReason:''}]);
  assert.deepEqual(restoreOrders('[]'), []);
});

test('restore rejects malformed fields and duplicate folios', () => {
  for (const value of [null, {}, [null], [order, order], [{...order, id:''}], [{...order, paid:'false'}], [{...order, phone:null}], [{...order, otherShopReason:42}]]) {
    assert.throws(() => restoreOrders(JSON.stringify(value)));
  }
  assert.throws(() => restoreOrders('{'));
});
