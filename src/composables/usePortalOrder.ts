import {computed,onMounted,onBeforeUnmount,ref} from 'vue';
import {usePortalStore} from '../stores/usePortalStore.ts';
import {useAuthStore} from '../stores/useAuthStore.ts';
import {normalizeIpcError} from '../services/api/errors.ts';
import type {PublicationAction} from '../types/portal.ts';
import {useCompany} from './useCompany.ts';
export function usePortalOrder(id:number,beforeAction:()=>Promise<boolean>){
  const store=usePortalStore(),auth=useAuthStore(),error=ref(''),pending=ref<PublicationAction|null>(null);
  const current=computed(()=>store.statuses[id]),working=ref(false),canEdit=computed(()=>auth.can('publication.edit','repairs',id));
  const {company}=useCompany();
  const confirmation=computed(()=>{const at=current.value?.confirmedAt;if(!at)return '';const date=new Date(at);return Number.isNaN(date.getTime())?'':new Intl.DateTimeFormat(company.value.locale,{dateStyle:'short',timeStyle:'short',timeZone:company.value.timezone}).format(date);});
  const label=computed(()=>!current.value?'Consultando…':!current.value.configured?'Sin configurar':({unpublished:'Sin publicar',pending:current.value.enabled?'Pendiente de enviar':'Retiro pendiente',sending:'Enviando',confirmed:current.value.enabled?'Publicado':'Retirado',superseded:'Revisión superada',error:'Error de publicación',conflict:'Conflicto'})[current.value.state]);
  let timer:ReturnType<typeof setInterval>|undefined,alive=true,refreshing=false;
  async function refresh(){
    if(!alive||refreshing||working.value)return;
    refreshing=true;const version=auth.sessionVersion;
    try{await store.status(id);if(alive&&version===auth.sessionVersion)error.value='';}
    catch(cause){if(alive&&version===auth.sessionVersion)error.value=normalizeIpcError(cause).message;}
    finally{refreshing=false;}
  }
  async function execute(action:PublicationAction){
    if(working.value||!canEdit.value)return;working.value=true;error.value='';const version=auth.sessionVersion;
    try{if(!await beforeAction()||!alive||version!==auth.sessionVersion)return;await store.publication(id,action);if(alive)pending.value=null;}
    catch(cause){if(alive)error.value=normalizeIpcError(cause).message;}
    finally{working.value=false;}
  }
  function request(action:PublicationAction){if(action==='publish')void execute(action);else pending.value=action;}
  onMounted(()=>{void refresh();timer=setInterval(()=>{if(!working.value&&auth.isAuthenticated)void refresh();},3000);});
  onBeforeUnmount(()=>{alive=false;if(timer)clearInterval(timer);});
  return {current,working,canEdit,label,confirmation,error,pending,refresh,execute,request};
}
