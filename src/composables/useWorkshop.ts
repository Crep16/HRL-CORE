import { computed, nextTick, onBeforeUnmount, onMounted, ref, useTemplateRef } from 'vue';
import { useAuthStore } from '../stores/useAuthStore.ts';
import { useOrderStore } from '../stores/useOrderStore.ts';
import { useWorkspaceStore } from '../stores/useWorkspaceStore.ts';
import { useCompanyStore } from '../stores/useCompanyStore.ts';
import { useCompany } from './useCompany.ts';
import { legacyOrder } from '../utils/legacyOrder.ts';
import { printCompanyNote } from '../utils/printCompanyNote.ts';
import type { AccessScope } from '../types/ipc.ts';
import type { NoteKind } from '../types/company.ts';
import type ReceptionView from '../views/ReceptionView.vue';
import type RepairsView from '../views/RepairsView.vue';
import type CompanyNote from '../components/company/CompanyNote.vue';
const labels:Record<AccessScope,string>={clients:'Clientes',repairs:'Reparaciones',logbook:'Bitácora',company:'Empresa',users:'Usuarios'};
export function useWorkshop(){
  const auth=useAuthStore(), orders=useOrderStore(), workspace=useWorkspaceStore(), companyStore=useCompanyStore();
  const {company}=useCompany();
  const reception=useTemplateRef<InstanceType<typeof ReceptionView>>('reception');
  const repair=useTemplateRef<InstanceType<typeof RepairsView>>('repair');
  const printNote=useTemplateRef<InstanceType<typeof CompanyNote>>('printNote');
  const printSource=useTemplateRef<HTMLElement>('printSource');
  const notice=ref(''),companyDirty=ref(false),printing=ref(false),printKind=ref<NoteKind>('entry');
  const selected=computed(()=>orders.currentOrder?legacyOrder(orders.currentOrder):undefined);
  const can=(page:AccessScope)=>auth.can(page==='clients'?'clients.view':page==='users'?'users.manage':`${page}.view`,page,page==='repairs'?orders.currentOrder?.id:null);
  const tabs=computed(()=>Object.entries(labels).filter(([page])=>page==='clients'||can(page as AccessScope)).map(([,label])=>label));
  const active=computed(()=>labels[workspace.page]);
  const canRepair=computed(()=>!!selected.value&&!reception.value?.editing&&!reception.value?.trashed&&!orders.isLoading);
  const canPrint=computed(()=>!!selected.value&&!reception.value?.editing&&!reception.value?.trashed&&!companyDirty.value&&!orders.isLoading&&!printing.value&&companyStore.loaded&&(can('clients')||workspace.page==='repairs'&&can('repairs')));
  const publicOrder=computed(()=>can('clients')||workspace.page==='repairs'&&can('repairs')?selected.value:undefined);
  async function loadCompany(){try{await companyStore.load();}catch{notice.value=companyStore.error?.message??'No se pudo consultar Empresa.';}}
  onMounted(()=>{workspace.page='clients';void loadCompany();});
  async function prepareLeave(){
    if(printing.value||orders.isLoading||auth.isLoading)return false;
    if(companyDirty.value){notice.value='Guarda o descarta los cambios de Empresa antes de salir.';return false;}
    if(workspace.page==='clients'&&reception.value&&!await reception.value.prepareLeave())return false;
    if(workspace.page==='repairs'&&repair.value&&!await repair.value.save())return false;
    return true;
  }
  async function navigate(label:string){
    const page=Object.entries(labels).find(([,value])=>value===label)?.[0] as AccessScope|undefined;
    if(!page||page===workspace.page||!await prepareLeave())return;
    if(page==='repairs'&&!selected.value){notice.value='Selecciona una orden para abrir su gestión.';return;}
    try{await auth.finishTemporaryAccess();}catch{return;}
    if(page==='company')await loadCompany();
    workspace.page=page;notice.value='';
    await nextTick();document.querySelector<HTMLElement>('main h1, .repair-page h1, .users-heading h1')?.focus();
  }
  async function logout(){if(await prepareLeave()){try{await auth.logoutUser();}catch{notice.value=auth.error?.message??'No se pudo cerrar la sesión.';}}}
  async function finishTemporary(){if(!await prepareLeave())return;try{await auth.finishTemporaryAccess();}catch{notice.value=auth.error?.message??'';}}
  async function refreshOrder(){const id=orders.currentOrder?.id;if(!id)return;try{await orders.getOrder({orderId:id,scope:workspace.page==='repairs'?'repairs':'clients'});}catch{notice.value=orders.error?.message??'';}}
  async function print(kind:NoteKind){
    if(!canPrint.value||!await prepareLeave())return;
    printing.value=true;notice.value='';printKind.value=kind;
    try{await nextTick();await printNote.value?.prepare();await nextTick();const note=printSource.value?.querySelector<HTMLElement>('.company-note');if(note)await printCompanyNote(note,company.value);}
    catch(cause){notice.value=cause instanceof Error?cause.message:'No se pudo preparar la nota para impresión.';}finally{printing.value=false;}
  }
  function requestAccess(){const page=workspace.page;void workspace.requestAccess(page==='clients'?'orders.create':page==='repairs'?'notes.add':page==='company'?'company.edit':page==='users'?'users.manage':'logbook.view',page,page==='repairs'?orders.currentOrder?.id:null,true);}
  function receptionTool(action:'lookup'|'history'|'refresh') {
    if (action === 'lookup') reception.value?.showLookup();
    else if (action === 'history') reception.value?.openHistory();
    else void reception.value?.refreshList(false);
  }
  onBeforeUnmount(()=>workspace.finishRequest());
  return {receptionTool,auth,orders,workspace,companyStore,company,companyDirty,notice,selected,publicOrder,tabs,active,can,canRepair,canPrint,printKind,printing,navigate,logout,finishTemporary,refreshOrder,print,requestAccess,prepareLeave,loadCompany};
}
