<script setup lang="ts">
import { ref, watch } from 'vue';
const props=withDefaults(defineProps<{message:string;tone?:'status'|'error';closable?:boolean}>(),{tone:'status',closable:true});
const emit=defineEmits<{dismiss:[]}>();
const hidden=ref(false);
watch(()=>props.message,()=>{hidden.value=false;});
function dismiss(){hidden.value=true;emit('dismiss');}
</script>
<template>
 <Teleport defer to="#app-notices">
  <div v-if="message&&!hidden" class="app-notice" :class="{'app-notice-error':tone==='error'}" :role="tone==='error'?'alert':'status'" aria-atomic="true">
   <span>{{ message }}</span><div v-if="$slots.default" class="app-notice-actions"><slot /></div>
   <button v-if="closable" class="app-notice-close" type="button" aria-label="Cerrar aviso" @click="dismiss">×</button>
  </div>
 </Teleport>
</template>
<style scoped>
.app-notice{display:flex;align-items:center;gap:12px;flex-wrap:wrap;box-sizing:border-box;width:fit-content;max-width:100%;margin:0;padding:12px 16px;background:var(--petrol,#17384a);color:#fff;border:1px solid #466576;border-radius:10px;box-shadow:0 8px 28px #12222d40;font:600 16px/1.5 var(--body,Arial,sans-serif);pointer-events:auto}
.app-notice>span{min-width:0;flex:1;overflow-wrap:anywhere;white-space:pre-line}
.app-notice-error{background:#7e211b;border-color:#a8281f}
.app-notice-close{flex:none;display:grid;place-items:center;min-width:36px;min-height:36px;margin:-4px -4px -4px 0;padding:0;border:0;border-radius:4px;background:transparent;color:#fff;font-size:24px;cursor:pointer}
.app-notice-close:hover{background:#ffffff24}.app-notice-close:focus-visible{outline:2px solid white;outline-offset:2px}
.app-notice-actions{display:flex;align-items:center;gap:8px;flex-wrap:wrap}
.app-notice-actions :deep(button){color:white;border:1px solid #c8d9e4;background:transparent;min-height:36px;font-size:14px;cursor:pointer}
.app-notice-actions :deep(button:focus-visible){outline:2px solid white;outline-offset:2px}
</style>
