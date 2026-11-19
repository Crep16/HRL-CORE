import type { FinancialFilters, FinancialOrder } from '../types/finances.ts';
import { calendarDate } from '../utils/finances.ts';

export function inDateRange(
  value: string | null,
  filters: Pick<FinancialFilters, 'from' | 'to'>,
  timezone: string,
): boolean {
  if (!filters.from && !filters.to) return true;
  const day = calendarDate(value, timezone);
  return !!day && (!filters.from || day >= filters.from) && (!filters.to || day <= filters.to);
}

export function filterFinancialOrders(
  orders: FinancialOrder[],
  filters: FinancialFilters,
  timezone: string,
) {
  const query = filters.query.trim().normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
  return orders.filter(order => {
    if (filters.scope === 'active' && order.excluded || filters.scope === 'excluded' && !order.excluded) return false;
    const searchable = `${order.id} ${order.client}`.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
    if (!searchable.includes(query)) return false;
    if (filters.classification === 'incomplete' && order.classification === 'completed') return false;
    if (!['all', 'incomplete'].includes(filters.classification) && filters.classification !== order.classification) return false;
    if (filters.dateBasis === 'payment') {
      return !filters.from && !filters.to || order.payments.some(payment => inDateRange(payment.occurredAt, filters, timezone));
    }
    return inDateRange(filters.dateBasis === 'delivered' ? order.deliveredAt : order.received, filters, timezone);
  });
}

type FinancialBreakdown = {
  classification: string;
  count: number;
  total: number;
  collected: number;
  balance: number;
  costs: number;
  profit: number;
};

type FinancialTotal = {
  currency: string;
  total: number;
  collected: number;
  received: number;
  adjusted: number;
  refunded: number;
  credit: number;
  balance: number;
  costs: number;
  profit: number;
  completedProfit: number;
  paidProfit: number;
  pendingProfit: number;
  breakdown: FinancialBreakdown[];
  excludedCollected: number;
};

export function financialTotals(orders: FinancialOrder[], filters: FinancialFilters, timezone: string) {
  const totals = new Map<string, FinancialTotal>();
  for (const order of orders) {
    let group = totals.get(order.currency);
    if (!group) {
      group = {
        currency: order.currency, total: 0, collected: 0, received: 0, adjusted: 0,
        refunded: 0, credit: 0, balance: 0, costs: 0, profit: 0, completedProfit: 0,
        paidProfit: 0, pendingProfit: 0, excludedCollected: 0,
        breakdown: ['completed', 'paid_uncollected', 'delivered_unpaid', 'pending'].map(classification => ({
          classification, count: 0, total: 0, collected: 0, balance: 0, costs: 0, profit: 0,
        })),
      };
      totals.set(order.currency, group);
    }
    const movements = order.payments.filter(payment => filters.dateBasis !== 'payment' || inDateRange(payment.occurredAt, filters, timezone));
    const collected = filters.dateBasis === 'payment'
      ? movements.reduce((sum, payment) => sum + payment.amountMinor, 0)
      : order.collectedMinor;
    if (order.excluded) {
      group.excludedCollected += collected;
      continue;
    }
    group.received += filters.dateBasis === 'payment'
      ? movements.filter(payment => !payment.kind || payment.kind === 'receipt').reduce((sum, payment) => sum + payment.amountMinor, 0)
      : (order.receivedMinor ?? order.collectedMinor);
    group.adjusted += filters.dateBasis === 'payment'
      ? movements.filter(payment => payment.kind === 'adjustment').reduce((sum, payment) => sum + payment.amountMinor, 0)
      : (order.adjustedMinor ?? 0);
    group.refunded += filters.dateBasis === 'payment'
      ? -movements.filter(payment => payment.kind === 'refund').reduce((sum, payment) => sum + payment.amountMinor, 0)
      : (order.refundedMinor ?? 0);
    group.credit += order.creditMinor ?? 0;
    group.total += order.totalMinor;
    group.collected += collected;
    group.balance += order.balanceMinor;
    group.costs += order.costsMinor;
    group.profit += order.profitMinor;
    if (order.paid) group.paidProfit += order.profitMinor;
    else group.pendingProfit += order.profitMinor;
    const breakdown = group.breakdown.find(item => item.classification === order.classification);
    if (breakdown) {
      breakdown.count++;
      breakdown.total += order.totalMinor;
      breakdown.collected += collected;
      breakdown.balance += order.balanceMinor;
      breakdown.costs += order.costsMinor;
      breakdown.profit += order.profitMinor;
    }
    if (order.classification === 'completed') group.completedProfit += order.profitMinor;
  }
  return [...totals.values()];
}
