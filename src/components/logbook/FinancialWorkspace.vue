<script setup lang="ts">
import SearchSelect from '../ui/SearchSelect.vue';
import AppNotice from '../ui/AppNotice.vue';
import { useFinancialWorkspace } from '../../composables/useFinancialWorkspace';
const emit=defineEmits<{history:[orderId:string]}>();
const {can,busy,error,message,filters,page,selectedId,validRange,filtered,totals,visible,selected,cost,money,date,load,addCost,financialClasses}=useFinancialWorkspace();

</script>
<template>
 <section class="financial-workspace" aria-labelledby="finance-title">
  <header class="logbook-heading"><div><h2 id="finance-title">Resumen financiero de órdenes</h2><p>Pago, entrega y avance técnico se consultan por separado.</p></div><button class="btn btn-s" :disabled="busy" @click="load">Actualizar</button></header>
  <form class="logbook-filters finance-filters" @submit.prevent>
   <div class="fld"><label for="finance-search">Folio o cliente</label><input id="finance-search" v-model="filters.query" type="search" placeholder="Buscar orden" /></div>
   <SearchSelect v-model="filters.classification" label="Clasificación" :options="Object.entries(financialClasses).map(([value,label])=>({value,label}))" />
   <SearchSelect v-model="filters.dateBasis" label="Filtrar fechas de" :options="[{value:'received',label:'Recepción'},{value:'payment',label:'Movimientos de pago'},{value:'delivered',label:'Entrega'}]" />
   <SearchSelect v-model="filters.scope" label="Órdenes incluidas" :options="[{value:'active',label:'Activas'},{value:'excluded',label:'Canceladas o en papelera'},{value:'all',label:'Todas, con trazabilidad'}]" />
   <div class="fld"><label for="finance-from">Desde</label><input id="finance-from" v-model="filters.from" type="date" :max="filters.to||undefined" /></div>
   <div class="fld"><label for="finance-to">Hasta</label><input id="finance-to" v-model="filters.to" type="date" :min="filters.from||undefined" /></div>
   <button class="btn btn-s" type="button" @click="Object.assign(filters,{query:'',classification:'all',dateBasis:'received',from:'',to:'',scope:'active'})">Limpiar filtros</button>
  </form>
  <p v-if="!validRange" class="logbook-error" role="alert">La fecha inicial debe ser anterior o igual a la final.</p>
  <AppNotice v-if="busy" message="Consultando importes y movimientos…" :closable="false" /><AppNotice :message="error" tone="error"><button class="btn btn-s" @click="load">Reintentar</button></AppNotice><AppNotice :message="message" @dismiss="message=''" />
  <div v-for="group in totals" :key="group.currency" class="finance-totals">
   <h3>Totales filtrados · {{ group.currency }}</h3>
   <dl><div v-for="(value,label) in {'Total de órdenes activas':group.total,[filters.dateBasis==='payment'?'Cobrado en el periodo':'Cobrado acumulado']:group.collected,'Saldo pendiente':group.balance,'Costos directos registrados':group.costs,'Ganancia estimada':group.profit,'Ganancia estimada de completadas':group.completedProfit}" :key="label"><dt>{{ label }}</dt><dd>{{ money(value,group.currency) }}</dd></div></dl>
   <details class="reception-details"><summary>Desglose de cobros y saldo a favor</summary><dl><div v-for="(value,label) in {'Cobros originales':group.received,'Ajustes de cobros':group.adjusted,'Devuelto':group.refunded,'Saldo a favor':group.credit}" :key="label"><dt>{{ label }}</dt><dd>{{ money(value,group.currency) }}</dd></div></dl></details>
   <p v-if="group.excludedCollected">Cobros rastreables de canceladas o papelera: {{ money(group.excludedCollected,group.currency) }}. Se excluyen de los totales de órdenes activas.</p>
  </div>
  <p class="finance-explanation">Las ganancias son provisionales: se calculan con los costos registrados y antes de gastos generales. {{ filters.dateBasis==='payment'?'El cobrado del periodo suma solo los movimientos de pago de esas fechas; el saldo y la ganancia usan el acumulado de cada orden.':'Lo cobrado es el acumulado real de las órdenes filtradas, no un ingreso atribuido a su fecha de recepción o entrega.' }} Los pagos sin fecha conocida no se asignan a un periodo.</p>
  <div class="logbook-layout">
   <section class="logbook-records"><header class="records-heading"><h3>Órdenes</h3><span>{{ filtered.length }} registros</span></header>
    <div class="logbook-table-wrap"><table class="logbook-table finance-table"><caption class="sr">Órdenes financieras y sus estados independientes</caption><thead><tr><th>Orden y cliente</th><th>Clasificación</th><th>Total / cobrado</th><th>Saldo / ganancia estimada</th></tr></thead><tbody>
     <tr v-for="order in visible" :key="order.id" :class="{selected:selected?.id===order.id}"><td><button class="logbook-select" :aria-pressed="selected?.id===order.id" @click="selectedId=order.id">{{ order.id }}</button><span class="record-order">{{ order.client }}</span><span class="record-description">{{ order.equipment }}</span></td><td>{{ financialClasses[order.classification] }}<span class="record-description">Técnico: {{ order.status }}</span><strong v-if="order.excluded" class="record-description">Fuera de totales activos</strong></td><td>{{ money(order.totalMinor,order.currency) }}<span class="record-description">Cobrado: {{ money(order.collectedMinor,order.currency) }}</span></td><td>{{ money(order.balanceMinor,order.currency) }}<span class="record-description">Ganancia: {{ money(order.profitMinor,order.currency) }}</span></td></tr>
    </tbody></table></div>
    <p v-if="!filtered.length&&!busy" class="logbook-empty">No hay órdenes que coincidan con los filtros.</p>
    <nav v-if="filtered.length" class="logbook-pagination" aria-label="Páginas de órdenes financieras"><span>Página {{ page+1 }} de {{ Math.ceil(filtered.length/50) }}</span><div><button class="btn btn-s" :disabled="page===0" @click="page--">Anterior</button><button class="btn btn-s" :disabled="(page+1)*50>=filtered.length" @click="page++">Siguiente</button></div></nav>
   </section>
   <aside class="logbook-detail"><h3>Detalle financiero</h3><template v-if="selected"><h4>{{ selected.id }} · {{ selected.client }}</h4><dl class="logbook-metadata"><div><dt>Estado técnico</dt><dd>{{ selected.status }}</dd></div><div><dt>Pago completo</dt><dd>{{ selected.paid?'Sí':'No' }}</dd></div><div><dt>Entregado</dt><dd>{{ selected.delivered?'Sí':'No' }}</dd></div><div><dt>Recepción</dt><dd>{{ date(selected.received) }}</dd></div><div><dt>Liquidación</dt><dd>{{ date(selected.settledAt) }}</dd></div><div><dt>Entrega</dt><dd>{{ date(selected.deliveredAt) }}</dd></div></dl>
    <dl class="logbook-metadata"><div><dt>Cobros originales</dt><dd>{{ money(selected.receivedMinor??selected.collectedMinor,selected.currency) }}</dd></div><div><dt>Ajustes</dt><dd>{{ money(selected.adjustedMinor??0,selected.currency) }}</dd></div><div><dt>Devuelto</dt><dd>{{ money(selected.refundedMinor??0,selected.currency) }}</dd></div><div><dt>Cobro neto</dt><dd>{{ money(selected.collectedMinor,selected.currency) }}</dd></div><div><dt>Saldo a favor</dt><dd>{{ money(selected.creditMinor??0,selected.currency) }}</dd></div></dl>
    <button class="btn btn-s" @click="emit('history',selected.id)">Historial de esta orden</button>
    <section class="logbook-change"><h4>Movimientos de pago</h4><p v-for="(payment,index) in selected.payments" :key="index">{{ date(payment.occurredAt) }} · {{ money(payment.amountMinor,selected.currency) }}</p><p v-if="!selected.payments.length">Sin pagos registrados.</p></section>
    <section class="logbook-change"><h4>Costos directos</h4><p v-for="(expense,index) in selected.expenses" :key="index">{{ expense.description }} · {{ money(expense.amountMinor,selected.currency) }}</p><p v-if="!selected.expenses.length">Sin costos registrados. La ganancia es provisional.</p></section>
    <details v-if="can('repairs.edit')&&!selected.excluded" class="expense-composer"><summary>Registrar otro gasto directo</summary><form class="expense-form" @submit.prevent="addCost"><div class="fld"><label for="expense-name">Concepto</label><input id="expense-name" v-model="cost.description" required maxlength="200" /></div><div class="fld"><label for="expense-amount">Importe · {{ selected.currency }}</label><input id="expense-amount" v-model.number="cost.amount" type="number" min="0.01" step="0.01" required /></div><div class="fld"><label for="expense-reason">Motivo</label><textarea id="expense-reason" v-model="cost.reason" required maxlength="500" /></div><button class="btn btn-p" :disabled="busy">Registrar costo</button></form></details>
   </template><p v-else>Selecciona una orden para consultar sus movimientos.</p></aside>
  </div>
 </section>
