export type SearchField = "all" | "order" | "client" | "serial";

export type Tone = "d-blue" | "d-amber" | "d-teal" | "d-slate";

export type Order = {
  id: string;
  currency?: 'MXN' | 'USD' | 'EUR';
  client: string;
  phone: string;
  received: string;
  status: string;
  serial: string;
  item: string;
  brand: string;
  model: string;
  otherShop: boolean;
  otherShopReason: string;
  dropped: boolean;
  dropReason: string;
  password: string;
  work: string;
  cost: string;
  deposit: string;
  notes: string;
  paid: boolean;
  delivered: boolean;
};
