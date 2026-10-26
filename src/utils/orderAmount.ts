// Orders may contain both Mexican and European money formatting.
export function orderAmount(value: string): number {
  const text = value.replace(/[^\d,.-]/g, '');
  const lastDot = text.lastIndexOf('.');
  const lastComma = text.lastIndexOf(',');
  const separator = Math.max(lastDot, lastComma);
  const decimals = separator >= 0 ? text.length - separator - 1 : 0;
  const hasDecimal = separator >= 0 && decimals > 0 && decimals <= 2;
  const normalized = hasDecimal
    ? `${text.slice(0, separator).replace(/[.,]/g, '')}.${text.slice(separator + 1)}`
    : text.replace(/[.,]/g, '');
  const amount = Number(normalized);
  return Number.isFinite(amount) ? amount : 0;
}
