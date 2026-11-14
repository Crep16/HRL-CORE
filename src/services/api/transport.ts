import { invoke } from "@tauri-apps/api/core";
import type { ClientError } from "../../types/ipc.ts";

// Control de representación IPC: permisos, saldos y validación de negocio siguen en Rust.
function assertSafeIntegers(value: unknown): void {
  if (typeof value === "number" && !Number.isSafeInteger(value)) {
    throw {
      code: "INVALID_INTEGER",
      message: "Los importes en centavos y los identificadores deben ser enteros seguros de JavaScript.",
    } satisfies ClientError;
  }
  if (Array.isArray(value)) value.forEach(assertSafeIntegers);
  else if (typeof value === "object" && value !== null) Object.values(value).forEach(assertSafeIntegers);
}

export async function request<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  assertSafeIntegers(args);
  const response = await invoke<T>(command, args);
  try {
    assertSafeIntegers(response);
  } catch {
    // Una respuesta ilegible no demuestra que una escritura haya sido rechazada.
    throw {
      code: "IPC_ERROR",
      message: "El backend respondió con un entero fuera del rango seguro. Consulta el resultado antes de repetir; si hay abono, conserva la solicitud exacta.",
    } satisfies ClientError;
  }
  return response;
}
