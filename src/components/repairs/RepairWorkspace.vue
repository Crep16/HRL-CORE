<script setup lang="ts">
import { computed, ref } from 'vue';
import type { Order } from '../../types/order';
import { useRepair } from '../../composables/useRepair';
import { useCompany } from '../../composables/useCompany';
import { companyDateTime } from '../../utils/companyDate';
import { orderAmount } from '../../utils/orderAmount';
const props = defineProps<{ order: Order }>();
const emit = defineEmits<{ back: [] }>();
const { repair, save, message } = useRepair(props.order.id);
const { company } = useCompany();
defineExpose({ save });
const stages = ['Recibido', 'Diagnóstico', 'Autorización', 'Refacciones', 'Reparación', 'Pruebas', 'Entrega'];
const noteType = ref<'publicNote' | 'internalNote'>('publicNote');
const title = ref('');
const detail = ref('');
const stage = ref('Reparación');
const publicEvent = ref(true);
const workspaceTab = ref<'work' | 'history'>('work');
const now = () => companyDateTime(company.value.timezone);
const date = ref(now());
const addingWarranty = ref(false);
const warranty = ref('');
const warrantyType = ref<'parts' | 'labor'>('parts');
const days = ref(company.value.partsDays);
const addingPart = ref(false);
const part = ref('');
const price = ref(0);
const paymentOpen = ref(false);
const amount = ref(0);
const money = (value: number) => new Intl.NumberFormat(company.value.locale, { style:'currency', currency:props.order.currency ?? company.value.currency }).format(value);
const number = orderAmount;
const deposited = computed(() => number(props.order.deposit) + repair.value.payments.reduce((sum, item) => sum + item.amount, 0));
const balance = computed(() => Math.max(0, number(props.order.cost) - deposited.value));
function addEvent() {
  if (!title.value.trim()) return;
  repair.value.events.unshift({ id:Date.now(), date: date.value, title:title.value.trim(), detail:detail.value, stage:stage.value, public:publicEvent.value });
  title.value = ''; detail.value = ''; save();
}
function complete() {
  if (repair.value.completed >= stages.length) return;
  repair.value.events.unshift({ id:Date.now(), date:now(), title:`${stages[repair.value.completed]}: etapa completada`, detail:'', stage:stages[repair.value.completed]!, public:true });
  repair.value.completed++; save();
}
function addPayment() {
  if (amount.value <= 0 || amount.value > balance.value) { message.value = 'El abono debe ser mayor a cero y no superar el saldo.'; return; }
  repair.value.payments.push({ date:now(), amount:amount.value }); amount.value = 0; paymentOpen.value = false; save();
}
function addWarranty() {
  if (!warranty.value.trim()) return;
  repair.value.warranties.push({ name:warranty.value.trim(), days:days.value }); warranty.value=''; addingWarranty.value=false; save();
}
function addPart() {
  if (!part.value.trim()) return;
  repair.value.parts.push({ name:part.value.trim(), price:price.value }); part.value=''; price.value=0; addingPart.value=false; save();
}
function back() { if (save()) emit('back'); }
</script>

