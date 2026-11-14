import { computed, ref, watch } from 'vue';
import { useWorkshopDataStore } from '../stores/useWorkshopDataStore.ts';
import type { CompanySettings, NoteKind } from '../types/company.ts';
import type { Order } from '../types/order.ts';
import { orderAmount } from '../utils/orderAmount.ts';
export function useCompanyNote(props:{ settings: CompanySettings; kind: NoteKind; order?: Order; folio?: string }){
const clauses = computed(() => props.settings.clauses.filter(c => props.kind === 'entry' ? c.entry : c.exit));
const total = computed(() => orderAmount(props.order?.cost || ''));
const emptySummary = () => ({technician:'', publicNote:'', warranties:[] as {name:string;days:number}[]});
const repairSummary = ref(emptySummary());
const summaryError = ref('');
let summaryRequest = 0;
async function prepare() {
  const request = ++summaryRequest;
  summaryError.value = '';
  const id = props.order?.databaseId;
  if (!id) {repairSummary.value=emptySummary(); return;}
  try {
  const summary = await useWorkshopDataStore().summary(id);
  if (request === summaryRequest && props.order?.databaseId === id) repairSummary.value = summary;
  } catch (cause) { if (request === summaryRequest) summaryError.value = 'No se pudo recuperar el seguimiento para la nota. Vuelve a intentar.'; throw cause; }
}
watch(() => props.order?.databaseId, () => {repairSummary.value=emptySummary();void prepare().catch(() => {});},{immediate:true});
const subtotal = computed(() => total.value / (1 + (Number(props.settings.taxRate) || 0) / 100));
const money = (value: number) => new Intl.NumberFormat(props.settings.locale, {style:'currency',currency:props.order?.currency ?? props.settings.currency}).format(value);
return {clauses,total,repairSummary,summaryError,prepare,subtotal,money};
}
