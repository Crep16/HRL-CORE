<script setup lang="ts">
import { ref } from "vue";
import { useAuthStore } from "./stores/useAuthStore";
import { useWindowProfile } from "./composables/useWindowProfile";
import AccessView from "./views/AccessView.vue";
import WorkshopApp from "./WorkshopApp.vue";
import BackupDialog from './components/backup/BackupDialog.vue';
import {useBackupClose} from './composables/useBackupClose';
import {useBackupStore} from './stores/useBackupStore';
const auth = useAuthStore();
const { error: windowError, isLoading: resizing, synchronize } = useWindowProfile();
const access = ref<InstanceType<typeof AccessView>>();
const reception = ref<InstanceType<typeof WorkshopApp>>();
const backups=useBackupStore();
async function prepareLeave(){
 if(backups.busy)return false;
 if(backups.restored)return true;
 const allowed=auth.user ? await reception.value?.prepareLeave() ?? true : access.value?.prepareLeave() ?? true;
 if(allowed)backups.close();
 return allowed;
}
const closeState=useBackupClose(prepareLeave);
defineExpose({prepareLeave});
</script>
<template>
  <div v-if="windowError" role="alert" class="window-profile-notice">
    <span>{{ windowError.message }}</span>
    <button type="button" :disabled="resizing" @click="synchronize">Reintentar ajuste</button>
  </div>
  <div class="application-content" :inert="closeState.closing.value || backups.restored">
    <WorkshopApp v-if="auth.user" ref="reception" :key="auth.user.id" />
    <AccessView v-else ref="access" />
  </div>
  <BackupDialog :inert="closeState.closing.value"/>
  <div v-if="closeState.closing.value || closeState.error.value" class="backup-close-notice" :role="closeState.error.value?'alert':'status'">
    <p>{{closeState.error.value || 'Comprobando el respaldo automático antes de cerrar. Espera a que termine…'}}</p>
    <template v-if="closeState.error.value"><button type="button" @click="closeState.close">Reintentar cierre con respaldo</button><button type="button" @click="closeState.error.value=''">Seguir trabajando</button><button type="button" @click="closeState.withoutBackup">Cerrar sin un respaldo nuevo</button></template>
  </div>
</template>
<style scoped>.application-content{display:contents}.backup-close-notice{position:fixed;inset:auto 20px 20px;z-index:2000;background:#fff;color:#12222d;border:2px solid #7c8d99;padding:16px;border-radius:8px;font:16px/1.5 Arial,sans-serif}.backup-close-notice button{min-height:44px;margin:4px;padding:8px 12px;border:1px solid #7c8d99;background:white;color:#0f4c78;cursor:pointer}.backup-close-notice button:focus-visible{outline:3px solid #1b72ad;outline-offset:2px}</style>
