<script setup lang="ts">
import AppNotice from '../ui/AppNotice.vue';
import FormDialog from '../ui/FormDialog.vue';
import {usePortalOrder} from '../../composables/usePortalOrder.ts';
const props=defineProps<{orderId:number;beforeAction:()=>Promise<boolean>;disabled?:boolean}>();
const {current,working,canEdit,label,confirmation,error,pending,refresh,execute,request}=usePortalOrder(props.orderId,props.beforeAction);
</script>
<template>
  <div class="portal-publication">
    <span class="pill" role="status">{{ label }}</span>
    <p v-if="current?.configured && current.paused" class="help">El envío está pausado en Empresa.</p>
    <p v-if="current?.code">Código del cliente: <strong>{{ current.code }}</strong></p>
    <p v-if="current?.url" class="portal-link"><a :href="current.url" target="_blank" rel="noopener noreferrer">Abrir seguimiento del cliente</a></p>
    <p class="help">Guardar conserva los cambios localmente. El envío se confirma cuando Firebase los acepta.</p>
    <p v-if="current?.state==='confirmed' && confirmation" class="help">Confirmación: {{ confirmation }}</p>
    <AppNotice :message="error || current?.error || ''" tone="error" />
    <div class="portal-actions">
      <button type="button" class="btn btn-p" :disabled="disabled || working || !canEdit" @click="request('publish')">Publicar cambios</button>
      <button type="button" class="btn btn-s" :disabled="disabled || working || !canEdit || !current?.code" @click="request('withdraw')">Retirar seguimiento</button>
      <button type="button" class="btn btn-s" :disabled="disabled || working || !canEdit || !current?.code" @click="request('rotate')">Cambiar código</button>
      <button v-if="current?.state==='conflict' || current?.state==='superseded'" type="button" class="btn btn-s" :disabled="disabled || working || !canEdit" @click="request('reconcile')">Reconciliar y publicar</button>
      <button type="button" class="btn btn-s" :disabled="working" @click="refresh">Actualizar estado</button>
    </div>
    <FormDialog :open="pending!==null" title="Confirmar publicación" :busy="working" :error="error" @close="pending=null">
      <p v-if="pending==='withdraw'">Se retirará el seguimiento del cliente cuando haya conexión y Firebase lo confirme.</p>
      <p v-else-if="pending==='rotate'">El enlace anterior se retirará y se generará un código nuevo. Entrega el nuevo comprobante al cliente. El cambio remoto requiere conexión.</p>
      <p v-else>Se consultará la revisión remota y se preparará una nueva publicación con los datos actuales de este equipo. Revisa la orden antes de continuar.</p>
      <div class="portal-actions"><button type="button" class="btn btn-s" :disabled="working" @click="pending=null">Cancelar</button><button type="button" class="btn btn-p" :disabled="working" @click="pending && execute(pending)">Confirmar</button></div>
    </FormDialog>
  </div>
</template>
<style scoped>
.portal-publication{display:grid;gap:10px}.portal-publication .pill{justify-self:start}.portal-publication p{margin:0}.portal-link{overflow-wrap:anywhere}.portal-actions{display:flex;flex-wrap:wrap;gap:8px}
</style>
