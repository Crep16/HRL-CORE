import { computed, onBeforeUnmount, ref } from "vue";
import { useAuthStore } from "../stores/useAuthStore.ts";
import type { AccessScope } from "../types/ipc.ts";

export interface TemporaryAccessContext { operation: string; scope: AccessScope; orderId: number | null; folio?: string }
export function useTemporaryAccessForm() {
  const auth = useAuthStore();
  const username = ref(""); const password = ref(""); const reason = ref("");
  const message = ref("");
  const errorMessage = computed(() => message.value || auth.error?.message || "");
  function reset() { username.value = password.value = reason.value = message.value = ""; auth.error = null; }
  async function approve(request: TemporaryAccessContext) {
    if (auth.isLoading) return false;
    try {
      await auth.requestTemporaryAccess({ username: username.value, password: password.value, reason: reason.value, scope: request.scope, orderId: request.orderId, operation: request.operation });
      return true;
    } catch { return false; }
    finally { password.value = ""; }
  }
  onBeforeUnmount(() => { password.value = ""; });
  return { auth, username, password, reason, errorMessage, reset, approve };
}
