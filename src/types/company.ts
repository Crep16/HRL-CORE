export type NoteKind = 'entry' | 'exit';
export const paperLabels = {letter:'Carta',ticket:'Ticket de 80 mm',ticket60:'Ticket de 60 mm'} as const;
export type CompanySection = 'general' | 'logo' | 'notes' | 'clauses' | 'paper' | 'folios' | 'region';
export type Clause = { id: string; title: string; text: string; entry: boolean; exit: boolean };
export type CompanySettings = {
  name: string; taxId: string; phone: string; whatsapp: string; email: string; website: string;
  address: string; hours: string;
  logo: string; logoName: string; logoPrint: boolean; logoWeb: boolean;
  entryFooter: string; exitFooter: string; printAddress: boolean; signature: boolean; webReference: boolean;
  clauses: Clause[]; clausePosition: 'foot' | 'back'; numberClauses: boolean;
  paper: keyof typeof paperLabels; orientation: 'portrait' | 'landscape'; copies: number;
  prefix: string; nextNumber: number; laborDays: number; partsDays: number;
  country: string; locale: 'es-MX' | 'en-US' | 'es-ES'; currency: 'MXN' | 'USD' | 'EUR';
  timezone: string; showTax: boolean; taxRate: number; taxName: string;
};
