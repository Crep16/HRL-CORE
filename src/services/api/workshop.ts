import type { Photo,PhotoUpload,PhotoUpdate } from '../../types/evidence.ts';
import type { LogbookFilters, LogbookPage } from '../../types/logbook.ts';
import type { FinancialOrder } from '../../types/finances.ts';
import { request } from './transport.ts';
import type { Repair } from '../../composables/useRepair.ts';
import type { MoneyRequest, MoneyMovement, PaymentState } from '../../types/payments.ts';
import { parseMinorUnits } from '../../utils/minorUnits.ts';
type RepairDto = Omit<Repair, 'parts'> & { parts: {name:string;price:number}[] };
export type RepairSnapshot = {repair:Repair;revision:number;payment:PaymentState};
const snapshot = (value: RepairSnapshot): RepairSnapshot => ({...value,repair:{...value.repair,parts:value.repair.parts.map(part=>({...part,price:part.price/100}))}});
export const workshopApi = {
  photos: (orderId:string,deleted:boolean)=>request<Photo[]>('list_normalized_evidence',{orderId:Number(orderId),deleted}),
  readPhoto: (orderId:string,id:string)=>request<number[]>('read_normalized_evidence',{orderId:Number(orderId),id}),
  addPhoto: (input:PhotoUpload)=>request<Photo>('add_normalized_evidence',{input:{...input,orderId:Number(input.orderId)}}),
  updatePhoto: (payload:PhotoUpdate)=>request<Photo>('update_normalized_evidence',{payload:{...payload,orderId:Number(payload.orderId)}}),
  logbook: (filters:LogbookFilters)=>request<LogbookPage>('list_normalized_logbook',{filters}),
  finances: ()=>request<FinancialOrder[]>('list_normalized_finances'),
  expense: (orderId:number,description:string,amountMinor:number,reason:string)=>request<void>('add_normalized_expense',{payload:{orderId,description,amountMinor,reason}}),
  repair: async (orderId:number) => snapshot(await request<RepairSnapshot>('load_normalized_repair',{orderId})),
  saveRepair: async (orderId:number,repair:Repair,revision:number) => {
    const value:RepairDto = {...repair,parts:repair.parts.map(part=>({...part,price:parseMinorUnits(String(part.price))}))};
    return snapshot(await request<RepairSnapshot>('save_normalized_repair',{payload:{orderId,repair:value,revision}}));
  },
  stages: (orderId:number) => request<string[]>('list_normalized_stages',{orderId}),
  hideStageName: (orderId:number,name:string) => request<string[]>('hide_normalized_stage_name',{orderId,name}),
  summary: (orderId:number) => request<{technician:string;publicNote:string;warranties:{name:string;days:number}[]}>('normalized_repair_summary',{orderId}),
  payments: (orderId:number) => request<{movements:MoneyMovement[];payment:PaymentState}>('list_normalized_payments',{orderId}),
  recordPayment: (value:MoneyRequest) => request<PaymentState>('record_normalized_payment',{payload:{...value,orderId:Number(value.orderId),paymentId:value.paymentId?Number(value.paymentId):null}}),
};
