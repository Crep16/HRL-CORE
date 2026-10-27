<script setup lang="ts">
import { onMounted } from 'vue';
import AccessWorkspace from './components/access/AccessWorkspace.vue';
import WorkshopApp from './WorkshopApp.vue';
import { useAccess } from './composables/useAccess';
const { user, error, refresh, logout } = useAccess();
onMounted(refresh);
</script>

<template>
  <template v-if="user">
    <p v-if="error" class="notice" role="alert">{{ error }}</p>
    <WorkshopApp :key="user.id" @logout="logout" />
  </template>
  <AccessWorkspace v-else />
</template>
