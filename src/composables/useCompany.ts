import { computed } from 'vue';
import { useCompanyStore } from '../stores/useCompanyStore.ts';
import type { CompanySettings } from '../types/company.ts';
import type { Order } from '../types/order.ts';
export function defaultCompany(): CompanySettings {
  return {
    name: '', taxId: '', phone: '', whatsapp: '', email: '', website: '', address: '', hours: '',
    logo: '', logoName: '', logoPrint: true, logoWeb: true,
    entryFooter: 'Conserva esta nota para recoger tu equipo.',
    exitFooter: 'Gracias por tu confianza. La garantía aplica presentando esta nota.',
    printAddress: true, signature: true, webReference: false,
    clauses: [],
    clausePosition: 'foot', numberClauses: true,
    paper: 'letter', orientation: 'portrait', copies: 2,
    prefix: 'ORD-', nextNumber: 1, laborDays: 30, partsDays: 90,
    country: 'México', locale: 'es-MX', currency: 'MXN', timezone: 'America/Cancun',
    showTax: false, taxRate: 16, taxName: 'IVA',
  };
}
export const cloneCompany = (value: CompanySettings): CompanySettings => JSON.parse(JSON.stringify(value));


export function companyErrors(value: CompanySettings): string[] {
  const errors: string[] = [];
  if (!value.name.trim()) errors.push('Escribe el nombre de la empresa.');
  if (value.email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value.email)) errors.push('Revisa el formato del correo.');
  if (value.website) {
    try { const url = new URL(/^https?:\/\//i.test(value.website) ? value.website : `https://${value.website}`); if (!['http:', 'https:'].includes(url.protocol) || !url.hostname.includes('.')) throw new Error(); }
    catch { errors.push('Escribe una página válida, por ejemplo: negocio.mx.'); }
  }
  if (!/^[\p{L}\p{N}_-]{1,12}$/u.test(value.prefix)) errors.push('El prefijo admite de 1 a 12 letras, números, guiones o guiones bajos.');
  for (const [label, number, min, max] of [
    ['El siguiente número', value.nextNumber, 1, 999999999], ['Las copias', value.copies, 1, 5],
    ['La garantía de mano de obra', value.laborDays, 0, 3650], ['La garantía de refacciones', value.partsDays, 0, 3650],
  ] as const) if (!Number.isInteger(number) || number < min || number > max) errors.push(`${label} debe ser un entero entre ${min} y ${max}.`);
  if (!Number.isFinite(value.taxRate) || value.taxRate < 0 || value.taxRate > 100) errors.push('El impuesto debe estar entre 0 y 100 %.');
  if (!value.taxName.trim() || value.taxName.length > 12) errors.push('Escribe una abreviatura de impuesto de hasta 12 caracteres.');
  if (value.entryFooter.length > 200 || value.exitFooter.length > 200) errors.push('Los mensajes al pie admiten hasta 200 caracteres.');
  if (value.clauses.some(c => !c.title.trim() || !c.text.trim() || c.title.length > 80 || c.text.length > 400)) errors.push('Cada cláusula necesita un título (hasta 80 caracteres) y texto (hasta 400).');
  return errors;
}


export function useCompany() {
  const store = useCompanyStore();
  const defaults = defaultCompany();
  const company = computed(() => store.settings ?? defaults);
  const companyStorageError = computed(() => store.error?.message ?? '');
  async function loadPersistedCompany() { try { await store.load(); return true; } catch { return false; } }
  async function persistCompany(value: CompanySettings) { if (companyErrors(value).length) return false; try { await store.save(cloneCompany(value)); return true; } catch { return false; } }
  return { company, companyStorageError, persistCompany, saveCompany: persistCompany, loadPersistedCompany, nextFolio: computed(() => formatFolio(company.value.prefix, company.value.nextNumber)) };
}
export const formatFolio = (prefix: string, number: number) => `${prefix}${String(number).padStart(3, '0')}`;
export function nextAvailableNumber(company: CompanySettings, orders: Order[]): number {
  const ids = new Set(orders.map(order => order.id));
  let number = company.nextNumber;
  while (ids.has(formatFolio(company.prefix, number))) number++;
  return number;
}
