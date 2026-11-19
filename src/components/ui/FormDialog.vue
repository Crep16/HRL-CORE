<script setup lang="ts">
import { useId } from 'vue';
import { useModalDialog } from '../../composables/useModalDialog';

const props = defineProps<{ open: boolean; title: string; busy?: boolean; error?: string }>();
const emit = defineEmits<{ close: [] }>();
const titleId = useId();
const modal = useModalDialog(props, () => emit('close'));
const { requestClose, pointerDown, click } = modal;
</script>

<template>
  <dialog :ref="modal.dialog" class="workshop-dialog form-dialog" :aria-labelledby="titleId" :aria-busy="busy" @cancel.prevent="requestClose" @pointerdown="pointerDown" @click="click">
    <header>
      <h2 :id="titleId">{{ title }}</h2>
      <button type="button" class="btn btn-s dialog-close" :aria-label="`Cerrar ${title.toLowerCase()}`" :disabled="busy" @click="requestClose"><svg class="i" viewBox="0 0 24 24" aria-hidden="true"><path d="m6 6 12 12M6 18 18 6" /></svg></button>
    </header>
    <p v-if="error" class="workshop-error" role="alert">{{ error }}</p>
    <p v-if="busy" class="workshop-status" role="status">Guardando…</p>
    <slot />
  </dialog>
</template>

<style scoped>
.form-dialog{width:min(500px,calc(100% - 32px));margin:auto;max-height:calc(100dvh - 32px)}
.form-dialog header{flex-wrap:nowrap;align-items:flex-start;margin-bottom:20px}
.form-dialog h2{padding-top:6px;overflow-wrap:anywhere}
.dialog-close{flex:none;min-width:44px;min-height:44px;padding:0}
@media(max-width:520px){.form-dialog{padding:20px}}
</style>
