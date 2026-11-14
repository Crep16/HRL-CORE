import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { companyApi } from '../services/api/company.ts';
import { useAuthStore } from './useAuthStore.ts';
import { useIpcTask } from '../composables/useIpcTask.ts';
import type { CompanySettings } from '../types/company.ts';
export const useCompanyStore = defineStore('company', () => {
  const auth = useAuthStore();
  const settings = ref<CompanySettings | null>(null);
  const loaded = ref(false);
  const task = useIpcTask(error => { if (error.code === 'UNAUTHENTICATED') auth.clearSession(); });
  watch(() => auth.sessionVersion, () => { settings.value = null; loaded.value = false; });
  async function run(save?: CompanySettings) {
    return task.run(async () => {
      const version = auth.sessionVersion;
      const response = save ? await companyApi.save(save, settings.value?.revision ?? 0) : await companyApi.load();
      if (auth.sessionVersion !== version) throw { code: 'SESSION_CHANGED', message: 'La sesión cambió durante la consulta de Empresa.' };
      settings.value = response;
      loaded.value = true;
      return response;
    });
  }
  return { settings, loaded, error: task.error, isLoading: task.isLoading, load: () => run(), save: (value: CompanySettings) => run(value) };
});
