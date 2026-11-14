import type { AppErrorCode, StoreError } from "../../types/ipc.ts";

const codes = new Set<AppErrorCode>([
  "STORAGE_ERROR", "MIGRATION_ERROR", "LEGACY_DATABASE", "VALIDATION_ERROR",
  "UNAUTHENTICATED", "FORBIDDEN", "INVALID_CREDENTIALS", "RATE_LIMITED",
  "CONFLICT", "NOT_FOUND", "CRYPTO_ERROR", "RECOVERY_UNAVAILABLE",
  "RECOVERY_REJECTED", "RESULT_UNCERTAIN", "WINDOW_ERROR",
]);
const clientCodes = new Set(["IPC_ERROR", "INVALID_INTEGER", "BUSY", "SESSION_CHANGED"]);

export function normalizeIpcError(cause: unknown): StoreError {
  if (typeof cause === "string") {
    try { return normalizeIpcError(JSON.parse(cause) as unknown); }
    catch { /* Una cadena no estructurada no pertenece al contrato AppError. */ }
  }
  if (typeof cause === "object" && cause !== null && "code" in cause && "message" in cause
    && typeof cause.code === "string" && typeof cause.message === "string"
    && (codes.has(cause.code as AppErrorCode) || clientCodes.has(cause.code))) {
    return { code: cause.code, message: cause.message } as StoreError;
  }
  return {
    code: "IPC_ERROR",
    message: "No se pudo confirmar la respuesta del backend. Si hay abono, conserva el mismo identificador y contenido antes de reintentar.",
  };
}
