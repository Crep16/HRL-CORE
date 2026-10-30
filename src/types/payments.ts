export type PaymentState = { totalMinor:number; collectedMinor:number; receivedMinor:number; adjustedMinor:number; refundedMinor:number; creditMinor:number; paid:boolean };
export type MoneyMovement = { id:string; kind:'receipt'|'adjustment'|'refund'; paymentId:string|null; amountMinor:number; effectiveMinor:number|null; method:string|null; reason:string|null; occurredAt:string|null };
export type MoneyRequest = { requestId:string; orderId:string; kind:'receipt'|'adjustment'|'refund'; paymentId:string|null; amountMinor:number; expectedMinor:number; method:string; reason:string };
