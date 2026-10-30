import type { Order } from '../types/order';
export function orderDisplayStatus(order: Order): string {
  if (!order.delivered) return order.status;
  return order.paid ? 'Entregado y pagado' : 'Entregado con saldo pendiente';
}
