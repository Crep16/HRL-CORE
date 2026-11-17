<script setup lang="ts">
import {onMounted, ref, watch} from 'vue';
const props=defineProps<{name?:string; suggestion:boolean; busy:boolean; error:string}>();
const emit=defineEmits<{confirm:[];cancel:[]}>();
const dialog=ref<HTMLDialogElement>(), cancelButton=ref<HTMLButtonElement>(), confirmButton=ref<HTMLButtonElement>();
let previousFocus:HTMLElement|null=null;
function sync(){
  if(props.name&&!dialog.value?.open){previousFocus=document.activeElement instanceof HTMLElement?document.activeElement:null;dialog.value?.showModal();cancelButton.value?.focus();}
  else if(!props.name&&dialog.value?.open){dialog.value.close();if(previousFocus?.isConnected)previousFocus.focus();else document.getElementById('repair-stage')?.focus();}
}
watch(()=>props.name,sync,{flush:'post'});onMounted(sync);
function cancel(){if(!props.busy)emit('cancel');}
function tab(event:KeyboardEvent){
  if(props.busy){event.preventDefault();return;}
  if(event.shiftKey&&document.activeElement===cancelButton.value){event.preventDefault();confirmButton.value?.focus();}
  else if(!event.shiftKey&&document.activeElement===confirmButton.value){event.preventDefault();cancelButton.value?.focus();}
}
</script>
<template>
  <dialog ref="dialog" class="stage-removal-dialog" aria-labelledby="stage-removal-title" aria-describedby="stage-removal-description" :aria-busy="busy" @cancel.prevent="cancel" @keydown.tab="tab">
    <h2 id="stage-removal-title">{{suggestion?'Quitar nombre del selector':'Eliminar etapa pendiente'}}</h2>
    <p id="stage-removal-description">{{suggestion?`«${name}» dejará de ofrecerse en el selector de todas las órdenes. Las etapas y los avances guardados se conservan.`:`«${name}» se quitará del avance del trabajo. Sus registros anteriores se conservarán en el historial.`}}</p>
    <p v-if="error" role="alert">{{error}}</p><p v-if="busy" role="status">Guardando el cambio…</p>
    <footer><button ref="cancelButton" type="button" class="btn btn-s" :disabled="busy" @click="cancel">Cancelar</button><button ref="confirmButton" type="button" class="btn workshop-danger" :disabled="busy" @click="emit('confirm')">{{suggestion?'Quitar nombre':'Eliminar etapa'}}</button></footer>
  </dialog>
</template>
<style scoped>
.stage-removal-dialog{box-sizing:border-box;width:min(32rem,calc(100% - 2rem));max-height:calc(100dvh - 2rem);overflow:auto;padding:24px;border:1px solid var(--line2);border-radius:12px;background:white;color:var(--ink)}.stage-removal-dialog::backdrop{background:rgb(0 0 0 / .4)}h2{margin:0 0 16px;font-size:1.25rem}p{line-height:1.5;overflow-wrap:anywhere}footer{display:flex;flex-wrap:wrap;justify-content:flex-end;gap:12px;margin-top:24px}.btn{min-height:44px}.btn:focus-visible{outline:3px solid var(--petrol);outline-offset:3px}@media(max-width:480px){.stage-removal-dialog{padding:20px}footer{flex-direction:column}.btn{width:100%}}
</style>
