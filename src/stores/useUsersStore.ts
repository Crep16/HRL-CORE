import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { usersApi } from '../services/api/users.ts';
import type { Account, UserDraft } from '../types/users.ts';
import { useAuthStore } from './useAuthStore.ts';
import { useIpcTask } from '../composables/useIpcTask.ts';
export const useUsersStore = defineStore('users', () => {
  const auth = useAuthStore();
  const accounts = ref<Account[]>([]);
  const task = useIpcTask(error => { if (error.code === 'UNAUTHENTICATED') auth.clearSession(); if (['UNAUTHENTICATED','FORBIDDEN'].includes(error.code)) accounts.value = []; });
  watch(() => auth.sessionVersion, () => { accounts.value = []; });
  async function load() { return task.run(async () => { const version = auth.sessionVersion; const rows = await usersApi.list(); if (version !== auth.sessionVersion) throw { code:'SESSION_CHANGED',message:'La sesión cambió.' }; accounts.value = rows; return rows; }); }
  async function save(input: UserDraft) { return task.run(async () => { const version=auth.sessionVersion;await usersApi.save(input);if(version!==auth.sessionVersion)throw {code:'SESSION_CHANGED',message:'La sesión cambió durante el guardado.'};await auth.refreshSession(); }); }
  return { accounts, load, save, error: task.error, isLoading: task.isLoading };
});
