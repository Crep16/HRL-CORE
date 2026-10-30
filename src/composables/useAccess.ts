import { readonly, ref } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';

export type AccessUser = { id: string; name: string; accessLevel: 'complete' | 'limited' | 'custom' };
type AccessStatus = { setupComplete: boolean; user: AccessUser | null };
export type FirstSetup = { name: string; password: string; confirmation: string; masterPassword: string; masterConfirmation: string };
const user = ref<AccessUser | null>(null);
const setupComplete = ref(false);
const loading = ref(false);
const error = ref('');
const permissions = ref<string[]>([]);
const available = isTauri();
export type TemporaryGrant={id:string;recipientId:string;approverId:string;approverName:string;scope:string;recordId:string|null;operations:string[]};
export type AccessRequest={operation:string;scope:string;recordId:string|null};
const temporary=ref<TemporaryGrant|null>(null);
const accessRequest=ref<AccessRequest|null>(null);
const context=ref({scope:'clients',recordId:null as string|null});
let answer:((allowed:boolean)=>void)|undefined;
const canFor=(operation:string,scope:string,recordId:string|null)=>permissions.value.includes(operation)||!!(temporary.value&&temporary.value.recipientId===user.value?.id&&temporary.value.scope===scope&&(temporary.value.recordId===null||temporary.value.recordId===recordId)&&temporary.value.operations.includes(operation));

async function run<T>(operation: () => Promise<T>): Promise<T | undefined> {
  if (loading.value) return;
  error.value = '';
  if (!available) { error.value = 'Abre la aplicación de escritorio para configurar o usar las cuentas de este equipo.'; return; }
  loading.value = true;
  try { return await operation(); }
  catch (cause) { error.value = typeof cause === 'string' ? cause : 'No se pudo completar el acceso. Vuelve a intentar.'; }
  finally { loading.value = false; }
}

export function useAccess() {
  async function refresh() {
    await run(async () => {
      const status = await invoke<AccessStatus>('access_status');
      permissions.value = status.user ? await invoke<string[]>('access_permissions') : [];
      user.value = status.user; setupComplete.value = status.setupComplete;
      if(temporary.value?.recipientId!==status.user?.id)temporary.value=null;
    });
  }
  async function firstSetup(input: FirstSetup) {
    return run(async () => { const account = await invoke<AccessUser>('first_setup', { input }); permissions.value = await invoke<string[]>('access_permissions'); user.value = account; setupComplete.value = true; return true; });
  }
  async function login(name: string, password: string) {
    return run(async () => { const account = await invoke<AccessUser>('login', { name, password }); permissions.value = await invoke<string[]>('access_permissions'); user.value = account; return true; });
  }
  async function logout() {
    return run(async () => { await invoke('logout'); temporary.value=null;user.value = null; permissions.value = []; return true; });
  }
  const can = (operation: string) => canFor(operation,context.value.scope,context.value.recordId);
  function requestAccess(operation:string,scope=context.value.scope,recordId=context.value.recordId,force=false):Promise<boolean>{
    if(!force&&canFor(operation,scope,recordId))return Promise.resolve(true);
    answer?.(false);accessRequest.value={operation,scope,recordId};
    return new Promise(resolve=>{answer=resolve;});
  }
  function resolveRequest(allowed:boolean){const resolve=answer;answer=undefined;accessRequest.value=null;resolve?.(allowed);}
  async function approveTemporary(name:string,password:string,reason:string,recordId:string|null){
    const requested=accessRequest.value;if(!requested)throw new Error('La solicitud ya no está abierta.');
    const grant=await invoke<TemporaryGrant>('grant_temporary_access',{input:{...requested,recordId,name,password,reason}});
    if(accessRequest.value!==requested||user.value?.id!==grant.recipientId)throw new Error('La solicitud o la sesión cambió. No se habilitó la ventana.');
    temporary.value=grant;resolveRequest(true);return grant;
  }
  async function endTemporary(){temporary.value=null;if(available)await invoke('end_temporary_access');}
  function setContext(scope:string,recordId:string|null=null){context.value={scope,recordId};}
  const baseCan=(operation:string)=>permissions.value.includes(operation);
  return { user: readonly(user), setupComplete: readonly(setupComplete), loading: readonly(loading), error: readonly(error), can, canFor, available, refresh, firstSetup, login, logout,
    temporary:readonly(temporary),accessRequest:readonly(accessRequest),requestAccess,resolveRequest,approveTemporary,endTemporary,setContext,baseCan };
}
