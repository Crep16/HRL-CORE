<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import AppIcon from '../ui/AppIcon.vue';
import type { Order } from '../../types/order';
const props = defineProps<{ order?: Order; busy: boolean; error: string }>();
const emit = defineEmits<{ confirm: []; cancel: [] }>();
const dialog = ref<HTMLDialogElement>();
function syncDialog() {
  if (props.order && !dialog.value?.open) dialog.value?.showModal();
  else if (!props.order && dialog.value?.open) dialog.value.close();
}
watch(() => props.order, syncDialog, { flush: 'post' });
onMounted(syncDialog);
function cancel() { if (!props.busy) emit('cancel'); }
</script>

<template>
  <dialog ref="dialog" class="order-delete-dialog" aria-labelledby="delete-order-title" aria-describedby="delete-order-description" :aria-busy="busy" @cancel.prevent="cancel">
    <div class="delete-dialog-content">
      <h2 id="delete-order-title"><AppIcon name="trash" />Eliminar orden</h2>
      <p>¿Enviar <strong>{{ order?.id }}</strong>, de {{ order?.client }}, a la papelera?</p>
      <p id="delete-order-description">Dejará de aparecer en las órdenes activas. Su historial y sus cobros se conservan; podrás restaurarla desde la papelera.</p>
      <p v-if="error" class="delete-dialog-error" role="alert">{{ error }}</p>
      <p v-if="busy" role="status">Guardando el cambio…</p>
      <div class="delete-dialog-actions"><button class="btn btn-s" type="button" :disabled="busy" autofocus @click="cancel">Cancelar</button><button class="btn btn-danger" type="button" :disabled="busy" @click="emit('confirm')"><AppIcon name="trash" />Enviar a la papelera</button></div>
    </div>
  </dialog>
</template>

<style>
.order-delete-dialog{width:min(480px,calc(100vw - 32px));max-height:calc(100dvh - 32px);padding:0;border:1px solid var(--line2);border-radius:12px;color:var(--ink);background:#fff;box-shadow:0 12px 48px #12222d33}
.order-delete-dialog::backdrop{background:#12222d80}
.delete-dialog-content{padding:24px;overflow-wrap:anywhere}
.delete-dialog-content h2{display:flex;align-items:center;gap:10px;font:700 23px var(--disp);margin:0 0 18px}
.delete-dialog-content h2 .i{width:24px;height:24px}
.delete-dialog-content p{font-size:15px;line-height:1.55;margin:12px 0}
.delete-dialog-actions{display:flex;flex-wrap:wrap;justify-content:flex-end;gap:10px;margin-top:24px}
.delete-dialog-actions .btn{min-height:44px;white-space:normal}
.btn-danger{background:#a82a20;color:#fff;border-color:#a82a20}.btn-danger:hover{background:#862219}
.delete-dialog-error{color:#862219;background:#fff2f0;border-radius:6px;padding:10px}
.order-delete-dialog button:focus-visible{outline:3px solid var(--acc);outline-offset:3px}
@media(max-width:420px){.delete-dialog-content{padding:20px}.delete-dialog-actions>.btn{flex:1 1 100%}}
</style>
