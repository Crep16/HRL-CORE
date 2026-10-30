<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useAccess } from '../../composables/useAccess';
import { useCompany } from '../../composables/useCompany';
import type { PaymentState, MoneyMovement, MoneyRequest } from '../../types/payments';
import AppNotice from '../ui/AppNotice.vue';
const props=defineProps<{orderId:string;currency:string;payment?:PaymentState;disabled:boolean;prepare:()=>Promise<boolean>}>();
const emit=defineEmits<{saved:[payment:PaymentState];refreshed:[payment:PaymentState];working:[busy:boolean]}>();
const {can,requestAccess}=useAccess(),{company}=useCompany();
const rows=ref<MoneyMovement[]>([]),busy=ref(false),error=ref(''),notice=ref(''),loaded=ref(false);
watch(busy,value=>emit('working',value));
const mode=ref<MoneyRequest['kind']|''>(''),amount=ref(''),method=ref(''),reason=ref(''),paymentId=ref('');
const requestId=ref(''),attempted=ref(false);
const remaining=computed(()=>Math.max(0,(props.payment?.totalMinor??0)-(props.payment?.collectedMinor??0)));
const original=computed(()=>rows.value.find(row=>row.kind==='receipt'&&row.id===paymentId.value));
const labels={receipt:'Abono',adjustment:'Corrección',refund:'Devolución'};
const money=(minor:number)=>new Intl.NumberFormat(company.value.locale,{style:'currency',currency:props.currency}).format(minor/100);
async function open(kind:MoneyRequest['kind']){const operation=kind==='receipt'?'payments.add':kind==='refund'?'payments.refund':'payments.adjust';if(!can(operation)&&(!await props.prepare()||!await requestAccess(operation,'repairs',props.orderId)))return;mode.value=kind;requestId.value=crypto.randomUUID();attempted.value=false;amount.value='';method.value='';reason.value='';paymentId.value='';notice.value='';}
async function load(){if(busy.value)return;busy.value=true;try{const result=await invoke<{movements:MoneyMovement[];payment:PaymentState}>('list_payment_ledger',{orderId:props.orderId});rows.value=result.movements;emit('refreshed',result.payment);loaded.value=true;}catch(cause){error.value=String(cause);loaded.value=false;}finally{busy.value=false;}}
onMounted(load);
async function submit(){
 if(!mode.value||!props.payment||busy.value||props.disabled)return;
 error.value='';notice.value='';
 const typed=Number(amount.value),minor=Math.round(typed*100);
 if(mode.value!=='refund'&&(!amount.value.trim()||!Number.isSafeInteger(minor)||!Number.isFinite(typed)||Math.abs(typed*100-minor)>.0001||minor<0)){error.value='Escribe un importe válido con hasta dos decimales.';return;}
 const request:MoneyRequest={requestId:requestId.value,orderId:props.orderId,kind:mode.value,paymentId:mode.value==='adjustment'?paymentId.value:null,amountMinor:mode.value==='refund'?props.payment.creditMinor:mode.value==='adjustment'?minor-(original.value?.effectiveMinor??0):minor,expectedMinor:mode.value==='adjustment'?(original.value?.effectiveMinor??0):props.payment.collectedMinor,method:mode.value==='adjustment'?'':method.value,reason:reason.value};
 if(request.kind==='adjustment'&&!original.value){error.value='Selecciona el abono original.';return;}
 if(request.kind!=='adjustment'&&!method.value.trim()){error.value='Escribe el método utilizado.';return;}
 if(request.kind!=='receipt'&&!reason.value.trim()){error.value='Escribe el motivo.';return;}
 if(!await props.prepare())return;
 busy.value=true;attempted.value=true;let sent=false;
 try{
  // Preserve the exact request across an uncertain response, including navigation.
  const key=`hrl.money.pending.${props.orderId}`;
  const stored=localStorage.getItem(key);
  const pending:MoneyRequest=stored?JSON.parse(stored):request;
  if(pending.orderId!==props.orderId)throw new Error('La operación pendiente pertenece a otra orden.');
  localStorage.setItem(key,JSON.stringify(pending));hasPending.value=true;
  sent=true;const result=await invoke<PaymentState>('record_payment_operation',{request:pending});
  localStorage.removeItem(key);hasPending.value=false;emit('saved',result);mode.value='';attempted.value=false;
  notice.value='Movimiento confirmado y agregado al historial.';
 }catch(cause){
  if(!sent){hasPending.value=false;attempted.value=false;error.value='No se pudo conservar la solicitud antes de enviarla. No se registró el movimiento.';}
  else if(typeof cause==='string'&&cause.startsWith('Operación no aplicada: ')){localStorage.removeItem(`hrl.money.pending.${props.orderId}`);hasPending.value=false;attempted.value=false;error.value=cause.slice('Operación no aplicada: '.length);}
  else{hasPending.value=true;error.value=`${String(cause)} Reintentar conserva la misma operación.`;}
 }
 finally{busy.value=false;}
 await load();
}
async function retryPending(){
 const key=`hrl.money.pending.${props.orderId}`;
 const raw=localStorage.getItem(key);if(!raw)return;
 busy.value=true;error.value='';
 try{const result=await invoke<PaymentState>('record_payment_operation',{request:JSON.parse(raw)});localStorage.removeItem(key);hasPending.value=false;emit('saved',result);mode.value='';attempted.value=false;notice.value='Operación pendiente confirmada sin duplicarla.';}
 catch(cause){error.value=String(cause);if(typeof cause==='string'&&cause.startsWith('Operación no aplicada: ')){localStorage.removeItem(key);hasPending.value=false;attempted.value=false;}}
 finally{busy.value=false;}
 await load();
}
const hasPending=ref(false);
onMounted(()=>{hasPending.value=localStorage.getItem(`hrl.money.pending.${props.orderId}`)!==null;});
</script>
<template>
 <section class="payment-panel" aria-labelledby="payment-heading" :aria-busy="busy">
  <h2 id="payment-heading" class="sec">Pago</h2>
  <dl v-if="payment" class="payment-totals"><div v-for="(value,label) in {'Total':payment.totalMinor,'Cobros originales':payment.receivedMinor,'Ajustes':payment.adjustedMinor,'Devuelto':payment.refundedMinor,'Cobro neto':payment.collectedMinor,'Pendiente':remaining,'Saldo a favor':payment.creditMinor}" :key="label"><dt>{{ label }}</dt><dd>{{ money(value) }}</dd></div></dl>
  <AppNotice :message="error" tone="error"><button type="button" class="btn btn-s" :disabled="busy" @click="load">Actualizar movimientos</button></AppNotice>
  <AppNotice :message="notice" @dismiss="notice=''" />
  <p v-if="busy" role="status">Consultando o registrando el movimiento…</p>
  <button v-if="hasPending||attempted" type="button" class="btn btn-s" :disabled="busy||disabled" @click="retryPending">Reintentar operación pendiente</button>
  <div v-if="!mode&&!hasPending&&!attempted" class="payment-actions">
   <button class="btn btn-s" :disabled="disabled||busy||!loaded||remaining===0" @click="open('receipt')">{{ can('payments.add')?'Registrar abono':'Autorizar abono' }}</button>
   <button class="btn btn-s" :disabled="disabled||busy||!loaded||!payment?.creditMinor" @click="open('refund')">{{ can('payments.refund')?'Devolver saldo a favor':'Autorizar devolución' }}</button>
   <button class="btn btn-s" :disabled="disabled||busy||!loaded||!rows.some(row=>row.kind==='receipt')" @click="open('adjustment')">{{ can('payments.adjust')?'Corregir abono':'Autorizar corrección' }}</button>
  </div>
  <form v-if="mode" class="payment-form" @submit.prevent="submit">
   <h3>{{ labels[mode] }}</h3>
   <fieldset :disabled="disabled||busy||attempted">
    <template v-if="mode==='adjustment'"><label for="original-payment">Abono original</label><select id="original-payment" v-model="paymentId" required><option value="">Seleccionar abono</option><option v-for="row in rows.filter(row=>row.kind==='receipt')" :key="row.id" :value="row.id">{{ row.occurredAt||'Fecha no registrada' }} · {{ money(row.effectiveMinor??row.amountMinor) }} · {{ row.id }}</option></select><p>La diferencia quedará ligada al original, que se conserva. No representa una devolución.</p></template>
    <label v-if="mode!=='refund'" for="money-amount">{{ mode==='adjustment'?'Importe correcto del abono':'Importe del abono' }}</label>
    <input v-if="mode!=='refund'" id="money-amount" v-model="amount" type="number" :min="mode==='receipt'?'0.01':'0'" step="0.01" :max="mode==='receipt'?remaining/100:undefined" required />
    <p v-else>Devolverás todo el saldo a favor: <strong>{{ money(payment?.creditMinor??0) }}</strong>. Esta acción confirma que el dinero fue devuelto al cliente.</p>
    <template v-if="mode!=='adjustment'"><label for="money-method">Método utilizado</label><input id="money-method" v-model="method" maxlength="200" required placeholder="Escribe el método" /></template>
    <label for="money-reason">{{ mode==='receipt'?'Nota (opcional)':'Motivo' }}</label><textarea id="money-reason" v-model="reason" maxlength="2000" :required="mode!=='receipt'" rows="2"></textarea>
   </fieldset>
   <button v-if="!attempted" class="btn btn-p" :disabled="disabled||busy">{{ mode==='refund'?'Confirmar devolución total':'Confirmar movimiento' }}</button>
   <button v-if="!attempted" class="btn btn-s" type="button" @click="mode=''">Cancelar</button>
  </form>
  <details><summary>Movimientos de dinero</summary><button class="btn btn-s" :disabled="busy" @click="load">Actualizar</button><ol class="money-movements"><li v-for="row in rows" :key="row.id"><strong>{{ labels[row.kind] }} · {{ money(row.amountMinor) }}</strong><span>{{ row.occurredAt||'Fecha no registrada' }} · {{ row.method||'Método no registrado' }}</span><span v-if="row.paymentId">Abono original: {{ row.paymentId }}</span><span v-if="row.effectiveMinor!==null">Importe corregido: {{ money(row.effectiveMinor) }}</span><span v-if="row.reason">{{ row.reason }}</span></li></ol><p v-if="loaded&&!rows.length">Sin movimientos de dinero.</p></details>
 </section>
</template>
<style scoped>
.payment-panel,.payment-form,.payment-actions{display:grid;gap:12px;min-width:0}.payment-totals{margin:0;display:grid;gap:8px}.payment-totals>div{display:flex;justify-content:space-between;gap:12px}.payment-totals dt{color:var(--muted)}.payment-totals dd{margin:0;font-weight:700;overflow-wrap:anywhere}.payment-form fieldset{border:0;padding:0;margin:0;min-width:0;display:grid;gap:8px}.payment-form input,.payment-form select,.payment-form textarea{width:100%;min-width:0;min-height:44px;border:1px solid #73838f;border-radius:8px;background:var(--surface,#fff);padding:10px;font:inherit;color:var(--ink)}.payment-form p,.money-movements{font-size:14px;line-height:1.5}.money-movements{padding-left:20px}.money-movements li{margin:12px 0;overflow-wrap:anywhere}.money-movements span{display:block;color:var(--muted)}summary{min-height:44px;display:flex;align-items:center;cursor:pointer}.payment-form :focus-visible,summary:focus-visible{outline:2px solid var(--acc-ink);outline-offset:3px}
</style>
