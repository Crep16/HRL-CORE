import { defineStore } from 'pinia';
import { ref,watch,computed } from 'vue';
import { backupApi, type BackupStatus } from '../services/api/backups.ts';
import { normalizeIpcError } from '../services/api/errors.ts';
import { useAuthStore } from './useAuthStore.ts';

export const useBackupStore=defineStore('backups',()=>{
  const auth=useAuthStore();
  const open=ref(false),busy=ref(false),unlocked=ref(false),restoreMode=ref(false),restored=ref(false);
  const masterPassword=ref(''),backupPassword=ref(''),error=ref(''),notice=ref('');
  const status=ref<BackupStatus>();
  const canManage=computed(()=>auth.user?.accessLevel==='complete'&&auth.user.active);
  function clear(){masterPassword.value='';backupPassword.value='';unlocked.value=false;status.value=undefined;}
  watch(()=>auth.sessionVersion,()=>{if(!restored.value){clear();open.value=false;error.value='';notice.value='';}});
  function show(blank=false){
    if(blank ? auth.setupComplete!==false||auth.user!==null : !canManage.value)return;
    clear();restoreMode.value=blank;error.value='';notice.value='';open.value=true;
  }
  function close(){if(busy.value||restored.value)return;clear();open.value=false;}
  async function run(operation:()=>Promise<void>){
    if(busy.value)return;busy.value=true;error.value='';notice.value='';
    const version=auth.sessionVersion;
    try{await operation();}catch(cause){const failure=normalizeIpcError(cause);if(failure.code==='UNAUTHENTICATED'&&version===auth.sessionVersion)auth.clearSession();else error.value=failure.message;}finally{if(version!==auth.sessionVersion&&!restored.value){clear();open.value=false;error.value='';notice.value='';}busy.value=false;}
  }
  const unlock=()=>run(async()=>{status.value=await backupApi.status(masterPassword.value);unlocked.value=true;});
  const create=()=>run(async()=>{const file=await backupApi.create(masterPassword.value);status.value=await backupApi.status(masterPassword.value);notice.value=`Respaldo creado: ${file}. Copia el archivo a tu USB o disco externo.`;});
  const configure=(automatic:boolean)=>run(async()=>{await backupApi.configure(masterPassword.value,automatic);status.value=await backupApi.status(masterPassword.value);notice.value=automatic?'Respaldo al cerrar activado.':'Respaldo al cerrar desactivado.';});
  const openFolder=()=>run(async()=>{await backupApi.openFolder(masterPassword.value);});
  const restore=()=>run(async()=>{
    if(await backupApi.restore(masterPassword.value,backupPassword.value)){
      restored.value=true;clear();notice.value='El respaldo fue validado y colocado. La instalación anterior se conservó. Reinicia HRL CORE para usarlo.';
    }
  });
  const restart=()=>run(async()=>{await backupApi.restart();});
  return{canManage,open,busy,unlocked,restoreMode,restored,masterPassword,backupPassword,error,notice,status,show,close,clear,unlock,create,configure,openFolder,restore,restart};
});
