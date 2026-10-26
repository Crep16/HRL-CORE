import { computed, ref } from 'vue';
import type { CompanySettings } from '../types/company';
import type { Order } from '../types/order';

const key = 'hrl.company.v1';
export function defaultCompany(): CompanySettings {
  return {
    name: 'Héctor Refacciones Laptop', taxId: '', phone: '', whatsapp: '', email: '', website: '', address: '', hours: '',
    logo: '', logoName: '', logoPrint: true, logoWeb: true,
    entryFooter: 'Conserva esta nota para recoger tu equipo.',
    exitFooter: 'Gracias por tu confianza. La garantía aplica presentando esta nota.',
    printAddress: true, signature: true, webReference: false,
    clauses: [
      { id: 'diagnosis', title: 'Diagnóstico con costo', text: 'Si el cliente no autoriza la reparación, el diagnóstico tiene un costo de $150.', entry: true, exit: false },
      { id: 'backup', title: 'Respaldo de información', text: 'El taller no se hace responsable por la pérdida de información. Se recomienda respaldar antes de dejar el equipo.', entry: true, exit: true },
      { id: 'unclaimed', title: 'Equipos no reclamados', text: 'Si el equipo no se recoge en 30 días después de avisar que está listo, se cobran $20 por día de almacenaje.', entry: true, exit: true },
      { id: 'warranty', title: 'Alcance de la garantía', text: 'La garantía cubre solo la falla reparada y se pierde por golpes, humedad o manipulación de terceros.', entry: false, exit: true },
      { id: 'receipt', title: 'Presentar la nota', text: 'Presenta esta nota al recoger tu equipo.', entry: false, exit: false },
    ],
    clausePosition: 'foot', numberClauses: true,
    paper: 'letter', orientation: 'portrait', copies: 2,
    prefix: 'ORD-', nextNumber: 12, laborDays: 30, partsDays: 90,
    country: 'México', locale: 'es-MX', currency: 'MXN', timezone: 'America/Cancun',
    showTax: false, taxRate: 16, taxName: 'IVA',
  };
}
export const cloneCompany = (value: CompanySettings): CompanySettings => JSON.parse(JSON.stringify(value));
const settings = ref(defaultCompany());
const storageError = ref('');
let loaded = false;

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

function validStored(value: unknown): value is CompanySettings {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const v = value as Record<string, unknown>;
  const defaults = defaultCompany();
  if (!Object.entries(defaults).every(([name, sample]) => Array.isArray(sample) ? Array.isArray(v[name]) : typeof v[name] === typeof sample)) return false;
  if (!['letter', 'ticket', 'ticket60'].includes(String(v.paper)) || !['portrait', 'landscape'].includes(String(v.orientation)) || !['foot', 'back'].includes(String(v.clausePosition))) return false;
  if (!['es-MX', 'en-US', 'es-ES'].includes(String(v.locale)) || !['MXN', 'USD', 'EUR'].includes(String(v.currency))) return false;
  if (v.logo && !/^data:image\/(png|jpeg);base64,[A-Za-z0-9+/=]+$/.test(String(v.logo))) return false;
  if (!(v.clauses as unknown[]).every(c => c && typeof c === 'object' && ['id', 'title', 'text'].every(k => typeof (c as Record<string,unknown>)[k] === 'string') && typeof (c as Record<string,unknown>).entry === 'boolean' && typeof (c as Record<string,unknown>).exit === 'boolean')) return false;
  try { new Intl.DateTimeFormat('es-MX', {timeZone: String(v.timezone)}); return companyErrors(value as CompanySettings).length === 0; } catch { return false; }
}

export function useCompany() {
  if (!loaded) {
    loaded = true;
    try {
      const stored = localStorage.getItem(key);
      if (stored) {
        const value: unknown = JSON.parse(stored);
        if (value && typeof value==='object' && 'paper' in value && value.paper==='half') value.paper='letter';
        if (!validStored(value)) throw new Error(); settings.value = value;
      }
    } catch { storageError.value = 'No se pudo recuperar la configuración de Empresa. Revisa los datos antes de guardar.'; }
  }
  function saveCompany(value: CompanySettings): boolean {
    try {
      if (companyErrors(value).length) return false;
      const copy = cloneCompany(value);
      localStorage.setItem(key, JSON.stringify(copy));
      settings.value = copy;
      storageError.value = '';
      return true;
    } catch { storageError.value = 'No se pudo guardar la configuración. Revisa el espacio disponible e inténtalo de nuevo; tus cambios siguen en pantalla.'; return false; }
  }
  const nextFolio = computed(() => formatFolio(settings.value.prefix, settings.value.nextNumber));
  return { company: settings, companyStorageError: storageError, saveCompany, nextFolio };
}

export const formatFolio = (prefix: string, number: number) => `${prefix}${String(number).padStart(3, '0')}`;
export function nextAvailableNumber(company: CompanySettings, orders: Order[]): number {
  const ids = new Set(orders.map(order => order.id));
  let number = company.nextNumber;
  while (ids.has(formatFolio(company.prefix, number))) number++;
  return number;
}
