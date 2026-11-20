import {computed,ref,watch} from 'vue';
import {defineStore} from 'pinia';
import {portalApi} from '../services/api/portal.ts';
import {useAuthStore} from './useAuthStore.ts';
import {normalizeIpcError} from '../services/api/errors.ts';
import {initialPortalConfig,type PortalConfig,type PortalSettings,type PortalOrderStatus,type PublicationAction} from '../types/portal.ts';

const editable=(v:PortalConfig)=>JSON.stringify([v.projectId,v.apiKey,v.publisherUid,v.portalUrl]);
export const usePortalStore=defineStore('portal',()=>{
  const auth=useAuthStore(),settings=ref<PortalSettings|null>(null),draft=ref(initialPortalConfig());
  const statuses=ref<Record<number,PortalOrderStatus>>({}),error=ref(''),active=ref(0);
  const statusRequests=new Map<number,number>(),changingOrders=new Set<number>();
  let settingsRequest=0;
  const stale=()=>({code:'CONFLICT',message:'El estado cambió durante la consulta. Vuelve a actualizarlo.'});
  const nextStatusRequest=(id:number)=>{const next=(statusRequests.get(id)??0)+1;statusRequests.set(id,next);return next;};
  const busy=computed(()=>active.value>0);
  const dirty=computed(()=>settings.value!==null&&editable(draft.value)!==editable(settings.value.config));
  const canManage=computed(()=>auth.user?.accessLevel==='complete'&&auth.can('company.edit','company',null));
  watch(()=>auth.sessionVersion,()=>{settingsRequest++;statusRequests.clear();changingOrders.clear();settings.value=null;statuses.value={};draft.value=initialPortalConfig();error.value='';active.value=0;},{flush:'sync'});
  async function run<T>(operation:()=>Promise<T>,mutation=false):Promise<T>{
    const version=auth.sessionVersion;if(mutation){active.value++;error.value='';}
    try{const result=await operation();if(version!==auth.sessionVersion)throw {code:'SESSION_CHANGED',message:'La sesión cambió durante la operación.'};return result;}
    catch(cause){const result=normalizeIpcError(cause);if(version===auth.sessionVersion){if(mutation)error.value=result.message;if(result.code==='UNAUTHENTICATED')auth.clearSession();}throw result;}
    finally{if(mutation&&version===auth.sessionVersion)active.value--;}
  }
  async function loadSettings(preserveDraft=false){
    const request=++settingsRequest,before=editable(draft.value);
    const replace=!preserveDraft&&(settings.value===null||!dirty.value);
    const result=await run(portalApi.settings,true);
    if(request!==settingsRequest)throw stale();
    settings.value=result;
    if(replace&&editable(draft.value)===before)draft.value=result.config.projectId?{...result.config}:initialPortalConfig();return result;
  }
  async function status(id:number){
    if(changingOrders.has(id))throw stale();
    const request=nextStatusRequest(id),result=await run(()=>portalApi.status(id));
    if(request!==statusRequests.get(id)||changingOrders.has(id))throw stale();
    statuses.value[id]=result;return result;
  }
  async function configure(){
    const value={...draft.value};settingsRequest++;
    await run(()=>portalApi.configure(value),true);
    const preserve=editable(draft.value)!==editable(value);settings.value=null;
    await loadSettings(preserve);
  }
  async function connect(email:string,password:string){settingsRequest++;await run(()=>portalApi.connect(email,password),true);await loadSettings();}
  async function control(action:'pause'|'resume'|'disconnect'){settingsRequest++;await run(portalApi[action],true);await loadSettings();}
  async function publication(id:number,action:PublicationAction){
    if(changingOrders.has(id))throw stale();
    const version=auth.sessionVersion;changingOrders.add(id);nextStatusRequest(id);
    try{const result=await run(()=>portalApi.action(id,action),true);statuses.value[id]=result;return result;}
    finally{if(version===auth.sessionVersion){nextStatusRequest(id);changingOrders.delete(id);}}
  }
  return {settings,draft,statuses,error,busy,dirty,canManage,loadSettings,status,configure,connect,control,publication};
});
