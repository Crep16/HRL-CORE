import { test } from 'node:test';
import assert from 'node:assert/strict';
import { filterFinancialOrders, financialTotals } from '../src/models/finances.ts';
import type { FinancialFilters, FinancialOrder } from '../src/types/finances.ts';

const filters: FinancialFilters = {
  query: '', classification: 'all', dateBasis: 'received', from: '', to: '', scope: 'all',
};
const order: FinancialOrder = {
  id: 'ORD-001', client: 'García', equipment: 'Laptop', status: 'Reparado', currency: 'MXN',
  totalMinor: 60000, collectedMinor: 60000, receivedMinor: 75000, adjustedMinor: -10000,
  refundedMinor: 5000, creditMinor: 0, balanceMinor: 0, costsMinor: 10000, profitMinor: 50000,
  delivered: true, paid: true, classification: 'completed', excluded: false,
  received: '2026-11-10', deliveredAt: '2026-11-12T12:00:00Z', settledAt: null,
  payments: [
    { amountMinor: 75000, kind: 'receipt', occurredAt: '2026-11-10T02:00:00Z' },
    { amountMinor: -10000, kind: 'adjustment', occurredAt: '2026-11-11T12:00:00Z' },
    { amountMinor: -5000, kind: 'refund', occurredAt: '2026-11-12T12:00:00Z' },
  ],
  expenses: [],
};

test('financial totals keep currencies and excluded collections separate', () => {
  const totals = financialTotals([
    order,
    { ...order, id: 'USD-001', currency: 'USD' },
    { ...order, id: 'ORD-002', excluded: true },
  ], filters, 'America/Cancun');
  assert.deepEqual(totals.map(group => group.currency), ['MXN', 'USD']);
  assert.deepEqual(totals[0], {
    currency: 'MXN', total: 60000, collected: 60000, received: 75000, adjusted: -10000,
    refunded: 5000, credit: 0, balance: 0, costs: 10000, profit: 50000,
    completedProfit: 50000, excludedCollected: 60000,
  });
  assert.equal(totals[1].excludedCollected, 0);
});

test('payment periods include adjustments and refunds on their own local dates', () => {
  const paymentFilters = { ...filters, dateBasis: 'payment', from: '2026-11-11', to: '2026-11-12' };
  const totals = financialTotals(filterFinancialOrders([order], paymentFilters, 'America/Cancun'), paymentFilters, 'America/Cancun');
  assert.equal(totals[0].received, 0);
  assert.equal(totals[0].adjusted, -10000);
  assert.equal(totals[0].refunded, 5000);
  assert.equal(totals[0].collected, -15000);
  assert.equal(totals[0].total, 60000);
  const receiptDay = { ...paymentFilters, from: '2026-11-09', to: '2026-11-09' };
  assert.equal(financialTotals([order], receiptDay, 'America/Cancun')[0].received, 75000);
});

test('classification, search, scope and missing dates preserve documented filtering', () => {
  const orders = [order, { ...order, id: 'ORD-002', excluded: true, classification: 'pending', deliveredAt: null }];
  assert.deepEqual(filterFinancialOrders(orders, { ...filters, query: 'garcia', scope: 'active' }, 'UTC'), [order]);
  assert.deepEqual(filterFinancialOrders(orders, { ...filters, classification: 'incomplete' }, 'UTC'), [orders[1]]);
  assert.deepEqual(filterFinancialOrders(orders, { ...filters, dateBasis: 'delivered', from: '2026-11-12', to: '2026-11-12' }, 'UTC'), [order]);
  assert.deepEqual(filterFinancialOrders(orders, { ...filters, dateBasis: 'payment', from: '2027-01-01' }, 'UTC'), []);
  assert.equal(filterFinancialOrders(orders, { ...filters, dateBasis: 'payment' }, 'UTC').length, 2);
});
