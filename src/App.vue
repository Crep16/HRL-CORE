<script setup lang="ts">
import { ref } from "vue";
import { useAuthStore } from "./stores/useAuthStore";
import { useWindowProfile } from "./composables/useWindowProfile";
import AccessView from "./views/AccessView.vue";
import WorkshopApp from "./WorkshopApp.vue";
const auth = useAuthStore();
const { error: windowError, isLoading: resizing, synchronize } = useWindowProfile();
const access = ref<InstanceType<typeof AccessView>>();
const reception = ref<InstanceType<typeof WorkshopApp>>();
defineExpose({ prepareLeave: async () => auth.user ? await reception.value?.prepareLeave() ?? true : access.value?.prepareLeave() ?? true });
</script>
<template>
  <div v-if="windowError" role="alert" class="window-profile-notice">
    <span>{{ windowError.message }}</span>
    <button type="button" :disabled="resizing" @click="synchronize">Reintentar ajuste</button>
  </div>
  <WorkshopApp v-if="auth.user" ref="reception" :key="auth.user.id" />
  <AccessView v-else ref="access" />
</template>
