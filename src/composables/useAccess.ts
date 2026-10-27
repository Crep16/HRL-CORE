import { readonly, ref } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';

export type AccessUser = { id: string; name: string; accessLevel: 'complete' | 'limited' | 'custom' };
type AccessStatus = { setupComplete: boolean; user: AccessUser | null };
export type FirstSetup = { name: string; password: string; confirmation: string; masterPassword: string; masterConfirmation: string };
const user = ref<AccessUser | null>(null);
const setupComplete = ref(false);
const loading = ref(false);
const error = ref('');
const permissions = ref<string[]>([]);
const available = isTauri();

async function run<T>(operation: () => Promise<T>): Promise<T | undefined> {
  if (loading.value) return;
  error.value = '';
  if (!available) { error.value = 'Abre la aplicación de escritorio para configurar o usar las cuentas de este equipo.'; return; }
  loading.value = true;
  try { return await operation(); }
  catch (cause) { error.value = typeof cause === 'string' ? cause : 'No se pudo completar el acceso. Vuelve a intentar.'; }
  finally { loading.value = false; }
}

export function useAccess() {
  async function refresh() {
    await run(async () => {
      const status = await invoke<AccessStatus>('access_status');
      permissions.value = status.user ? await invoke<string[]>('access_permissions') : [];
      user.value = status.user; setupComplete.value = status.setupComplete;
    });
  }
  async function firstSetup(input: FirstSetup) {
    return run(async () => { const account = await invoke<AccessUser>('first_setup', { input }); permissions.value = await invoke<string[]>('access_permissions'); user.value = account; setupComplete.value = true; return true; });
  }
  async function login(name: string, password: string) {
    return run(async () => { const account = await invoke<AccessUser>('login', { name, password }); permissions.value = await invoke<string[]>('access_permissions'); user.value = account; return true; });
  }
  async function logout() {
    return run(async () => { await invoke('logout'); user.value = null; permissions.value = []; return true; });
  }
  const can = (operation: string) => permissions.value.includes(operation);
  return { user: readonly(user), setupComplete: readonly(setupComplete), loading: readonly(loading), error: readonly(error), can, available, refresh, firstSetup, login, logout };
}
