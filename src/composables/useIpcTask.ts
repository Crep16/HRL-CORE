import { ref } from "vue";
import { normalizeIpcError } from "../services/api/errors.ts";
import type { StoreError } from "../types/ipc.ts";

export function useIpcTask(onError?: (error: StoreError) => void) {
  const isLoading = ref(false);
  const error = ref<StoreError | null>(null);

  async function run<T>(operation: () => Promise<T>, onFailure?: (error: StoreError) => void): Promise<T> {
    if (isLoading.value) {
      const busy: StoreError = { code: "BUSY", message: "Espera a que termine la operación actual." };
      error.value = busy;
      throw busy;
    }
    isLoading.value = true;
    error.value = null;
    try {
      const result = await operation();
      error.value = null;
      return result;
    } catch (cause: unknown) {
      const normalized = normalizeIpcError(cause);
      error.value = normalized;
      onError?.(normalized);
      onFailure?.(normalized);
      throw normalized;
    } finally {
      isLoading.value = false;
    }
  }
  return { isLoading, error, run };
}
