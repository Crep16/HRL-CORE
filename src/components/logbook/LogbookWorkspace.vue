<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import SearchSelect from '../ui/SearchSelect.vue';
import AppNotice from '../ui/AppNotice.vue';
import FinancialWorkspace from './FinancialWorkspace.vue';
import { utcDayBoundary } from '../../utils/finances';
import { useCompany } from '../../composables/useCompany';
import type { LogbookEntry, LogbookFilters, LogbookPage } from '../../types/logbook';
import { entryChanges, logbookFields, logbookOperations, logbookValue } from '../../utils/logbook';

const { company } = useCompany();
const tab=ref<'history'|'finances'>('history');
const filters = reactive({query:'', operation:'', authorId:'',orderId:'',from:'',to:''});
const applied = ref({...filters});
const result = ref<LogbookPage>({entries:[],total:0,page:0,pageSize:50,authors:[]});
const selected = ref<LogbookEntry>();
const busy = ref(false);
const error = ref('');
const changes = computed(() => selected.value ? entryChanges(selected.value) : []);
const pages = computed(() => Math.max(1,Math.ceil(result.value.total/result.value.pageSize)));
const hasFilters = computed(() => Object.values(applied.value).some(Boolean));
async function load(page = result.value.page) {
  if (busy.value) return;
  busy.value = true; error.value = '';
  try {
    const current = selected.value?.id;
    if(applied.value.from&&applied.value.to&&applied.value.from>applied.value.to)throw new Error('La fecha inicial debe ser anterior o igual a la final.');
    const {from,to,...rest}=applied.value;
    result.value = await invoke<LogbookPage>('list_logbook',{filters:{...rest,page,fromUtc:utcDayBoundary(from,company.value.timezone),toUtc:utcDayBoundary(to,company.value.timezone,true)} satisfies LogbookFilters});
    selected.value = result.value.entries.find(entry => entry.id === current) ?? result.value.entries[0];
  } catch (cause) {
    result.value = {...result.value,entries:[],total:0,page:0,authors:[]}; selected.value = undefined;
    error.value = typeof cause === 'string' ? cause : cause instanceof Error?cause.message:'No se pudo consultar la bitácora. Vuelve a intentar.';
  } finally { busy.value = false; }
}
function apply() { applied.value = {...filters}; void load(0); }
function clear() { Object.assign(filters,{query:'',operation:'',authorId:'',orderId:'',from:'',to:''}); apply(); }
function orderHistory(orderId:string){Object.assign(filters,{query:'',operation:'',authorId:'',orderId,from:'',to:''});tab.value='history';apply();}
function date(value: string | null) {
  if (!value) return 'Sin fecha de registro';
  const parsed = new Date(value);
  return Number.isNaN(parsed.getTime()) ? value : new Intl.DateTimeFormat(company.value.locale,{timeZone:company.value.timezone,dateStyle:'medium',timeStyle:'short'}).format(parsed);
}
function description(entry: LogbookEntry) {
  return entryChanges(entry).map(change => logbookFields[change.field] ?? 'Campo actualizado').join(' · ') || 'Operación del sistema';
}
onMounted(() => load(0));
</script>