<template>
  <div class="repair-page">
    <p v-if="message" class="notice" role="status">{{ message }}</p>
    <div class="work">
      <aside class="rail" aria-label="Contexto de la orden">
        <button class="btn btn-s" style="align-self:flex-start" @click="back">‹ Volver a clientes</button>
        <section class="card"><h2 class="h3">Registro seleccionado</h2>
          <div v-for="(value, label) in { Orden:order.id, Cliente:order.client, Estado:order.status, Artículo:order.item }" :key="label" class="kv"><span class="k">{{ label }}</span><span class="v">{{ value || 'Sin registrar' }}</span></div>
          <details class="equipment-details">
            <summary>Datos del equipo</summary>
            <div v-for="(value, label) in { Serie:order.serial, Marca:order.brand, Modelo:order.model, 'Fecha de entrada':order.received }" :key="label" class="kv"><span class="k">{{ label }}</span><span class="v">{{ value || 'Sin registrar' }}</span></div>
          </details>
          <div class="fld"><label for="technician">Técnico</label><input id="technician" v-model="repair.technician" placeholder="Asignar responsable" /></div>
        </section>
        <section class="block"><h2 class="sec">Pago</h2>
          <div class="line"><span class="k">Total</span><strong>{{ order.cost }}</strong></div>
          <div class="line"><span class="k">Abonado</span><strong>{{ money(deposited) }}</strong></div>
          <div class="bar"><i :style="{ width: `${Math.min(100, deposited / (number(order.cost) || 1) * 100)}%` }"></i></div>
          <div class="line"><span>Saldo pendiente</span><strong style="font:800 30px var(--disp)">{{ money(balance) }}</strong></div>
          <div v-for="(payment, i) in repair.payments" :key="i" class="line"><span>{{ payment.date.replace('T',' ') }}</span><strong>{{ money(payment.amount) }}</strong></div>
          <button class="btn btn-s" @click="paymentOpen = !paymentOpen">+ Registrar abono</button>
          <form v-if="paymentOpen" class="compose" @submit.prevent="addPayment"><div class="fld"><label for="payment">Importe</label><input id="payment" v-model.number="amount" type="number" min="0.01" step="0.01" :max="balance" required /></div><button class="btn btn-p">Guardar abono</button></form>
        </section>
      </aside>
      <section class="sheet" aria-label="Gestión de la reparación">
        <div class="sh-h"><div><h1 class="num">Reparaciones</h1><p class="subt">Orden {{ order.id }} de {{ order.client }}</p></div><div class="sh-act"><button class="btn btn-p" @click="save">Guardar cambios</button></div></div>
        <div class="perf" aria-hidden="true"></div>
        <div class="workspace-tabs" role="group" aria-label="Vista de reparación">
          <button class="btn" :class="workspaceTab === 'work' ? 'btn-p' : 'btn-s'" :aria-pressed="workspaceTab === 'work'" @click="workspaceTab = 'work'">Trabajo y notas</button>
          <button class="btn" :class="workspaceTab === 'history' ? 'btn-p' : 'btn-s'" :aria-pressed="workspaceTab === 'history'" @click="workspaceTab = 'history'">Historial ({{ repair.events.length }})</button>
        </div>
        <div class="sh-b" :class="{ 'history-view': workspaceTab === 'history' }">
          <div v-show="workspaceTab === 'work'" class="repair-work">
          <section class="block" style="gap:18px"><h2 class="sec">Avance del trabajo<small>{{ Math.round(repair.completed / 7 * 100) }}%</small></h2>
            <ol class="steps"><li v-for="(item, i) in stages" :key="item" class="step" :class="{ done:i < repair.completed, cur:i === repair.completed }"><span class="st-c"><span v-if="i < repair.completed">✓</span><i v-else-if="i === repair.completed"></i></span><span class="st-l">{{ item }}</span></li></ol>
            <div class="actions"><p class="help">Completaste {{ repair.completed }} de 7 etapas. {{ repair.completed < 7 ? `Ahora: ${stages[repair.completed]}.` : 'Trabajo completado.' }}</p><button class="btn btn-p" :disabled="repair.completed === 7" @click="complete">Completar etapa</button></div>
          </section>
          <section class="block"><h2 class="sec">Notas del técnico</h2><div class="seg" style="max-width:340px"><button :class="{on:noteType === 'publicNote'}" :aria-pressed="noteType === 'publicNote'" @click="noteType = 'publicNote'">Para el cliente</button><button :class="{on:noteType === 'internalNote'}" :aria-pressed="noteType === 'internalNote'" @click="noteType = 'internalNote'">Internas</button></div><div class="fld note"><label class="sr" for="technical-note">Nota del técnico</label><textarea id="technical-note" v-model="repair[noteType]" rows="4" placeholder="Escribe directamente aquí…"></textarea></div><p class="help">{{ noteType === 'publicNote' ? 'Nota preparada para el seguimiento del cliente.' : 'Nota interna del taller. No se comparte con el cliente.' }}</p></section>
          </div>
          <section v-show="workspaceTab === 'history'" class="block" style="gap:18px"><h2 class="sec">Línea de tiempo<small>{{ repair.events.length }} avances</small></h2>
            <details class="event-composer">
              <summary>Agregar avance</summary>
              <form class="compose" @submit.prevent="addEvent">
                <div class="g2">
                  <div class="fld"><label for="stage">Etapa</label><select id="stage" v-model="stage"><option v-for="item in stages" :key="item">{{ item }}</option></select></div>
                  <div class="fld"><label for="event-date">Fecha y hora</label><input id="event-date" v-model="date" type="datetime-local" required /><button class="lnk" type="button" @click="date = now()">Ahora</button></div>
                </div>
                <div class="fld"><label for="event-title">¿Qué pasó?</label><input id="event-title" v-model="title" required placeholder="Ej. Cambio de pantalla terminado" /></div>
                <div class="fld"><label for="event-detail">Detalle (opcional)</label><textarea id="event-detail" v-model="detail" rows="2"></textarea></div>
                <div class="actions"><label><input v-model="publicEvent" type="checkbox" /> Mostrar al cliente en la web</label><button class="btn btn-p">Agregar a la línea de tiempo</button></div>
              </form>
            </details>
            <ol class="tl"><li v-for="event in repair.events" :key="event.id" class="ev"><div class="ev-t"><b>{{ event.date.slice(0,10) }}</b><span>{{ event.date.slice(11) }}</span></div><div class="ev-r"><span class="ev-dot">✓</span></div><div class="ev-c"><div class="ev-top"><h4>{{ event.title }}</h4><span class="pub"><i class="dot" :class="event.public ? 'd-amber' : 'd-slate'"></i>{{ event.public ? 'Pendiente de publicar' : 'Solo interno' }}</span></div><p>{{ event.detail }}</p></div></li></ol><p v-if="!repair.events.length" class="help">Agrega el primer avance de esta reparación.</p>
          </section>
        </div>
      </section>
      <aside class="aside" aria-label="Publicación, garantías y refacciones">
        <section class="card block"><h2 class="h3">Publicación en la web</h2><span class="pill amber" style="align-self:flex-start">Sin configurar</span><div class="seg"><button :class="{on:repair.syncMode === 'auto'}" @click="repair.syncMode = 'auto'">Automática</button><button :class="{on:repair.syncMode === 'manual'}" @click="repair.syncMode = 'manual'">Manual</button></div><p class="help">Los cambios se guardan en este equipo. La conexión con el portal web está pendiente.</p><button class="btn btn-p" disabled>Publicar cambios</button><div class="sep"></div><b>Qué ve el cliente</b><div v-for="option in repair.visibility" :key="option.label" class="swrow"><span>{{ option.label }}</span><button class="sw" :class="{on:option.enabled}" role="switch" :aria-checked="option.enabled" :aria-label="option.label" @click="option.enabled = !option.enabled"><span></span></button></div></section>
        <section class="block">
          <h2 class="sec">Garantías<button class="lnk" @click="addingWarranty = !addingWarranty">+ Agregar</button></h2>
          <p v-if="repair.warranties.length || addingWarranty" class="help">La garantía de esta orden empieza a contar el día de la entrega.</p>
          <div v-for="(item, i) in repair.warranties" :key="i" class="row"><div><div class="n">{{ item.name }}</div><div class="s">{{ item.days }} días de cobertura</div></div><span class="pill amber">{{ order.delivered ? 'Entrega registrada' : 'Empieza al entregar' }}</span></div>
          <p v-if="!repair.warranties.length" class="help">Sin garantías registradas.</p>
          <form v-if="addingWarranty" class="compose" @submit.prevent="addWarranty">
            <div class="fld"><label for="warranty-type">Tipo de garantía sugerida</label><select id="warranty-type" v-model="warrantyType" @change="days = warrantyType === 'labor' ? company.laborDays : company.partsDays"><option value="parts">Refacciones</option><option value="labor">Mano de obra</option></select></div>
            <div class="fld"><label for="warranty">Concepto</label><input id="warranty" v-model="warranty" required /></div>
            <div class="fld"><label for="days">Días de cobertura</label><input id="days" v-model.number="days" type="number" min="1" max="3650" required /></div>
            <p v-if="days === 0" class="help">Empresa no tiene días sugeridos para este tipo. Escribe los días acordados para registrar la garantía.</p>
            <button class="btn btn-p">Guardar garantía</button>
          </form>
        </section>
        <section class="block"><h2 class="sec">Refacciones usadas<button class="lnk" @click="addingPart = !addingPart">+ Agregar</button></h2><div v-for="(item, i) in repair.parts" :key="i" class="row"><div><div class="n">{{ item.name }}</div><div class="s">{{ money(item.price) }}</div></div><span class="pill teal">Registrada</span></div><p v-if="!repair.parts.length" class="help">Sin refacciones registradas.</p><form v-if="addingPart" class="compose" @submit.prevent="addPart"><div class="fld"><label for="part">Refacción</label><input id="part" v-model="part" required /></div><div class="fld"><label for="price">Costo</label><input id="price" v-model.number="price" type="number" min="0" step="0.01" required /></div><button class="btn btn-p">Guardar refacción</button></form></section>
      </aside>
    </div>
  </div>
