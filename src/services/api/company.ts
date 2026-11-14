import { request } from './transport.ts';
import type { CompanySettings } from '../../types/company.ts';
type CompanyDto = Omit<CompanySettings, 'taxRate'> & { taxRateThousandths: number };
function fromDto(value: CompanyDto): CompanySettings {
  const { taxRateThousandths, ...settings } = value;
  return { ...settings, taxRate: taxRateThousandths / 1000 };
}
export const companyApi = {
  async load() { const result = await request<CompanyDto | null>('load_normalized_company', {}); return result ? fromDto(result) : null; },
  async save(value: CompanySettings, expectedRevision: number) {
    const { taxRate, ...settings } = value;
    const taxRateThousandths = Math.round(taxRate * 1000);
    return fromDto(await request<CompanyDto>('save_normalized_company', { payload: { value: { ...settings, taxRateThousandths }, expectedRevision } }));
  },
};
