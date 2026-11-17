<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
const props = defineProps<{ id: string; open: boolean; busy: boolean; error: string; area: string }>();
const emit = defineEmits<{ save: []; discard: []; keep: [] }>();
const dialog = ref<HTMLDialogElement>();
const keepButton = ref<HTMLButtonElement>();
let previousFocus: HTMLElement | null = null;
function sync() {
  if (props.open && !dialog.value?.open) {
    previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    dialog.value?.showModal();
    keepButton.value?.focus();
  } else if (!props.open && dialog.value?.open) {
    dialog.value.close();
    previousFocus?.focus();
  }
}
watch(() => props.open, sync, { flush: 'post' });
onMounted(sync);
function keep() { if (!props.busy) emit('keep'); }
function trapTab(event: KeyboardEvent) {
  const buttons = [...dialog.value?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? []];
  const first = buttons[0], last = buttons[buttons.length - 1];
  if (!first) { event.preventDefault(); return; }
  if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
  else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
}
</script>
<template>
  <dialog ref="dialog" class="draft-leave-dialog" :aria-labelledby="`${id}-title`" :aria-describedby="`${id}-description`" :aria-busy="busy" @cancel.prevent="keep" @keydown.tab="trapTab">
    <h2 :id="`${id}-title`">Cambios sin guardar en {{ area }}</h2>
    <p :id="`${id}-description`">Guarda los cambios antes de salir o descártalos. Si sigues editando, permanecerás aquí con tu borrador.</p>
    <p v-if="error" role="alert">{{ error }}</p>
    <p v-if="busy" role="status">Guardando cambios…</p>
    <footer>
      <button ref="keepButton" type="button" class="btn btn-s" :disabled="busy" @click="keep">Seguir editando</button>
      <button type="button" class="btn workshop-danger" :disabled="busy" @click="emit('discard')">Descartar y salir</button>
      <button type="button" class="btn btn-p" :disabled="busy" @click="emit('save')">Guardar y salir</button>
    </footer>
  </dialog>
</template>
<style scoped>
.draft-leave-dialog{box-sizing:border-box;width:min(36rem,calc(100% - 2rem));max-height:calc(100dvh - 2rem);overflow:auto;padding:24px;border:1px solid var(--line2);border-radius:12px;background:var(--card,#fff);color:var(--ink,#172d35)}
.draft-leave-dialog::backdrop{background:rgb(0 0 0 / .4)}
h2{margin:0 0 16px;font-size:1.3rem}p{line-height:1.5}footer{display:flex;flex-wrap:wrap;justify-content:flex-end;gap:12px;margin-top:24px}.btn{min-height:44px;white-space:normal}.btn:focus-visible{outline:3px solid var(--petrol,#155e68);outline-offset:3px}
@media(max-width:540px){.draft-leave-dialog{padding:20px}footer{flex-direction:column}.btn{width:100%}}
</style>
