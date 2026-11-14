import { computed, onMounted, reactive, ref } from 'vue';
import { useWorkshopDataStore } from '../stores/useWorkshopDataStore.ts';
import { utcDayBoundary } from '../utils/finances.ts';
import { useCompany } from './useCompany.ts';
import type { LogbookEntry, LogbookFilters, LogbookPage } from '../types/logbook.ts';
import { entryChanges, logbookFields, logbookOperations, logbookValue } from '../utils/logbook.ts';
export function useLogbookWorkspace(){
const { company } = useCompany();
const tab=ref<'history'|'finances'>('history');
const filters = reactive({query:'', operation:'', authorId:'',orderId:'',from:'',to:''});
const applied = ref({...filters});
const result = ref<LogbookPage>({entries:[],total:0,page:0,pageSize:50,authors:[]});
const selected = ref<LogbookEntry>();
const entityNames: Record<string, string> = {
  ordenes: 'Orden', clientes: 'Cliente', equipos: 'Equipo', abonos: 'Abono',
  ajustes_abono: 'Ajuste de abono', devoluciones: 'Devolución', gastos_directos: 'Gasto directo',
  empresa: 'Empresa', usuarios: 'Usuario', evidencias: 'Evidencia', seguimientos: 'Seguimiento',
  accesos_temporales: 'Acceso temporal',
};
const recordIdentity = computed(() => {
  const entry = selected.value;
  if (!entry?.entity) return entry?.orderId || 'Sin identificar';
  const name = entityNames[entry.entity] ?? entry.entity;
  return entry.entityId ? `${name} #${entry.entityId}` : name;
});
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
    result.value = await useWorkshopDataStore().logbook({...rest,page,fromUtc:utcDayBoundary(from,company.value.timezone),toUtc:utcDayBoundary(to,company.value.timezone,true)} satisfies LogbookFilters);
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
return {company,tab,filters,result,selected,recordIdentity,busy,error,changes,pages,hasFilters,load,apply,clear,orderHistory,date,description,logbookFields,logbookOperations,logbookValue};
}
