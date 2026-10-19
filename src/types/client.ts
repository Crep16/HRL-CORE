export type Activity = {
  order: string;
  date: string;
  item: string;
  status: string;
  tone: string;
};

export type Client = {
  id: string;
  name: string;
  phone: string;
  email: string;
  taxId: string;
  kind: string;
  since: string;
  address: string;
  city: string;
  notes: string;
  orders: number;
  spent: string;
  frequent: boolean;
  activity: Activity[];
};

export type ClientDraft = {
  name: string;
  phone: string;
  email: string;
};
