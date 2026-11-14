import type { OrderResponse } from '../types/ipc.ts';
import type { Order } from '../types/order.ts';
import { formatMinorUnits } from './minorUnits.ts';

export function legacyOrder(order:OrderResponse):Order {
  return {id:order.folio,databaseId:order.id,currency:order.currency as Order['currency'],client:order.client.fullName,phone:order.client.phone,received:order.receivedAt,status:order.status,serial:order.equipment.serial,item:order.equipment.item,brand:order.equipment.brand,model:order.equipment.model,otherShop:order.history.previousWorkshop,otherShopReason:order.history.previousWorkshopDetail,dropped:order.history.sufferedImpact||order.history.liquidSpill,dropReason:[order.history.impactDetail,order.history.liquidDetail].filter(Boolean).join('\n'),password:'',failure:order.reportedFailure,work:order.plannedWork,cost:formatMinorUnits(order.priceMinor),deposit:formatMinorUnits(order.netMinor),notes:order.receptionNotes,paid:order.pendingMinor===0,delivered:order.delivered};
}
