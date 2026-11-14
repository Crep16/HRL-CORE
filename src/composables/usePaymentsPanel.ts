import { computed, onMounted, ref, watch } from 'vue';
import { useWorkshopDataStore } from '../stores/useWorkshopDataStore.ts';
import { useWorkshopAccess } from './useWorkshopAccess.ts';
import { useCompany } from './useCompany.ts';
import type { PaymentState, MoneyMovement, MoneyRequest } from '../types/payments.ts';
import { parseMinorUnits } from '../utils/minorUnits.ts';
export function usePaymentsPanel(props:{orderId:string;currency:string;payment?:PaymentState;disabled:boolean;prepare:()=>Promise<boolean>},emit:{(event:"saved",...args:[payment:PaymentState]):void;(event:"refreshed",...args:[payment:PaymentState]):void;(event:"working",...args:[busy:boolean]):void;}){
const {can,requestAccess}=useWorkshopAccess(),{company}=useCompany();
const rows=ref<MoneyMovement[]>([]),busy=ref(false),error=ref(''),notice=ref(''),loaded=ref(false);
watch(busy,value=>emit('working',value));
const mode=ref<MoneyRequest['kind']|''>(''),amount=ref(''),method=ref(''),reason=ref(''),paymentId=ref('');
const requestId=ref(''),attempted=ref(false);
const remaining=computed(()=>Math.max(0,(props.payment?.totalMinor??0)-(props.payment?.collectedMinor??0)));
const original=computed(()=>rows.value.find(row=>row.kind==='receipt'&&row.id===paymentId.value));
const labels={receipt:'Abono',adjustment:'Corrección',refund:'Devolución'};
const money=(minor:number)=>new Intl.NumberFormat(company.value.locale,{style:'currency',currency:props.currency}).format(minor/100);
async function open(kind:MoneyRequest['kind']){const operation=kind==='receipt'?'payments.add':kind==='refund'?'payments.refund':'payments.adjust';if(!can(operation)&&(!await props.prepare()||!await requestAccess(operation,'repairs',props.orderId)))return;mode.value=kind;requestId.value=crypto.randomUUID();attempted.value=false;amount.value='';method.value='';reason.value='';paymentId.value='';notice.value='';}
async function load(){if(busy.value)return;busy.value=true;try{const result=await useWorkshopDataStore().payments(Number(props.orderId));rows.value=result.movements;emit('refreshed',result.payment);loaded.value=true;}catch(cause){error.value=cause && typeof cause==='object' && 'message' in cause ? String(cause.message) : String(cause);loaded.value=false;}finally{busy.value=false;}}
onMounted(load);
async function submit(){
 if(!mode.value||!props.payment||busy.value||props.disabled)return;
 error.value='';notice.value='';
 let minor=0;
 if(mode.value!=='refund'){
  try{if(!String(amount.value).trim())throw new Error('Escribe el importe del movimiento.');minor=parseMinorUnits(String(amount.value));}
  catch(cause){error.value=cause instanceof Error?cause.message:'Escribe un importe válido con hasta dos decimales.';return;}
  if(mode.value==='receipt'&&(minor===0||minor>remaining.value)){error.value='El abono debe ser mayor que cero y no superar el saldo pendiente.';return;}
 }
 const request:MoneyRequest={requestId:requestId.value,orderId:props.orderId,kind:mode.value,paymentId:mode.value==='adjustment'?paymentId.value:null,amountMinor:mode.value==='refund'?props.payment.creditMinor:mode.value==='adjustment'?minor-(original.value?.effectiveMinor??0):minor,expectedMinor:mode.value==='adjustment'?(original.value?.effectiveMinor??0):props.payment.collectedMinor,method:mode.value==='adjustment'?'':method.value,reason:reason.value};
 if(request.kind==='adjustment'&&!original.value){error.value='Selecciona el abono original.';return;}
 if(request.kind!=='adjustment'&&!method.value.trim()){error.value='Escribe el método utilizado.';return;}
 if(request.kind!=='receipt'&&!reason.value.trim()){error.value='Escribe el motivo.';return;}
 if(!await props.prepare())return;
 busy.value=true;attempted.value=true;let sent=false;
 try{
  // Conserva la solicitud exacta para reintentar una respuesta incierta.
  const key=`hrl.3fn.money.pending.${props.orderId}`;
  const stored=localStorage.getItem(key);
  const pending:MoneyRequest=stored?JSON.parse(stored):request;
  if(pending.orderId!==props.orderId)throw new Error('La operación pendiente pertenece a otra orden.');
  localStorage.setItem(key,JSON.stringify(pending));hasPending.value=true;
  sent=true;const result=await useWorkshopDataStore().recordPayment(pending);
  localStorage.removeItem(key);hasPending.value=false;emit('saved',result);mode.value='';attempted.value=false;
  notice.value='Movimiento confirmado y agregado al historial.';
 }catch(cause){
  if(!sent){hasPending.value=false;attempted.value=false;error.value='No se pudo conservar la solicitud antes de enviarla. No se registró el movimiento.';}
  else if(cause && typeof cause==='object' && 'code' in cause && ['VALIDATION_ERROR','CONFLICT','FORBIDDEN','UNAUTHENTICATED'].includes(String(cause.code))){localStorage.removeItem(`hrl.3fn.money.pending.${props.orderId}`);hasPending.value=false;attempted.value=false;error.value='message' in cause ? String(cause.message) : 'La operación fue rechazada.';}
  else{hasPending.value=true;error.value=`${String(cause)} Reintentar conserva la misma operación.`;}
 }
 finally{busy.value=false;}
 await load();
}
async function retryPending(){
 const key=`hrl.3fn.money.pending.${props.orderId}`;
 const raw=localStorage.getItem(key);if(!raw)return;
 busy.value=true;error.value='';
 try{const result=await useWorkshopDataStore().recordPayment(JSON.parse(raw));localStorage.removeItem(key);hasPending.value=false;emit('saved',result);mode.value='';attempted.value=false;notice.value='Operación pendiente confirmada sin duplicarla.';}
 catch(cause){error.value=String(cause);if(cause && typeof cause==='object' && 'code' in cause && ['VALIDATION_ERROR','CONFLICT','FORBIDDEN','UNAUTHENTICATED'].includes(String(cause.code))){localStorage.removeItem(key);hasPending.value=false;attempted.value=false;}}
 finally{busy.value=false;}
 await load();
}
const hasPending=ref(false);
onMounted(()=>{hasPending.value=localStorage.getItem(`hrl.3fn.money.pending.${props.orderId}`)!==null;});
return {can,rows,busy,error,notice,loaded,mode,amount,method,reason,paymentId,attempted,remaining,original,labels,money,open,load,submit,retryPending,hasPending};
}
