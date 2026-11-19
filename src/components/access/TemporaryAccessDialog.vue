<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { useTemporaryAccessForm } from "../../composables/useTemporaryAccessForm";
import type { TemporaryAccessContext } from "../../composables/useTemporaryAccessForm";
const props = defineProps<{ request: TemporaryAccessContext | null; chooseOperation?: boolean }>();
const emit = defineEmits<{ close: []; approved: [] }>();
const { auth, username, password, reason, errorMessage, reset, approve } = useTemporaryAccessForm();
const dialog = ref<HTMLDialogElement>();
const operation = ref("orders.create");
let previousFocus: HTMLElement | null = null;
const labels = { clients: "Clientes", repairs: "Reparaciones", company: "Empresa", users: "Usuarios", logbook: "Finanzas" };
const actionGroups: Record<string,Record<string,string>> = {
  clients:{'clients.view':'Consultar Clientes','orders.create':'Crear una orden','orders.edit':'Editar una orden','orders.delete':'Enviar a papelera o restaurar','evidence.add':'Agregar fotografías','evidence.edit':'Editar descripciones','evidence.delete':'Retirar o restaurar fotografías','payments.add':'Registrar abono inicial'},
  repairs:{'repairs.view':'Consultar Reparaciones','repairs.edit':'Editar seguimiento','notes.add':'Agregar notas y avances','stages.advance':'Completar etapa','warranties.add':'Agregar garantía','parts.add':'Agregar refacción','publication.edit':'Editar preferencias de publicación','payments.add':'Registrar abono','payments.adjust':'Corregir abono','payments.refund':'Devolver saldo a favor'},
  company:{'company.view':'Consultar Empresa','company.edit':'Editar Empresa'},
  logbook:{'logbook.view':'Consultar Finanzas'},users:{'users.manage':'Administrar usuarios'},
};
const actions = computed(() => actionGroups[props.request?.scope ?? 'clients']!);
watch(() => props.request, async (request) => {
  reset(); await nextTick();
  if (request) {
    operation.value = request.operation;
    previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    if (!dialog.value?.open) dialog.value?.showModal();
    await nextTick(); dialog.value?.querySelector<HTMLInputElement>("#temporary-user")?.focus();
  } else { dialog.value?.close(); previousFocus?.focus(); }
}, { immediate: true });
function cancel() { if (!auth.isLoading) { password.value = ""; emit("close"); } }
async function submit() {
  if (props.request && await approve({ ...props.request, operation: props.chooseOperation ? operation.value : props.request.operation })) emit("approved");
}
onBeforeUnmount(() => { password.value = ""; dialog.value?.close(); });
</script>
<template>
  <dialog ref="dialog" class="temporary-dialog" aria-labelledby="temporary-title" aria-describedby="temporary-description" @cancel.prevent="cancel">
    <template v-if="request">
      <header><h2 id="temporary-title">Necesitas autorización temporal</h2><button type="button" class="btn btn-s" :disabled="auth.isLoading" @click="cancel">Cerrar</button></header>
      <p id="temporary-description">La persona que autoriza introduce sus credenciales y motivo. Tu identidad como operador se conserva.</p>
      <p v-if="!chooseOperation"><strong>{{ actions[request.operation] ?? request.operation }}</strong> · {{ labels[request.scope] }}</p>
      <form @submit.prevent="submit">
        <fieldset :disabled="auth.isLoading">
          <div v-if="chooseOperation"><label for="temporary-operation">Acción que requiere autorización</label><select id="temporary-operation" v-model="operation"><option v-for="(label, code) in actions" :key="code" :value="code">{{ label }}</option></select></div>
          <div v-if="request.scope === 'repairs'"><label for="temporary-order">Orden autorizada</label><input id="temporary-order" :value="request.folio ?? request.orderId" readonly maxlength="100"></div>
          <div><label for="temporary-user">Usuario que autoriza</label><input id="temporary-user" v-model="username" autocomplete="off" autocapitalize="none" spellcheck="false" maxlength="100" required></div>
          <div><label for="temporary-password">Contraseña de quien autoriza</label><input id="temporary-password" v-model="password" type="password" autocomplete="off" maxlength="1024" required></div>
          <div><label for="temporary-reason">Motivo de la autorización</label><textarea id="temporary-reason" v-model="reason" rows="3" maxlength="2000" required></textarea></div>
        </fieldset>
        <p v-if="errorMessage" role="alert" class="temporary-error">{{ errorMessage }}</p>
        <p v-if="auth.isLoading" role="status" class="workshop-status">Comprobando autorización…</p>
        <footer><button type="button" class="btn btn-s" :disabled="auth.isLoading" @click="cancel">Cancelar</button><button type="submit" class="btn btn-p" :disabled="auth.isLoading">Autorizar este apartado temporalmente</button></footer>
      </form>
    </template>
  </dialog>
</template>

<style scoped>
.temporary-dialog{width:min(620px,calc(100vw - 32px));max-height:calc(100dvh - 32px);overflow:auto;padding:24px;border:1px solid var(--line2);border-radius:12px;background:var(--surface,#fff);color:var(--ink);font-family:var(--body)}.temporary-dialog::backdrop{background:rgb(16 40 50 / .5)}header,footer{display:flex;justify-content:space-between;gap:12px;align-items:center}header h2{font:700 24px var(--disp);margin:0}p{line-height:1.6}fieldset{display:grid;gap:14px;border:0;margin:0;padding:0;min-width:0}input,textarea,select{min-height:44px;width:100%;border:1px solid #73838f;border-radius:8px;padding:10px;font:inherit}footer{margin-top:20px;flex-wrap:wrap}.temporary-error{color:#9b2525}:focus-visible{outline:2px solid var(--acc-ink);outline-offset:3px}
</style>
