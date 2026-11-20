import {computed,onMounted,onBeforeUnmount,ref} from 'vue';
import {usePortalStore} from '../stores/usePortalStore.ts';
import {normalizeIpcError} from '../services/api/errors.ts';
export function usePortalSettings(){
  const store=usePortalStore(),email=ref(''),password=ref(''),message=ref(''),localError=ref('');
  const error=computed(()=>localError.value||store.error||store.settings?.error||'');
  const canConnect=computed(()=>!!store.settings?.config.projectId&&!store.dirty&&!store.busy);
  async function perform(operation:()=>Promise<unknown>,success=''){
    message.value='';localError.value='';try{await operation();message.value=success;}catch(cause){localError.value=normalizeIpcError(cause).message;}
  }
  async function refresh(){if(store.canManage)await perform(()=>store.loadSettings());}
  async function save(){await perform(()=>store.configure(),'Configuración guardada. El envío está pausado.');}
  async function connect(){const credential=password.value;password.value='';await perform(()=>store.connect(email.value,credential),'Cuenta conectada. Revisa y reanuda el envío.');}
  async function control(action:'pause'|'resume'|'disconnect'){await perform(()=>store.control(action),action==='resume'?'Envío reanudado.':action==='pause'?'Envío pausado.':'Cuenta desconectada.');}
  onMounted(refresh);onBeforeUnmount(()=>{password.value='';email.value='';});
  return {store,email,password,message,error,canConnect,refresh,save,connect,control};
}
