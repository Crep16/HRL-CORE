<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue';
import { useAccess } from '../../composables/useAccess';
const {accessRequest,approveTemporary,resolveRequest}=useAccess();
const dialog=ref<HTMLDialogElement>(),name=ref(''),password=ref(''),reason=ref(''),recordId=ref(''),error=ref(''),busy=ref(false);
const labels:Record<string,string>={clients:'Clientes',repairs:'Reparaciones',logbook:'Bitácora',company:'Empresa',users:'Usuarios'};
const actions:Record<string,string>={'clients.view':'Consultar Clientes','orders.create':'Crear una orden','orders.edit':'Editar una orden','orders.delete':'Eliminar o restaurar órdenes','repairs.view':'Abrir Reparaciones','repairs.edit':'Editar Reparaciones','notes.add':'Agregar notas o avances','stages.advance':'Completar etapas','warranties.add':'Agregar garantías','parts.add':'Agregar refacciones','payments.add':'Registrar abonos','payments.refund':'Devolver saldo a favor','payments.adjust':'Corregir abonos','logbook.view':'Consultar Bitácora','company.view':'Consultar Empresa','company.edit':'Editar Empresa','users.manage':'Administrar usuarios'};
Object.assign(actions,{'evidence.view':'Consultar fotografías de evidencia','evidence.add':'Agregar fotografías a una orden','evidence.edit':'Editar descripciones de fotografías','evidence.delete':'Retirar o restaurar fotografías'});
let previousFocus:HTMLElement|null=null;
watch(accessRequest,async value=>{
 password.value='';name.value='';reason.value='';error.value='';recordId.value=value?.recordId??'';
 await nextTick();
 if(value){previousFocus=document.activeElement instanceof HTMLElement?document.activeElement:null;dialog.value?.showModal();await nextTick();dialog.value?.querySelector<HTMLInputElement>(value.scope==='repairs'&&!value.recordId?'#temporary-order':'#temporary-user')?.focus();}
 else{dialog.value?.close();previousFocus?.focus();}
});
function cancel(){if(busy.value)return;password.value='';resolveRequest(false);}
async function approve(){
 if(busy.value||!accessRequest.value)return;busy.value=true;error.value='';
 try{await approveTemporary(name.value,password.value,reason.value,accessRequest.value.scope==='repairs'?recordId.value.trim():null);}
 catch(cause){error.value=typeof cause==='string'?cause:'No se pudo autorizar el acceso. Inténtalo de nuevo.';}
 finally{password.value='';busy.value=false;}
}
onBeforeUnmount(()=>{password.value='';resolveRequest(false);});
</script>
<template>
 <dialog ref="dialog" class="temporary-dialog" aria-labelledby="temporary-title" aria-describedby="temporary-description" @cancel.prevent="cancel">
  <template v-if="accessRequest">
   <header><h2 id="temporary-title">Necesitas autorización temporal</h2><button class="btn btn-s" type="button" :disabled="busy" aria-label="Cerrar solicitud de acceso temporal" @click="cancel">Cerrar</button></header>
   <p id="temporary-description">No tienes acceso a esta opción. Consulta con un superior o una persona que tenga el permiso para que autorice el acceso temporal.</p>
   <p><strong>{{ actions[accessRequest.operation]??accessRequest.operation }}</strong> · {{ labels[accessRequest.scope] }}</p>
   <p>La persona que autoriza debe introducir su usuario y contraseña. Tu sesión se conserva. El permiso termina al salir de este apartado o cambiar de orden y se registra en Bitácora.</p>
   <form @submit.prevent="approve">
    <fieldset :disabled="busy">
     <div v-if="accessRequest.scope==='repairs'" class="fld"><label for="temporary-order">Folio de la orden</label><input id="temporary-order" v-model="recordId" :readonly="!!accessRequest.recordId" maxlength="100" required /></div>
     <div class="fld"><label for="temporary-user">Usuario que autoriza</label><input id="temporary-user" v-model="name" autocomplete="off" maxlength="100" required /></div>
     <div class="fld"><label for="temporary-password">Contraseña de quien autoriza</label><input id="temporary-password" v-model="password" type="password" autocomplete="off" maxlength="1024" required /></div>
     <div class="fld"><label for="temporary-reason">Motivo de la autorización</label><textarea id="temporary-reason" v-model="reason" maxlength="2000" rows="3" required></textarea></div>
    </fieldset>
    <p v-if="error" class="temporary-error" role="alert">{{ error }}</p>
    <p v-if="busy" role="status">Comprobando autorización…</p>
    <footer><button class="btn btn-s" type="button" :disabled="busy" @click="cancel">Cancelar</button><button class="btn btn-p" :disabled="busy">Autorizar este apartado temporalmente</button></footer>
   </form>
  </template>
 </dialog>
</template>
<style scoped>
.temporary-dialog{width:min(620px,calc(100vw - 32px));max-height:calc(100dvh - 32px);overflow:auto;padding:24px;border:1px solid var(--line2);border-radius:12px;background:var(--surface,#fff);color:var(--ink);font-family:var(--body)}.temporary-dialog::backdrop{background:rgb(16 40 50 / .5)}header,footer{display:flex;justify-content:space-between;gap:12px;align-items:center}header h2{font:700 24px var(--disp);margin:0}p{line-height:1.6}fieldset{display:grid;gap:14px;border:0;margin:0;padding:0;min-width:0}input,textarea{min-height:44px;width:100%;border:1px solid #73838f;border-radius:8px;padding:10px;font:inherit}footer{margin-top:20px;flex-wrap:wrap}.temporary-error{color:#9b2525}:focus-visible{outline:2px solid var(--acc-ink);outline-offset:3px}
</style>
