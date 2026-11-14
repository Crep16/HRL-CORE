/** Entrada decimal sin separadores de millares. La conversión nunca multiplica floats. */
export function parseMinorUnits(input: string): number {
  const match = /^(\d+)(?:[.,](\d{1,2}))?$/.exec(input.trim() || "0");
  if (!match) throw new Error("Escribe un importe sin separadores de millares y con hasta dos decimales.");
  const minor = BigInt(match[1]) * BigInt(100) + BigInt((match[2] ?? "").padEnd(2, "0"));
  if (minor > BigInt(Number.MAX_SAFE_INTEGER)) throw new Error("El importe supera el rango de enteros seguros.");
  return Number(minor);
}

export function formatMinorUnits(minor: number): string {
  if (!Number.isSafeInteger(minor)) throw new Error("El importe debe ser un entero seguro de centavos.");
  const value = BigInt(minor);
  const positive = value < BigInt(0) ? -value : value;
  return `${value < BigInt(0) ? "-" : ""}${positive / BigInt(100)}.${(positive % BigInt(100)).toString().padStart(2, "0")}`;
}