</template>
<style scoped src="../../assets/styles/repairs.css"></style>
<style scoped>
.actions{display:flex;flex-wrap:wrap;align-items:center;justify-content:space-between;gap:12px}.note textarea{min-height:120px;padding:14px 16px;font-size:15.5px}.row{justify-content:space-between;flex-wrap:wrap}.work{display:grid;grid-template-columns: minmax(240px, .8fr) minmax(480px, 1.65fr) minmax(280px, 1fr)}.rail,.aside{min-width:0}.sheet{margin-left:8px}.steps{overflow:auto}.st-l{font-size:11px}.notice{margin:12px 28px;padding:12px 16px}
@media(max-width:1200px){.work{grid-template-columns:280px minmax(0,1fr)}.aside{grid-column:2;padding-left:8px}.sh-b,.sh-h{padding:24px}.num{font-size:36px}}
@media(max-width:760px){.work{display:flex;flex-direction:column}.rail,.aside{width:100%;padding:20px 16px}.sheet{width:calc(100% - 32px);margin:0 16px;flex:auto}.g2{grid-template-columns:1fr}.ev{grid-template-columns:70px 26px minmax(0,1fr);gap:8px}.st-l{font-size:10px}.st-c{width:28px;height:28px}}

.repair-page{min-width:0;flex:1}
.work{max-width:1920px;align-items:start;grid-template-columns:minmax(220px,.85fr) minmax(0,2fr) minmax(240px,.95fr)}
.rail,.aside{padding:16px;gap:16px}
.sheet{margin:16px 0;min-width:0;border-radius:16px}
.sh-h{padding:20px;gap:12px;align-items:center}
.num{font-size:32px}
.sh-b{padding:20px;gap:24px}
.repair-work{display:flex;flex-direction:column;gap:24px;min-width:0}
.workspace-tabs{display:flex;gap:8px;padding:16px 20px 0;flex-wrap:wrap}
.workspace-tabs .btn{min-height:40px;font-size:13px;padding-inline:14px}
.card{padding:16px;border-radius:12px}
.h3,.sec{font-size:17px}
.block{gap:10px;min-width:0}
.kv,.line{min-height:30px;font-size:13px;align-items:baseline;gap:8px}
.kv .k{flex-shrink:0}
.kv .v{overflow-wrap:anywhere;min-width:0}
.help{font-size:12px;line-height:1.5}
.fld input,.fld select{font-size:14px;min-height:38px}
.fld input:not([readonly]){background:var(--tint);padding-inline:8px}
.fld label{font-size:12px}
.swrow{font-size:13px;min-height:40px}
.aside .card{gap:10px}
.aside .swrow{min-height:36px}
.aside{gap:12px}
.perf::before,.perf::after{display:none}
.aside .seg button{font-size:12px;min-height:36px}
.aside .btn,.rail .btn{font-size:13px;min-height:38px;padding-inline:12px;white-space:normal}
.rail .compose{padding:12px;gap:12px}
.rail .block{gap:8px}
.steps{overflow:visible;gap:4px;padding-block:8px}
.step::before{top:17px}
.st-l{font-size:11px;overflow-wrap:anywhere;line-height:1.35}
.actions{gap:10px}
.actions>label{font-size:13px;display:flex;align-items:center;gap:8px}
.actions>label input{width:18px;height:18px;flex-shrink:0}
.event-composer{border:1px solid var(--line2);border-radius:10px}
.event-composer summary{padding:12px 16px;font-size:14px;font-weight:600;color:var(--acc-ink);cursor:pointer;min-height:44px}
.equipment-details{margin-block:4px 12px}
.equipment-details summary{padding-block:10px;font-size:13px;font-weight:600;color:var(--acc-ink);cursor:pointer;min-height:40px}
.event-composer .compose{border-top:1px solid var(--line);border-radius:0 0 10px 10px;padding:16px}
.ev{grid-template-columns:72px 28px minmax(0,1fr);gap:10px;padding-bottom:20px}
.ev-top{align-items:flex-start;gap:6px}
.ev-top h4{font-size:15px;overflow-wrap:anywhere}
.ev-c p{font-size:14px;white-space:pre-wrap;overflow-wrap:anywhere}
.ev-t b,.ev-t span,.pub{font-size:11px}
.pub{white-space:normal}
.ev-dot{width:28px;height:28px}
.row .n{font-size:14px;overflow-wrap:anywhere}
.pill{white-space:normal;height:auto;min-height:26px;padding-block:4px}

