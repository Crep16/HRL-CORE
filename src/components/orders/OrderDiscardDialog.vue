<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue';
const props=defineProps<{open:boolean;orderId:string;busy:boolean}>();
const emit=defineEmits<{keep:[];discard:[]}>();
const dialog=ref<HTMLDialogElement>();
watch(()=>props.open,value=>{if(value&&!dialog.value?.open)dialog.value?.showModal();else if(!value)dialog.value?.close();},{flush:'post'});
onBeforeUnmount(()=>dialog.value?.close());
</script>
<template>
 <dialog ref="dialog" class="order-discard-dialog" aria-labelledby="discard-order-title" aria-describedby="discard-order-description" @cancel.prevent="!busy&&emit('keep')">
  <h2 id="discard-order-title">¿Descartar los cambios?</h2>
  <p id="discard-order-description">Los cambios sin guardar de <strong>{{ orderId }}</strong> se perderán. La información guardada de la orden se conserva.</p>
  <div><button class="btn btn-s" type="button" autofocus :disabled="busy" @click="emit('keep')">Seguir editando</button><button class="btn btn-p" type="button" :disabled="busy" @click="emit('discard')">Descartar cambios</button></div>
 </dialog>
</template>
<style scoped>
.order-discard-dialog{width:min(480px,calc(100vw - 32px));max-height:calc(100dvh - 32px);padding:24px;border:1px solid #73838f;border-radius:12px;background:#fff;color:var(--ink);font:16px/1.5 var(--body);box-shadow:0 12px 48px #12222d33}.order-discard-dialog::backdrop{background:#12222d80}h2{margin:0 0 16px;font:700 23px var(--disp)}p{margin:0 0 22px;overflow-wrap:anywhere}div{display:flex;justify-content:flex-end;gap:10px;flex-wrap:wrap}:focus-visible{outline:3px solid var(--acc);outline-offset:3px}@media(max-width:420px){.btn{flex:1 1 100%}}
</style>
