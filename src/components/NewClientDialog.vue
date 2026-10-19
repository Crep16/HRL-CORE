<script setup lang="ts">
import { ref } from "vue";
import type { ClientDraft } from "../types/client";

const emit = defineEmits<{ close: []; save: [draft: ClientDraft] }>();
const draft = ref<ClientDraft>({ name: "", phone: "", email: "" });
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')"><section class="dialog" role="dialog" aria-modal="true" aria-labelledby="dialog-title">
    <header class="dialog-header"><div><p class="eyebrow">DIRECTORIO</p><h2 id="dialog-title">Nuevo cliente</h2></div><button class="close-button" type="button" aria-label="Cerrar" @click="emit('close')"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m18 6-12 12M6 6l12 12" /></svg></button></header>
    <form class="client-form" @submit.prevent="emit('save', draft)"><label>Nombre completo<input v-model="draft.name" required autofocus placeholder="Nombre y apellidos" /></label><label>Teléfono<input v-model="draft.phone" type="tel" placeholder="998 555 0000" /></label><label>Correo electrónico<input v-model="draft.email" type="email" placeholder="nombre@correo.com" /></label><div class="dialog-actions"><button class="button secondary" type="button" @click="emit('close')">Cancelar</button><button class="button primary" type="submit">Guardar cliente</button></div></form>
  </section></div>
</template>