</template>
<style scoped src="../../assets/styles/logbook.css"></style>
<style scoped>
.finance-filters{grid-template-columns:repeat(4,minmax(0,1fr))}.finance-totals{padding:20px;margin-bottom:16px;background:white;border:1px solid var(--line);border-radius:12px}.finance-totals h3{margin:0 0 16px;font:700 20px var(--disp)}.finance-totals dl{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px;margin:0}.finance-totals dl>div{min-width:0;border-left:3px solid var(--line);padding-left:12px}.finance-totals dt{font-size:14px;color:var(--muted)}.finance-totals dd{margin:6px 0 0;font-size:21px;font-weight:700;overflow-wrap:anywhere}.finance-explanation{color:var(--muted);font-size:14px;line-height:1.6;margin:20px 0}.finance-table th:first-child,.finance-table th:last-child{width:25%}.expense-composer{margin-top:20px;border-top:1px solid var(--line)}.expense-composer summary{min-height:44px;padding-block:12px;cursor:pointer;color:var(--acc-ink)}.expense-form{display:grid;gap:12px}.expense-form input,.expense-form textarea{width:100%;padding:10px;border:1px solid var(--line2);border-radius:8px;min-height:44px;font:inherit}.financial-workspace .logbook-heading h2{font:700 24px var(--disp);margin:0}@media(max-width:1100px){.finance-filters{grid-template-columns:1fr 1fr}.finance-totals dl{grid-template-columns:1fr 1fr}}@media(max-width:520px){.finance-filters,.finance-totals dl{grid-template-columns:1fr}.finance-totals dd{font-size:20px}}
</style>
