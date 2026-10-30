<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';
import AccessWorkspace from './components/access/AccessWorkspace.vue';
import WorkshopApp from './WorkshopApp.vue';
import AppNotice from './components/ui/AppNotice.vue';
import { useAccess } from './composables/useAccess';
const { user, setupComplete, error, refresh, logout } = useAccess();
const windowError = ref('');
watch(() => [!!user.value, setupComplete.value], async () => {
  if (!isTauri()) return;
  try { await invoke('sync_window_profile'); windowError.value = ''; }
  catch (cause) { windowError.value = String(cause); }
}, { immediate: true });
onMounted(refresh);
</script>

<template>
  <div id="app-notices" class="app-notices" :style="{top:user?'calc(var(--app-header-bottom,76px) + 12px)':'16px'}" aria-label="Avisos del sistema"></div>
  <AppNotice :message="windowError" tone="error" @dismiss="windowError=''" />
  <template v-if="user">
    <AppNotice :message="error" tone="error" />
    <WorkshopApp :key="user.id" @logout="logout" />
  </template>
  <AccessWorkspace v-else />
</template>
<style>
.app-notices{position:fixed;left:50%;transform:translateX(-50%);z-index:1000;display:flex;flex-direction:column;align-items:center;gap:10px;width:max-content;max-width:min(84ch,calc(100vw - 32px));max-height:calc(100dvh - 100px);overflow-y:auto;padding:4px;pointer-events:none}
@media print{.app-notices{display:none!important}}
</style>
