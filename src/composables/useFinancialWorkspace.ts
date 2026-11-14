import { computed,onMounted,reactive,ref,watch } from 'vue';
import { useWorkshopDataStore } from '../stores/useWorkshopDataStore.ts';
import { useCompany } from './useCompany.ts';
import { useWorkshopAccess } from './useWorkshopAccess.ts';
import type { FinancialFilters,FinancialOrder } from '../types/finances.ts';
import { financialClasses } from '../data/financialClasses.ts';
import { filterFinancialOrders, financialTotals } from '../models/finances.ts';
import { calendarDate } from '../utils/finances.ts';
export function useFinancialWorkspace(){
const {company}=useCompany();
const {can}=useWorkshopAccess();
const orders=ref<FinancialOrder[]>([]),busy=ref(false),error=ref(''),message=ref('');
const filters=reactive<FinancialFilters>({query:'',classification:'all',dateBasis:'received',from:'',to:'',scope:'active'});
const page=ref(0),selectedId=ref('');
const validRange=computed(()=>!filters.from||!filters.to||filters.from<=filters.to);
const filtered=computed(()=>validRange.value?filterFinancialOrders(orders.value,filters,company.value.timezone):[]);
const totals=computed(()=>financialTotals(filtered.value,filters,company.value.timezone));
const visible=computed(()=>filtered.value.slice(page.value*50,(page.value+1)*50));
const selected=computed(()=>visible.value.find(order=>order.id===selectedId.value)??visible.value[0]);
const cost=reactive({description:'',amount:0,reason:''});
watch(()=>selected.value?.id,()=>{Object.assign(cost,{description:'',amount:0,reason:''});message.value='';});
watch(()=>filtered.value.length,length=>{page.value=Math.min(page.value,Math.max(0,Math.ceil(length/50)-1));});
watch(filters,()=>{page.value=0;},{deep:true});
function money(value:number,currency:string){return new Intl.NumberFormat(company.value.locale,{style:'currency',currency}).format(value/100);}
function date(value:string|null){return calendarDate(value,company.value.timezone)||'Sin fecha disponible';}
async function load(){if(busy.value)return;busy.value=true;error.value='';try{orders.value=await useWorkshopDataStore().finances();}catch(cause){orders.value=[];error.value=typeof cause==='string'?cause:'No se pudo consultar el resumen financiero.';}finally{busy.value=false;}}
async function addCost(){if(!selected.value||busy.value)return;const id=selected.value.databaseId!;const amountMinor=Math.round(cost.amount*100);if(!Number.isFinite(cost.amount)||amountMinor<=0||Math.abs(cost.amount*100-amountMinor)>.0001){message.value='El importe debe ser positivo y tener hasta dos decimales.';return;}busy.value=true;message.value='';try{await useWorkshopDataStore().expense(id,cost.description,amountMinor,cost.reason);Object.assign(cost,{description:'',amount:0,reason:''});message.value='Costo directo registrado y agregado al historial.';}catch(cause){message.value=String(cause);}finally{busy.value=false;}await load();}
onMounted(load);
return {can,busy,error,message,filters,page,selectedId,validRange,filtered,totals,visible,selected,cost,money,date,load,addCost,financialClasses};
}
