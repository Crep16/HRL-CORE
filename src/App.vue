<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';
import AccessWorkspace from './components/access/AccessWorkspace.vue';
import WorkshopApp from './WorkshopApp.vue';
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
  <p v-if="windowError" class="notice" role="alert">{{ windowError }}</p>
  <template v-if="user">
    <p v-if="error" class="notice" role="alert">{{ error }}</p>
    <WorkshopApp :key="user.id" @logout="logout" />
  </template>
  <AccessWorkspace v-else />
</template>
