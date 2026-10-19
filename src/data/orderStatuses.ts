import type { Tone } from "../types/order";

export const orderStatuses: { label: string; tone: Tone }[] = [
  { label: "En diagnóstico", tone: "d-blue" },
  { label: "Esperando autorización", tone: "d-amber" },
  { label: "Autorizado", tone: "d-blue" },
  { label: "Esperando refacción", tone: "d-amber" },
  { label: "En reparación", tone: "d-blue" },
  { label: "Listo para entrega", tone: "d-teal" },
  { label: "Entregado", tone: "d-slate" },
];

export function statusTone(status: string): Tone {
  return orderStatuses.find((entry) => entry.label === status)?.tone ?? "d-blue";
}
