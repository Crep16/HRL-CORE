import type { Order } from '../types/order';

export function restoreOrders(raw: string): Order[] {
  const parsed: unknown = JSON.parse(raw);
  if (!Array.isArray(parsed)) throw new Error('Invalid order list');
  const ids = new Set<string>();
  const strings = ['id', 'client', 'phone', 'received', 'status', 'serial', 'item', 'brand', 'model', 'dropReason', 'password', 'work', 'cost', 'deposit', 'notes'];
  const booleans = ['otherShop', 'dropped', 'paid', 'delivered'];
  return parsed.map(value => {
    if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Invalid order');
    if (!strings.every(key => typeof value[key] === 'string') || !booleans.every(key => typeof value[key] === 'boolean')) throw new Error('Invalid order fields');
    if (!value.id.trim() || ids.has(value.id)) throw new Error('Duplicate or empty order id');
    if (value.otherShopReason !== undefined && typeof value.otherShopReason !== 'string') throw new Error('Invalid shop reason');
    ids.add(value.id);
    return { otherShopReason: '', ...value } as Order;
  });
}