<template>
  <main class="logbook-page" aria-labelledby="logbook-title">
    <header class="logbook-heading">
      <div><h1 id="logbook-title">Bitácora interna</h1><p>Consulta quién realizó cada operación y qué cambió en el taller.</p></div>
      <button type="button" class="btn btn-s" :disabled="busy" @click="load()">Actualizar registros</button>
    </header>
    <nav class="logbook-section-tabs" aria-label="Apartados de Bitácora"><button class="btn" :class="tab==='history'?'btn-p':'btn-s'" :aria-pressed="tab==='history'" @click="tab='history'">Historial de cambios</button><button class="btn" :class="tab==='finances'?'btn-p':'btn-s'" :aria-pressed="tab==='finances'" @click="tab='finances'">Resumen financiero de órdenes</button></nav>
    <FinancialWorkspace v-if="tab==='finances'" @history="orderHistory" />
    <template v-else>
    <p v-if="filters.orderId" class="logbook-order-context">Historial de {{ filters.orderId }} <button class="btn btn-s" @click="clear">Consultar todas las órdenes</button></p>
    <form class="logbook-filters" aria-label="Filtros de la bitácora" @submit.prevent="apply">
      <div class="fld"><label for="logbook-search">Buscar por folio o usuario</label><input id="logbook-search" v-model="filters.query" type="search" maxlength="100" placeholder="Ej. ORD-015 o nombre del usuario" :disabled="busy" /></div>
      <SearchSelect id="logbook-operation" v-model="filters.operation" label="Operación" :disabled="busy" :options="[{value:'',label:'Todas las operaciones'},...Object.entries(logbookOperations).map(([value,label])=>({value,label}))]" />
      <SearchSelect id="logbook-author" v-model="filters.authorId" label="Usuario" :disabled="busy" :options="[{value:'',label:'Todos los usuarios'},...result.authors.map(author=>({value:author.id,label:author.name}))]" />
      <div class="fld"><label for="history-from">Registrado desde</label><input id="history-from" v-model="filters.from" type="date" :disabled="busy" :max="filters.to||undefined" /></div>
      <div class="fld"><label for="history-to">Registrado hasta</label><input id="history-to" v-model="filters.to" type="date" :disabled="busy" :min="filters.from||undefined" /></div>
      <div class="filter-actions"><button class="btn btn-p" :disabled="busy">Buscar</button><button type="button" class="btn btn-s" :disabled="busy" @click="clear">Limpiar</button></div>
    </form>
    <AppNotice :message="error" tone="error"><button class="btn btn-s" type="button" :disabled="busy" @click="load()">Reintentar</button></AppNotice>
    <AppNotice v-if="busy" message="Consultando los movimientos del taller…" :closable="false" />
    <div class="logbook-layout" :aria-busy="busy">
      <section class="logbook-records" aria-labelledby="logbook-records-title">
        <header class="records-heading"><h2 id="logbook-records-title">Movimientos</h2><span role="status">{{ result.total }} {{ result.total === 1 ? 'registro' : 'registros' }}</span></header>
        <div v-if="result.entries.length" class="logbook-table-wrap">
          <table class="logbook-table"><caption class="sr">Movimientos de la bitácora. Selecciona una operación para consultar sus valores.</caption>
            <thead><tr><th scope="col">Fecha de registro</th><th scope="col">Operación y orden</th><th scope="col">Usuario</th></tr></thead>
            <tbody><tr v-for="entry in result.entries" :key="entry.id" :class="{selected:selected?.id === entry.id}">
              <td><time v-if="entry.occurredAt" :datetime="entry.occurredAt">{{ date(entry.occurredAt) }}</time><span v-else>Sin fecha de registro</span></td>
              <td><button type="button" class="logbook-select" :aria-pressed="selected?.id === entry.id" :aria-label="`${logbookOperations[entry.operation] ?? 'Operación registrada'} · ${entry.orderId} · Ver detalle`" @click="selected=entry">{{ logbookOperations[entry.operation] ?? 'Operación registrada' }}</button><span class="record-order">{{ entry.orderId }}</span><span class="record-description">{{ description(entry) }}</span></td>
              <td>{{ entry.authorName }}</td>
            </tr></tbody>
          </table>
        </div>
        <div v-else-if="!busy && !error" class="logbook-empty"><h3>{{ hasFilters ? 'No hay coincidencias' : 'Todavía no hay movimientos' }}</h3><p>{{ hasFilters ? 'Prueba con otro folio, usuario u operación.' : 'Las operaciones del taller aparecerán aquí al guardar órdenes, avances y abonos.' }}</p><button v-if="hasFilters" class="btn btn-s" type="button" @click="clear">Mostrar todos los movimientos</button></div>
        <nav v-if="result.total" class="logbook-pagination" aria-label="Páginas de movimientos"><span>Página {{ result.page + 1 }} de {{ pages }}</span><div><button class="btn btn-s" type="button" :disabled="busy || result.page === 0" @click="load(result.page-1)">Anterior</button><button class="btn btn-s" type="button" :disabled="busy || result.page+1 >= pages" @click="load(result.page+1)">Siguiente</button></div></nav>
      </section>
      <aside class="logbook-detail" aria-labelledby="logbook-detail-title">
        <h2 id="logbook-detail-title">Detalle del movimiento</h2>
        <template v-if="selected">
          <h3>{{ logbookOperations[selected.operation] ?? 'Operación registrada' }}</h3>
          <dl class="logbook-metadata"><div><dt>Registro</dt><dd>{{ selected.orderId }}</dd></div><div><dt>Realizado por</dt><dd>{{ selected.authorName }}</dd></div><div><dt>Registrado el</dt><dd>{{ date(selected.occurredAt) }}</dd></div><div><dt>Resultado</dt><dd>{{ selected.result==='error'?'No completado':'Operación confirmada' }}</dd></div><div><dt>Motivo</dt><dd>{{ selected.reason||'No registrado para esta operación' }}</dd></div><div v-if="selected.eventDate"><dt>Fecha indicada para el abono</dt><dd>{{ selected.eventDate.replace('T',' ') }}</dd></div></dl>
          <p v-if="!selected.occurredAt" class="logbook-historical">Este abono es anterior a la bitácora. Su fecha de registro no estaba disponible.</p>
          <section v-for="(change,index) in changes" :key="index" class="logbook-change">
            <h4>{{ logbookFields[change.field] ?? 'Campo actualizado' }}</h4>
            <div class="change-values"><div><h5>Antes</h5><p>{{ logbookValue(change.field,change.oldValue,selected.currency,company.locale) }}</p></div><div><h5>Después</h5><p>{{ logbookValue(change.field,change.newValue,selected.currency,company.locale) }}</p></div></div>
          </section>
          <p v-if="!changes.length" class="logbook-historical">{{ selected.details === null && selected.operation !== 'orders.delete' && selected.operation !== 'orders.restore' ? 'Este registro anterior conserva la operación, el usuario y la fecha; no guardaba el detalle de los valores.' : selected.operation === 'orders.delete' ? 'La orden se envió a la papelera. Se conservan su historial y sus cobros.' : selected.operation === 'orders.restore' ? 'La orden volvió al listado de órdenes activas.' : 'La operación no registró cambios de campos.' }}</p>
        </template>
        <p v-else class="logbook-historical">Selecciona un movimiento para revisar su información.</p>
        <footer>Registro interno de consulta. Los movimientos no se pueden editar ni borrar.</footer>
      </aside>
    </div>
    </template>
  </main>
</template>

<style scoped src="../../assets/styles/logbook.css"></style>
