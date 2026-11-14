<script setup lang="ts">
import AppNotice from '../ui/AppNotice.vue';
import type { PaymentState } from '../../types/payments';
import { usePaymentsPanel } from '../../composables/usePaymentsPanel';
const props=defineProps<{orderId:string;currency:string;payment?:PaymentState;disabled:boolean;prepare:()=>Promise<boolean>}>();
const emit=defineEmits<{saved:[payment:PaymentState];refreshed:[payment:PaymentState];working:[busy:boolean]}>();
const {can,rows,busy,error,notice,loaded,mode,amount,method,reason,paymentId,attempted,remaining,labels,money,open,load,submit,retryPending,hasPending}=usePaymentsPanel(props,emit);

</script>
<template>
 <section class="payment-panel" aria-labelledby="payment-heading" :aria-busy="busy">
  <h2 id="payment-heading" class="sec">Pago</h2>
  <template v-if="payment">
   <dl class="payment-totals"><div><dt>Total</dt><dd>{{ money(payment.totalMinor) }}</dd></div><div><dt>Abonado</dt><dd>{{ money(payment.collectedMinor) }}</dd></div></dl>
   <progress class="payment-progress" :value="Math.max(0,payment.collectedMinor)" :max="Math.max(1,payment.totalMinor)" aria-label="Importe abonado"></progress>
   <div class="payment-balance"><span>Saldo pendiente</span><strong>{{ money(remaining) }}</strong></div>
   <div v-for="row in rows.filter(item=>item.kind==='receipt').slice(0,3)" :key="row.id" class="payment-receipt"><span>{{ row.occurredAt?.replace('T',' ').slice(0,16) || 'Fecha no registrada' }}</span><strong>{{ money(row.effectiveMinor ?? row.amountMinor) }}</strong></div>
  </template>
  <AppNotice :message="error" tone="error"><button type="button" class="btn btn-s" :disabled="busy" @click="load">Actualizar movimientos</button></AppNotice>
  <AppNotice :message="notice" @dismiss="notice=''" />
  <p v-if="busy" role="status">Consultando o registrando el movimiento…</p>
  <button v-if="hasPending||attempted" type="button" class="btn btn-s" :disabled="busy||disabled" @click="retryPending">Reintentar operación pendiente</button>
  <div v-if="!mode&&!hasPending&&!attempted" class="payment-actions">
   <button class="btn btn-s" :disabled="disabled||busy||!loaded||remaining===0" @click="open('receipt')">{{ can('payments.add')?'+ Registrar abono':'Autorizar abono' }}</button>
   <details class="payment-extra"><summary>Correcciones y devoluciones</summary>
   <dl v-if="payment" class="payment-totals"><div v-for="(value,label) in {'Cobros originales':payment.receivedMinor,'Ajustes':payment.adjustedMinor,'Devuelto':payment.refundedMinor,'Saldo a favor':payment.creditMinor}" :key="label"><dt>{{ label }}</dt><dd>{{ money(value) }}</dd></div></dl>
   <button class="btn btn-s" :disabled="disabled||busy||!loaded||!payment?.creditMinor" @click="open('refund')">{{ can('payments.refund')?'Devolver saldo a favor':'Autorizar devolución' }}</button>
   <button class="btn btn-s" :disabled="disabled||busy||!loaded||!rows.some(row=>row.kind==='receipt')" @click="open('adjustment')">{{ can('payments.adjust')?'Corregir abono':'Autorizar corrección' }}</button>
   </details>
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
.payment-progress{appearance:none;width:100%;height:8px;border:0;border-radius:4px;overflow:hidden;background:var(--line)}
.payment-progress::-webkit-progress-bar{background:var(--line)}
.payment-progress::-webkit-progress-value{background:var(--acc);border-radius:4px}
.payment-balance,.payment-receipt{display:flex;align-items:baseline;justify-content:space-between;gap:12px;padding-bottom:10px;border-bottom:1px solid var(--line);font-size:14px}
.payment-balance{flex-wrap:wrap;row-gap:4px}
.payment-balance strong{font:800 30px/1.15 var(--disp);white-space:nowrap;max-width:100%;overflow-x:auto}
.payment-receipt span{font-size:13px}
.payment-totals>div{min-height:40px;align-items:center;border-bottom:1px solid var(--line)}
.payment-extra>.btn{width:100%;margin-top:10px}
.payment-panel,.payment-form,.payment-actions{display:grid;gap:12px;min-width:0}.payment-totals{margin:0;display:grid;gap:8px}.payment-totals>div{display:flex;justify-content:space-between;gap:12px}.payment-totals dt{color:var(--muted)}.payment-totals dd{margin:0;font-weight:700;overflow-wrap:anywhere}.payment-form fieldset{border:0;padding:0;margin:0;min-width:0;display:grid;gap:8px}.payment-form input,.payment-form select,.payment-form textarea{width:100%;min-width:0;min-height:44px;border:1px solid #73838f;border-radius:8px;background:var(--surface,#fff);padding:10px;font:inherit;color:var(--ink)}.payment-form p,.money-movements{font-size:14px;line-height:1.5}.money-movements{padding-left:20px}.money-movements li{margin:12px 0;overflow-wrap:anywhere}.money-movements span{display:block;color:var(--muted)}summary{min-height:44px;display:flex;align-items:center;cursor:pointer}.payment-form :focus-visible,summary:focus-visible{outline:2px solid var(--acc-ink);outline-offset:3px}
</style>
