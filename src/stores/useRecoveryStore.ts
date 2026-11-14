import { defineStore } from "pinia";
import { ref } from "vue";
import { recoveryApi } from "../services/api/recovery.ts";
import { useIpcTask } from "../composables/useIpcTask.ts";
import { useAuthStore } from "./useAuthStore.ts";
import type { RecoveryChallengeResponse, RecoveryResponse, VerifyRecoveryPayload } from "../types/ipc.ts";

export const useRecoveryStore = defineStore("recovery", () => {
  const challenge = ref<RecoveryChallengeResponse | null>(null);
  const expiresAt = ref<number | null>(null);
  const result = ref<RecoveryResponse | null>(null);
  const auth = useAuthStore();
  const task = useIpcTask((error) => {
    if (["RECOVERY_REJECTED", "IPC_ERROR", "RESULT_UNCERTAIN"].includes(error.code)) clearChallenge();
  });
  function clearChallenge() {
    challenge.value = null;
    expiresAt.value = null;
  }
  function generateRecoveryChallenge() {
    return task.run(async () => {
      clearChallenge();
      result.value = null;
      const response = await recoveryApi.generateRecoveryChallenge();
      challenge.value = response;
      // Solo referencia visual: la caducidad válida la decide el reloj monotónico de Rust.
      expiresAt.value = Date.now() + response.expiresInSeconds * 1000;
      return response;
    });
  }
  function verifyRecoverySignature(payload: VerifyRecoveryPayload) {
    return task.run(async () => {
      result.value = null;
      const response = await recoveryApi.verifyRecoverySignature(payload);
      clearChallenge();
      auth.clearSession();
      result.value = response;
      return response;
    }, (error) => {
      if (["IPC_ERROR", "RESULT_UNCERTAIN"].includes(error.code)) auth.clearSession();
    });
  }
  return {
    challenge, expiresAt, result, isLoading: task.isLoading, error: task.error,
    generateRecoveryChallenge, verifyRecoverySignature, clearChallenge,
  };
});
