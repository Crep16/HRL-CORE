import { computed, onMounted, ref } from 'vue';
import { useWorkshopDataStore } from '../stores/useWorkshopDataStore.ts';
import { defaultStageOptions, includeRepairStage } from './useRepair.ts';
import type { Order } from '../types/order.ts';
import type { PaymentState } from '../types/payments.ts';
import { useRepairWorkspace } from './useRepairWorkspace.ts';
import { useWorkshopAccess } from './useWorkshopAccess.ts';
import { useCompany } from './useCompany.ts';
import { companyDateTime } from '../utils/companyDate.ts';
import { orderDisplayStatus } from '../utils/orderStatus.ts';
import {useAuthStore} from '../stores/useAuthStore.ts';
export function useRepairScreen(props:{ order: Order },emit:{(event:"back",...args:[]):void;(event:"saved",...args:[]):void;}){
const { repair, payment, save:persist, commit, message, busy:repairBusy, ready, dirty, reload } = useRepairWorkspace(props.order.databaseId!, () => emit('saved'));
const moneyBusy=ref(false);
async function save(){return moneyBusy.value||stageNameBusy.value?false:persist();}
const stageNameBusy=ref(false);
const busy=computed(()=>repairBusy.value||moneyBusy.value||stageNameBusy.value);
async function paymentsChanged(value: PaymentState) {
  payment.value = value;
  await reload();
  emit('saved');
}
const { can } = useWorkshopAccess();
const newNote = ref('');
const { company } = useCompany();
const stages = computed(() => repair.value.stages);
const savedStages = ref<string[]>([]);
const canManageStageNames=computed(()=>useAuthStore().can('repairs.edit','repairs',null));
const removableStageName=computed(()=>savedStages.value.includes(stage.value)&&![...defaultStageOptions,'Entrega'].includes(stage.value));
const stageAction=ref<{name:string;suggestion:boolean}>();
const stageActionError=ref('');
const customStage = ref(false);
const customStageName = ref('');
const stageOptions = computed(() => {
  const names = [...defaultStageOptions, ...savedStages.value];
  return names.filter((name, index) => names.findIndex(item => item.toLocaleLowerCase('es') === name.toLocaleLowerCase('es')) === index);
});
onMounted(async () => {
  try { savedStages.value = await useWorkshopDataStore().stages(props.order.databaseId!); }
  catch { message.value = 'No se pudieron cargar las etapas guardadas. Puedes usar las opciones predeterminadas o escribir una personalizada.'; }
});
const noteType = ref<'publicNote' | 'internalNote'>('publicNote');
const title = ref('');
const detail = ref('');
const stage = ref('Reparación');
function requestHideStageName(){
  if(!ready.value||busy.value||!canManageStageNames.value||!removableStageName.value)return;
  stageActionError.value='';stageAction.value={name:stage.value,suggestion:true};
}
async function confirmStageAction(){
  if(!stageAction.value||busy.value||!canManageStageNames.value)return;
  stageNameBusy.value=true;stageActionError.value='';
  try{
    savedStages.value=await useWorkshopDataStore().hideStageName(props.order.databaseId!,stageAction.value.name);
    stage.value='Reparación';stageAction.value=undefined;
    message.value='Nombre quitado del selector. Las etapas y el historial se conservan.';
  }catch(cause){stageActionError.value=cause&&typeof cause==='object'&&'message'in cause?String(cause.message):'No se pudo quitar el nombre. Vuelve a intentarlo.';}
  finally{stageNameBusy.value=false;}
}
const publicEvent = ref(true);
const workspaceTab = ref<'work' | 'history'>('work');
const now = () => companyDateTime(company.value.timezone);
const date = ref(now());
const addingWarranty = ref(false);
const warranty = ref('');
const warrantyType = ref<'parts' | 'labor'>('parts');
const days = ref(company.value.partsDays);
function selectWarrantyType(value: string) {
  warrantyType.value = value === 'labor' ? 'labor' : 'parts';
  days.value = warrantyType.value === 'labor' ? company.value.laborDays : company.value.partsDays;
}
const addingPart = ref(false);
const part = ref('');
const price = ref(0);
const money = (value: number) => new Intl.NumberFormat(company.value.locale, { style:'currency', currency:props.order.currency ?? company.value.currency }).format(value);
const nextEventId = () => repair.value.events.reduce((id, event) => Math.max(id, event.id), Date.now() - 1) + 1;
async function addEvent() {
  if (!title.value.trim() || !can('notes.add')) return;
  const name = (customStage.value ? customStageName.value : stage.value).trim();
  if (!name) { message.value = 'Selecciona o escribe una etapa.'; return; }
  try {
    if (await commit(next => {
      const selected = includeRepairStage(next, name);
      next.events.unshift({ id:nextEventId(), date:date.value, title:title.value.trim(), detail:detail.value, stage:selected, public:publicEvent.value });
    })) {
      savedStages.value.push(name); stage.value = name; title.value = ''; detail.value = '';
      customStage.value = false; customStageName.value = '';
    }
  } catch (cause) { message.value = cause instanceof Error ? cause.message : 'No se pudo agregar la etapa.'; }
}
async function complete() {
  if (repair.value.completed >= stages.value.length || !can('stages.advance')) return;
  await commit(next => { next.events.unshift({ id:nextEventId(), date:now(), title:`${next.stages[next.completed]}: etapa completada`, detail:'', stage:next.stages[next.completed]!, public:true }); next.completed++; });
}
async function addWarranty() {
  if (!warranty.value.trim() || !can('warranties.add')) return;
  if (await commit(next => next.warranties.push({ name:warranty.value.trim(), days:days.value }))) { warranty.value=''; addingWarranty.value=false; }
}
async function addPart() {
  if (!part.value.trim() || !can('parts.add')) return;
  if (await commit(next => next.parts.push({ name:part.value.trim(), price:price.value }))) { part.value=''; price.value=0; addingPart.value=false; }
}
async function appendNote() {
  if (!newNote.value.trim() || !can('notes.add')) return;
  const text = newNote.value.trim();
  if (await commit(next => { const old=next[noteType.value]; next[noteType.value] = old ? `${old}\n${text}` : text; })) newNote.value='';
}
async function back() { if (await save()) emit('back'); }
return {stageAction,stageActionError,canManageStageNames,removableStageName,requestHideStageName,confirmStageAction,repair,payment,save,message,ready,dirty,reload,moneyBusy,busy,can,newNote,company,stages,customStage,customStageName,stageOptions,noteType,title,detail,stage,publicEvent,workspaceTab,now,date,addingWarranty,warranty,warrantyType,days,selectWarrantyType,addingPart,part,price,money,addEvent,complete,addWarranty,addPart,appendNote,back,orderDisplayStatus,paymentsChanged};
}
