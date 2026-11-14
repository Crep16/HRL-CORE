import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { authApi } from "../services/api/auth.ts";
import { useIpcTask } from "../composables/useIpcTask.ts";
import type {
  AccessScope, ClientError, LoginPayload, SessionResponse, SetupPayload,
  StoreError, TemporaryAccessPayload, TemporaryAccessResponse, User,
} from "../types/ipc.ts";

export const useAuthStore = defineStore("auth", () => {
  const user = ref<User | null>(null);
  const permissions = ref<string[]>([]);
  const temporaryAccess = ref<TemporaryAccessResponse | null>(null);
  const setupComplete = ref<boolean | null>(null);
  const sessionVersion = ref(0);
  const isAuthenticated = computed(() => user.value !== null);
  const task = useIpcTask((error) => {
    if (["UNAUTHENTICATED", "SESSION_CHANGED"].includes(error.code)) clearSession();
  });

  function clearSession() {
    user.value = null;
    permissions.value = [];
    temporaryAccess.value = null;
    sessionVersion.value += 1;
  }
  function assertSessionVersion(version: number) {
    if (version !== sessionVersion.value) {
      throw { code: "SESSION_CHANGED", message: "La sesión cambió durante la solicitud. Inicia sesión de nuevo." } satisfies ClientError;
    }
  }
  function clearUncertainSession(error: StoreError) {
    if (["IPC_ERROR", "RESULT_UNCERTAIN"].includes(error.code)) clearSession();
  }
  function applySession(response: SessionResponse) {
    clearSession();
    user.value = response.user;
    permissions.value = response.permissions;
    setupComplete.value = true;
  }
  function checkSetupStatus() {
    return task.run(async () => {
      const status = await authApi.checkSetupStatus();
      setupComplete.value = status.setupComplete;
      return status;
    });
  }
  function refreshSession() {
    return task.run(async () => {
      const version = sessionVersion.value;
      const response = await authApi.refresh();
      assertSessionVersion(version);
      user.value = response.user;
      permissions.value = response.permissions;
    });
  }
  function loginUser(payload: LoginPayload) {
    return task.run(async () => {
      const version = sessionVersion.value;
      const session = await authApi.loginUser(payload);
      assertSessionVersion(version);
      applySession(session);
      return session;
    }, clearUncertainSession);
  }
  function initializeSetup(payload: SetupPayload) {
    return task.run(async () => {
      const version = sessionVersion.value;
      const session = await authApi.initializeSetup(payload);
      assertSessionVersion(version);
      applySession(session);
      return session;
    }, clearUncertainSession);
  }
  function requestTemporaryAccess(payload: TemporaryAccessPayload) {
    return task.run(async () => {
      const version = sessionVersion.value;
      const grant = await authApi.requestTemporaryAccess(payload);
      assertSessionVersion(version);
      if (grant.recipientId !== user.value?.id) {
        throw { code: "SESSION_CHANGED", message: "La autorización no corresponde al usuario actual." } satisfies ClientError;
      }
      temporaryAccess.value = grant;
      return grant;
    }, clearUncertainSession);
  }
  function finishTemporaryAccess() {
    return task.run(async () => {
      temporaryAccess.value = null;
      await authApi.finishTemporaryAccess();
    });
  }
  function logoutUser() {
    return task.run(async () => {
      clearSession();
      await authApi.logoutUser();
    });
  }
  // Solo orienta la presentación. Rust revalida cada facultad y el autorizador al usarla.
  function can(operation: string, scope: AccessScope, orderId: number | null = null): boolean {
    if (!user.value) return false;
    if (permissions.value.includes(operation)) return true;
    const grant = temporaryAccess.value;
    return !!grant && grant.recipientId === user.value.id && grant.scope === scope
      && (grant.orderId === null || grant.orderId === orderId) && grant.operations.includes(operation);
  }

  return {
    listLoginUsers: () => authApi.listLoginUsers(),
    user, permissions, temporaryAccess, setupComplete, sessionVersion, isAuthenticated,
    isLoading: task.isLoading, error: task.error,
    checkSetupStatus, loginUser, initializeSetup, logoutUser, refreshSession,
    requestTemporaryAccess, finishTemporaryAccess, can, clearSession,
  };
});
