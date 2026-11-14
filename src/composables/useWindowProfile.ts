import { isTauri } from "@tauri-apps/api/core";
import { watch } from "vue";
import { useAuthStore } from "../stores/useAuthStore.ts";
import { windowApi } from "../services/api/window.ts";
import { useIpcTask } from "./useIpcTask.ts";

export function useWindowProfile() {
  const auth = useAuthStore();
  const task = useIpcTask();
  // Cada petición consulta la sesión real; la cola evita perder cambios durante un ajuste.
  let pending = Promise.resolve();
  function synchronize() {
    if (!isTauri() || auth.setupComplete === null) return pending;
    pending = pending.then(async () => {
      try { await task.run(windowApi.synchronizeProfile); }
      catch { /* El aviso permite reintentar sin alterar la sesión. */ }
    });
    return pending;
  }
  watch(() => [auth.isAuthenticated, auth.setupComplete], synchronize, { immediate: true });
  return { error: task.error, isLoading: task.isLoading, synchronize };
}
