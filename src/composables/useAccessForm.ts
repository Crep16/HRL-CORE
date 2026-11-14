import { computed, onBeforeUnmount, ref } from "vue";
import { useAuthStore } from "../stores/useAuthStore.ts";
import { useRecoveryStore } from "../stores/useRecoveryStore.ts";

export function useAccessForm() {
  const auth = useAuthStore();
  const recovery = useRecoveryStore();
  const step = ref<"master" | "first">("master");
  const name = ref("");
  const password = ref("");
  const confirmation = ref("");
  const masterPassword = ref("");
  const masterConfirmation = ref("");
  const support = ref(false);
  const signature = ref("");
  const newMasterPassword = ref("");
  const newAdminPassword = ref("");
  const message = ref("");
  const successMessage = ref("");
  const loginUsers = ref<string[]>([]);
  const usersLoading = ref(false);
  const usersError = ref("");
  let disposed = false;
  async function loadUsers() {
    if (usersLoading.value || !auth.setupComplete || disposed) return;
    usersLoading.value = true;
    usersError.value = "";
    try {
      const users = await auth.listLoginUsers();
      if (!disposed) loginUsers.value = users;
    } catch {
      if (!disposed) {
        loginUsers.value = [];
        usersError.value = "No se pudo cargar la lista. Puedes escribir tu usuario o volver a consultar.";
      }
    } finally { if (!disposed) usersLoading.value = false; }
  }
  const view = computed(() => auth.setupComplete ? "login" : step.value);
  const busy = computed(() => auth.isLoading || recovery.isLoading);
  const title = computed(() => support.value ? "Recuperación con soporte" : ({ login: "Iniciar sesión", master: "Define la clave maestra", first: "Crea el primer administrador" })[view.value]);
  const errorMessage = computed(() => message.value || (support.value ? recovery.error?.message : auth.error?.message) || "");
  function clearSecrets() {
    password.value = confirmation.value = masterPassword.value = masterConfirmation.value = "";
    signature.value = newMasterPassword.value = newAdminPassword.value = "";
  }
  async function check() {
    message.value = "";
    try { await auth.checkSetupStatus(); await loadUsers(); } catch { /* El store publica el error. */ }
  }
  async function submit() {
    if (busy.value) return;
    message.value = "";
    if (view.value !== "login" && (password.value !== confirmation.value || [...password.value].length < 8)) {
      message.value = "Las claves deben coincidir y tener al menos 8 caracteres.";
      return;
    }
    if (view.value === "master") {
      masterPassword.value = password.value;
      masterConfirmation.value = confirmation.value;
      password.value = confirmation.value = "";
      step.value = "first";
      return;
    }
    try {
      if (view.value === "login") await auth.loginUser({ username: name.value, password: password.value });
      else await auth.initializeSetup({ name: name.value, username: name.value, password: password.value, confirmation: confirmation.value, masterPassword: masterPassword.value, masterConfirmation: masterConfirmation.value });
      clearSecrets();
    } catch { password.value = confirmation.value = ""; }
  }
  function back() { clearSecrets(); step.value = "master"; message.value = ""; }
  function toggleSupport() { if (busy.value) return; clearSecrets(); message.value = successMessage.value = ""; recovery.error = null; support.value = !support.value; recovery.clearChallenge(); }
  async function generateChallenge() {
    if (busy.value) return;
    message.value = "";
    signature.value = newMasterPassword.value = newAdminPassword.value = "";
    try { await recovery.generateRecoveryChallenge(); } catch { /* El store publica el error. */ }
  }
  async function verifySignature() {
    if (busy.value || !recovery.challenge) return;
    message.value = "";
    try {
      await recovery.verifyRecoverySignature({ challenge: recovery.challenge.challenge, signature: signature.value.trim(), newMasterPassword: newMasterPassword.value, newAdminPassword: newAdminPassword.value });
      clearSecrets(); support.value = false;
      successMessage.value = "Claves restablecidas. Inicia sesión con la nueva clave del administrador.";
    } catch { signature.value = newMasterPassword.value = newAdminPassword.value = ""; }
  }
  const hasSecrets = computed(() => !!(password.value || masterPassword.value || newMasterPassword.value || newAdminPassword.value));
  onBeforeUnmount(() => { disposed = true; loginUsers.value = []; clearSecrets(); recovery.clearChallenge(); });
  return { loginUsers, usersLoading, usersError, loadUsers, auth, recovery, step, name, password, confirmation, support, signature, newMasterPassword, newAdminPassword, message, successMessage, view, busy, title, errorMessage, hasSecrets, check, submit, back, toggleSupport, generateChallenge, verifySignature };
}
