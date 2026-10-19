import type { Order } from "../types/order";

type OrderBase = Pick<Order, "id" | "client" | "serial" | "status"> & Partial<Order>;

const defaults = {
  phone: "998 555 0000",
  received: "20/09/2026",
  item: "Laptop",
  brand: "HP",
  model: "Pavilion 14",
  otherShop: false,
  dropped: false,
  dropReason: "",
  password: "",
  work: "Diagnóstico general del equipo.",
  cost: "$0.00",
  deposit: "$0.00",
  notes: "",
  paid: false,
  delivered: false,
};

function order(base: OrderBase): Order {
  return { ...defaults, ...base } as Order;
}

export const initialOrders: Order[] = [
  order({
    id: "ORD-001", client: "García Hdez.", serial: "SN-2291A", status: "En reparación",
    phone: "998 555 0101", received: "20/09/2026", item: "Laptop", brand: "HP", model: "Pavilion 14",
    dropped: true, dropReason: "Cayó de una mesa y se golpeó la esquina izquierda", password: "clave1234",
    work: "Cambiar la pantalla y revisar la bisagra izquierda.", cost: "$1,200.00", deposit: "$600.00",
    notes: "El cliente pasa por el equipo cuando se le avise.", paid: true,
  }),
  order({ id: "ORD-002", client: "Martínez L.", serial: "SN-0482C", status: "Esperando autorización", phone: "998 555 0182", brand: "Lenovo", model: "IdeaPad" }),
  order({ id: "ORD-003", client: "López Torres", serial: "SN-1139B", status: "Listo para entrega", phone: "998 555 0234", brand: "Dell", model: "Inspiron", paid: true }),
  order({ id: "ORD-004", client: "Ramírez F.", serial: "SN-3374D", status: "En reparación", phone: "998 555 0346", brand: "Acer", model: "Aspire 5" }),
  order({ id: "ORD-005", client: "Hernández R.", serial: "SN-5521E", status: "Esperando refacción", phone: "998 555 0468" }),
  order({ id: "ORD-006", client: "Castro V.", serial: "SN-6612F", status: "Entregado", phone: "998 555 0579", paid: true, delivered: true }),
  order({ id: "ORD-007", client: "Morales G.", serial: "SN-7743G", status: "En diagnóstico" }),
  order({ id: "ORD-008", client: "Vega P.", serial: "SN-8854H", status: "Autorizado" }),
  order({ id: "ORD-009", client: "Ríos A.", serial: "SN-9965I", status: "Listo para entrega" }),
  order({ id: "ORD-010", client: "Cruz N.", serial: "SN-0076J", status: "En reparación" }),
  order({ id: "ORD-011", client: "Flores M.", serial: "SN-1187K", status: "Entregado", paid: true, delivered: true }),
];
