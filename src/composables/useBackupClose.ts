import {onMounted,onBeforeUnmount,ref} from 'vue';
import {backupApi} from '../services/api/backups.ts';
import {windowApi} from '../services/api/window.ts';
import {normalizeIpcError} from '../services/api/errors.ts';
export function useBackupClose(prepare:()=>Promise<boolean>){
 const closing=ref(false),error=ref('');let stop:(()=>void)|undefined;let disposed=false;
 async function close(){
  if(closing.value)return;error.value='';closing.value=true;
  try{
   if(!await prepare()){error.value='Hay cambios sin confirmar o una operación en curso. Guarda o descarta los cambios antes de cerrar.';return;}
   await backupApi.automatic();await windowApi.finishClose();
  }
  catch(cause){error.value=normalizeIpcError(cause).message;}
  finally{closing.value=false;}
 }
 async function withoutBackup(){
  if(closing.value)return;closing.value=true;
  try{if(await prepare())await windowApi.finishClose();}
  catch(cause){error.value=normalizeIpcError(cause).message;}
  finally{closing.value=false;}
 }
 onMounted(async()=>{if(!windowApi.native())return;try{const unlisten=await windowApi.listenClose(close);if(disposed)unlisten();else stop=unlisten;}catch{error.value='No se pudo preparar el respaldo al cerrar. Reinicia la aplicación antes de continuar.';}});
 onBeforeUnmount(()=>{disposed=true;stop?.();});
 return{closing,error,close,withoutBackup};
}
