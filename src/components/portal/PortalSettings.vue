<script setup lang="ts">
import AppNotice from '../ui/AppNotice.vue';
import {usePortalSettings} from '../../composables/usePortalSettings.ts';
const {store,email,password,message,error,canConnect,refresh,save,connect,control}=usePortalSettings();
</script>
<template>
  <details v-if="store.canManage" class="portal-settings">
    <summary><strong>Conexión con el portal</strong><span>{{ store.settings?.connected ? (store.settings.config.paused ? 'Cuenta conectada · envío pausado' : 'Cuenta conectada · envío activo') : 'Sin cuenta conectada' }}</span></summary>
    <div class="portal-settings-body">
      <p class="help">Las órdenes se guardan en este equipo. Solo se envía la información autorizada para el cliente.</p>
      <AppNotice :message="error" tone="error" /><AppNotice :message="message" />
      <form @submit.prevent="save"><fieldset :disabled="store.busy" class="portal-fields">
        <div class="fld"><label for="portal-project">Proyecto de Firebase</label><input id="portal-project" v-model="store.draft.projectId" required /></div>
        <div class="fld"><label for="portal-api-key">API key web</label><input id="portal-api-key" v-model="store.draft.apiKey" required autocomplete="off" /></div>
        <div class="fld"><label for="portal-uid">UID del publicador</label><input id="portal-uid" v-model="store.draft.publisherUid" required /></div>
        <div class="fld"><label for="portal-url">Dirección del portal</label><input id="portal-url" v-model="store.draft.portalUrl" type="url" placeholder="https://…" /><small>Necesaria para imprimir el enlace y el QR.</small></div>
        <button type="submit" class="btn btn-s" :disabled="!store.dirty">Guardar configuración</button>
      </fieldset></form>
      <form @submit.prevent="connect"><fieldset :disabled="!canConnect" class="portal-fields">
        <div class="fld"><label for="publisher-email">Correo del publicador en Firebase</label><input id="publisher-email" v-model="email" type="email" autocomplete="username" required /></div>
        <div class="fld"><label for="publisher-password">Contraseña de Firebase</label><input id="publisher-password" v-model="password" type="password" autocomplete="current-password" required /><small>Se usa para conectar; no se guarda.</small></div>
        <button type="submit" class="btn btn-p">Conectar cuenta</button>
      </fieldset></form>
      <p v-if="store.dirty" class="help">Guarda la configuración antes de conectar.</p>
      <p class="help">Reanudar comprueba primero las revisiones remotas. Si restauraste un respaldo, revisa las órdenes superadas o en conflicto.</p>
      <div class="portal-actions">
        <button type="button" class="btn btn-p" :disabled="store.busy || !store.settings?.connected || !store.settings.config.paused || store.dirty" @click="control('resume')">Reanudar envío</button>
        <button type="button" class="btn btn-s" :disabled="store.busy || !store.settings || store.settings.config.paused" @click="control('pause')">Pausar envío</button>
        <button type="button" class="btn btn-s" :disabled="store.busy || !store.settings?.connected" @click="control('disconnect')">Desconectar cuenta</button>
        <button type="button" class="btn btn-s" :disabled="store.busy" @click="refresh">Actualizar estado</button>
      </div><p v-if="store.busy" role="status">Procesando conexión…</p>
    </div>
  </details>
</template>
<style scoped>
.portal-settings{margin:0 0 20px;border:1px solid var(--line2);border-radius:10px;background:white}.portal-settings summary{padding:16px;cursor:pointer}.portal-settings summary span{margin-left:12px;color:var(--muted)}.portal-settings-body{padding:0 16px 16px}.portal-fields{border:0;margin:16px 0;padding:0;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:14px}.portal-fields button{justify-self:start;align-self:end}.portal-fields small{color:var(--muted)}.portal-actions{display:flex;flex-wrap:wrap;gap:10px}.fld{min-width:0}.fld input{width:100%;min-width:0}@media(max-width:640px){.portal-fields{grid-template-columns:1fr}.portal-settings summary span{display:block;margin:6px 0 0}.portal-actions button{width:100%}}
</style>
