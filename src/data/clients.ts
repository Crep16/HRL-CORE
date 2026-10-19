import type { Client } from "../types/client";

export const initialClients: Client[] = [
  { id: "CL-001", name: "García Hernández", phone: "998 555 0101", email: "m.garcia@email.com", taxId: "GAHM9004128K2", kind: "Particular", since: "20 sep 2024", address: "Calle 24, Mza. 18, Lt. 6", city: "Cancún, Quintana Roo, 77500", notes: "Prefiere contacto por WhatsApp. Recoge los equipos por la tarde.", orders: 8, spent: "$12,450", frequent: true, activity: [
    { order: "ORD-024", date: "20 sep 2026", item: "Laptop HP Pavilion 14", status: "En reparación", tone: "blue" },
    { order: "ORD-018", date: "04 jun 2026", item: "Cambio de teclado", status: "Entregado", tone: "teal" },
    { order: "ORD-011", date: "12 ene 2026", item: "Mantenimiento preventivo", status: "Entregado", tone: "slate" },
  ] },
  { id: "CL-002", name: "Lucía Martínez", phone: "998 555 0182", email: "lucia.mtz@email.com", taxId: "MALU850923QJ4", kind: "Particular", since: "11 feb 2025", address: "Av. Nader 48, Supermanzana 2", city: "Cancún, Quintana Roo, 77500", notes: "Contactar antes de realizar cualquier reparación adicional.", orders: 3, spent: "$4,800", frequent: false, activity: [
    { order: "ORD-023", date: "18 sep 2026", item: "Laptop Lenovo IdeaPad", status: "Esperando autorización", tone: "amber" },
    { order: "ORD-009", date: "02 may 2025", item: "Limpieza interna", status: "Entregado", tone: "teal" },
  ] },
  { id: "CL-003", name: "Roberto López Torres", phone: "998 555 0234", email: "roberto.lt@email.com", taxId: "LOTR7803065P1", kind: "Particular", since: "06 mar 2025", address: "Calle 31, Región 92", city: "Cancún, Quintana Roo, 77516", notes: "Sin notas adicionales.", orders: 5, spent: "$8,200", frequent: true, activity: [
    { order: "ORD-021", date: "12 sep 2026", item: "Laptop Dell Inspiron", status: "Listo para entrega", tone: "teal" },
    { order: "ORD-014", date: "29 nov 2025", item: "Reemplazo de batería", status: "Entregado", tone: "slate" },
  ] },
  { id: "CL-004", name: "Fernanda Ramírez", phone: "998 555 0346", email: "fer.ramirez@email.com", taxId: "RARF9207142B7", kind: "Particular", since: "14 jul 2025", address: "Av. Las Torres 215", city: "Cancún, Quintana Roo, 77535", notes: "Entregar diagnóstico por correo electrónico.", orders: 2, spent: "$3,100", frequent: false, activity: [
    { order: "ORD-020", date: "08 sep 2026", item: "Laptop Acer Aspire 5", status: "En diagnóstico", tone: "blue" },
  ] },
  { id: "CL-005", name: "Héctor Hernández Ruiz", phone: "998 555 0468", email: "hector.hr@email.com", taxId: "HERH810221MN3", kind: "Empresa", since: "21 ago 2025", address: "Av. Tulum 320, Local 4", city: "Cancún, Quintana Roo, 77500", notes: "Facturación mensual. Solicitar constancia fiscal actualizada.", orders: 12, spent: "$26,750", frequent: true, activity: [
    { order: "ORD-019", date: "02 sep 2026", item: "Mantenimiento de equipo", status: "Esperando refacción", tone: "amber" },
    { order: "ORD-015", date: "17 feb 2026", item: "Diagnóstico de laptop", status: "Entregado", tone: "teal" },
  ] },
  { id: "CL-006", name: "Valeria Castro", phone: "998 555 0579", email: "val.cast@email.com", taxId: "CAVV9605188R6", kind: "Particular", since: "09 ene 2026", address: "Calle 10, Supermanzana 15", city: "Cancún, Quintana Roo, 77505", notes: "Sin notas adicionales.", orders: 1, spent: "$950", frequent: false, activity: [
    { order: "ORD-016", date: "18 ago 2026", item: "Respaldo de información", status: "Entregado", tone: "slate" },
  ] },
];