@media screen and (min-width:1400px){
  .rail,.aside{padding:20px;gap:20px}
  .sh-h,.sh-b{padding:24px}
  .kv,.line{min-height:32px}
  .note textarea{min-height:150px}
}
@media screen and (min-width:901px) and (max-width:1200px){
  .work{grid-template-columns:240px minmax(0,1fr)}
  .rail{grid-column:1;grid-row:1}
  .sheet{grid-column:2;grid-row:1;margin-right:16px}
  .aside{grid-column:1 / -1;display:grid;grid-template-columns:repeat(3,minmax(0,1fr));padding-top:0;align-items:start}
  .aside>.card{box-shadow:none;border:1px solid var(--line)}
}
@media screen and (max-width:900px){
  .work{display:flex;flex-direction:column}
  .rail{width:100%;display:grid;grid-template-columns:1fr 1fr;gap:16px;padding:16px}
  .rail>button{grid-column:1 / -1}
  .rail>.card,.rail>.block{min-width:0}
  .sheet{width:calc(100% - 32px);margin:0 16px;flex:auto}
  .aside{width:100%;display:grid;grid-template-columns:1fr 1fr;padding:16px}
  .aside>.card{grid-column:1 / -1}
  .aside .card .swrow{min-height:44px}
}
@media screen and (max-width:520px){
  .rail,.aside{display:flex;flex-direction:column;padding:16px}
  .rail>.card,.rail>.block,.aside>.block{width:100%}
  .sh-h,.sh-b{padding:16px}
  .num{font-size:28px}
  .sh-h .sh-act,.sh-h .btn{width:100%}
  .workspace-tabs{padding:16px 16px 0;gap:8px}
  .workspace-tabs .btn{flex:1;padding-inline:8px;min-height:44px}
  .steps{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:16px 8px}
  .step::before{display:none}
  .step{gap:6px}
  .st-l{font-size:12px}
  .actions>.btn{width:100%;white-space:normal}
  .fld input,.fld select,.fld textarea{font-size:16px}
  .btn,.rail .btn,.aside .btn,.seg button{min-height:44px}
  .ev{grid-template-columns:24px minmax(0,1fr);gap:8px}
  .ev-t{grid-column:2;flex-direction:row;text-align:left;gap:8px}
  .ev-r{grid-column:1;grid-row:1 / 3}
  .ev-c{grid-column:2}
  .ev-dot{width:24px;height:24px}
  .ev:not(:last-child) .ev-r::after{top:28px}
}
@media print{
  .work{display:block}.rail,.aside,.workspace-tabs,.event-composer{display:none}
  .sheet{width:100%;margin:0;box-shadow:none}
  .sh-b>section,.repair-work{display:flex!important}.sh-h .sh-act{display:none}
}
@media screen{
  .h3,.sec{font-size:20px}
  .num{font-size:36px}
  .subt{font-size:15px;line-height:1.5}
  .block{gap:12px}
  .kv,.line{min-height:36px;font-size:14px}
  .kv .v{font-size:15px}
  .help{font-size:14px;line-height:1.5}
  .fld label{font-size:13px}
  .fld input,.fld select{font-size:16px;min-height:44px}
  .note textarea{font-size:16px;line-height:1.5;min-height:150px}
  .btn,.rail .btn,.aside .btn{font-size:14px;min-height:44px}
  .aside .seg button,.seg button{font-size:14px;min-height:44px}
  .aside .swrow,.swrow{font-size:14px;min-height:44px}
  .workspace-tabs .btn{font-size:14px;min-height:44px}
  .st-l{font-size:12px}
  .ev{grid-template-columns:84px 30px minmax(0,1fr)}
  .ev-t b,.ev-t span,.pub{font-size:12px}
  .ev-top h4{font-size:17px}
  .ev-c p{font-size:16px;line-height:1.5}
  .row .n{font-size:16px}.row .s{font-size:14px}
  .pill{font-size:13px}
  .event-composer summary,.equipment-details summary{font-size:14px;min-height:44px}
}
@media screen and (max-width:520px){
  .ev{grid-template-columns:24px minmax(0,1fr)}
  .num{font-size:30px}
  .sec,.h3{font-size:19px}
}
@media screen and (min-width:1201px) and (max-height:800px){
  .sh-h,.sh-b{padding:14px 20px}
  .repair-work{gap:20px}
  .note textarea{min-height:136px}
  .aside{padding-block:12px}
  .aside .card{gap:8px;padding:14px}
}
</style>
