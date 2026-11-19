// Implementación de referencia de la huella del contrato (§6). HRL CORE debe producir
// el mismo resultado en Rust; el vector de prueba está en el contrato y en reglas.test.js.
import { createHash } from 'node:crypto'

// Fechas: ISO 8601 en UTC con milisegundos, p. ej. 2026-11-19T22:57:17.594Z
function normalizar(v) {
  if (v instanceof Date) return v.toISOString()
  if (v && typeof v.toDate === 'function') return v.toDate().toISOString()
  if (Array.isArray(v)) return v.map(normalizar)
  if (v && typeof v === 'object') {
    return Object.fromEntries(Object.keys(v).sort().map((k) => [k, normalizar(v[k])]))
  }
  return v
}

// JSON canónico: claves ordenadas, sin espacios, UTF-8; se excluyen actualizadoEn y huella
export function huella(documento) {
  const { actualizadoEn, huella: _h, ...resto } = documento
  const canonico = JSON.stringify(normalizar(resto))
  return createHash('sha256').update(canonico, 'utf8').digest('hex')
}
