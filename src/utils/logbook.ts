import type { LogbookChange, LogbookEntry } from '../types/logbook.ts';

export const logbookOperations: Record<string, string> = {
  'repairs.save':'Detalle del seguimiento guardado',
  'evidence.add':'Foto de evidencia agregada','evidence.edit':'Descripción de evidencia modificada','evidence.delete':'Foto de evidencia retirada','evidence.restore':'Foto de evidencia restaurada',
  'access.temporary.start':'Acceso temporal autorizado','access.temporary.use':'Autorización temporal utilizada','access.temporary.end':'Acceso temporal finalizado',
  'payments.refund':'Saldo a favor devuelto','payments.adjust':'Abono corregido',
  'costs.add':'Gasto directo registrado','users.create':'Usuario creado','users.edit':'Usuario o permisos modificados','company.edit':'Configuración de Empresa modificada',
  'orders.create':'Orden creada', 'orders.edit':'Orden modificada',
  'orders.delete':'Orden enviada a papelera', 'orders.restore':'Orden restaurada',
  'payments.add':'Abono registrado', 'notes.add':'Avance o nota agregada',
  'stages.advance':'Etapa completada', 'repairs.edit':'Seguimiento modificado',
  'warranties.add':'Garantía agregada', 'parts.add':'Refacción agregada',
  'publication.edit':'Configuración de publicación modificada', imported:'Seguimiento importado',
};
export const logbookFields: Record<string, string> = {
  equipmentHistory:'Antecedentes del equipo',
  estimatedDelivery:'Entrega estimada',sufferedImpact:'Golpes',impactDetail:'Detalle de golpes',liquidSpill:'Derrame de líquidos',liquidDetail:'Detalle de líquidos',delivered:'Equipo entregado',revision:'Revisión',
  evidenceId:'Identificador de foto',evidenceName:'Archivo de evidencia',evidenceSource:'Origen de la foto',evidenceNote:'Descripción de la foto',evidenceRemoved:'Foto retirada',
  temporaryGrant:'Identificador de autorización',authorizedBy:'Autorizado por',performedBy:'Realizado por',temporaryScope:'Apartado autorizado',temporaryOperations:'Permisos prestados',temporaryOperation:'Acción autorizada',
  paymentMethod:'Método utilizado',paymentReference:'Abono original',
  credentialsUpdated:'Se actualizó la clave de acceso',
  taxId:'RFC',whatsapp:'WhatsApp',email:'Correo',website:'Página web',address:'Dirección',hours:'Horario',logo:'Logo',logoName:'Nombre del logo',logoPrint:'Logo en impresión',logoWeb:'Logo en web',entryFooter:'Mensaje de entrada',exitFooter:'Mensaje de salida',printAddress:'Imprimir dirección',signature:'Firma',webReference:'Referencia de orden',clauses:'Cláusulas',clausePosition:'Posición de cláusulas',numberClauses:'Numerar cláusulas',paper:'Papel',orientation:'Orientación',copies:'Copias',prefix:'Prefijo de folios',nextNumber:'Siguiente folio',laborDays:'Garantía de mano de obra',partsDays:'Garantía de refacciones',country:'País',locale:'Formato regional',currency:'Moneda',timezone:'Zona horaria',showTax:'Mostrar impuesto',taxRate:'Tasa de impuesto',taxName:'Nombre del impuesto',
  direct_cost_minor:'Importe del costo directo',expenseDescription:'Concepto del gasto',name:'Nombre',accessLevel:'Nivel de acceso',active:'Usuario activo',permissions:'Permisos',
  created:'Folio',client:'Cliente',phone:'Teléfono',item:'Artículo',brand:'Marca',model:'Modelo',serial:'Número de serie',
  received:'Fecha de recepción',status:'Estado',failure:'Falla',work:'Trabajo registrado',price_minor:'Importe de la orden',
  other_shop:'Paso por otro taller',other_shop_reason:'Detalle del otro taller',dropped:'Caída o humedad',drop_reason:'Detalle de caída o humedad',
  delivered_at:'Fecha de entrega',notes:'Notas de recepción',amount_minor:'Importe del abono',
  completed:'Etapas completadas',stages:'Etapas de la orden',technician:'Técnico',publicNote:'Nota para el cliente',
  internalNote:'Nota interna',events:'Avances agregados',warranties:'Garantías agregadas',parts:'Refacciones agregadas',
  syncMode:'Modo de publicación',visibility:'Información visible para el cliente',
};
function decode(value: string | null): unknown {
  if (value === null) return null;
  try { return JSON.parse(value); } catch { return value; }
}
export function entryChanges(entry: LogbookEntry): LogbookChange[] {
  if (entry.details) return entry.details;
  if (entry.operation === 'payments.add') return [{field:'amount_minor',oldValue:null,newValue:Number(entry.newValue)}];
  if (!entry.field) return [];
  return [{field:entry.field,oldValue:entry.field === 'notes' ? entry.oldValue : decode(entry.oldValue),newValue:entry.field === 'notes' ? entry.newValue : decode(entry.newValue)}];
}
export function logbookValue(field: string, value: unknown, currency: string, locale: string): string {
  if (value === null || value === undefined || value === '') return 'Sin registrar';
  if (['price_minor','amount_minor','direct_cost_minor','amountMinor','priceMinor'].includes(field)||field.endsWith('.priceMinor')) return new Intl.NumberFormat(locale,{style:'currency',currency}).format(Number(value)/100);
  if (typeof value === 'boolean') return value ? 'Sí' : 'No';
  if (field==='temporaryScope') return ({clients:'Clientes',repairs:'Reparaciones',logbook:'Bitácora',company:'Empresa',users:'Usuarios'} as Record<string,string>)[String(value)]??String(value);
  if ((field==='temporaryOperation'||field==='temporaryOperations')&&typeof value==='string') return logbookOperations[value]??value;
  if (field === 'syncMode') return value === 'auto' ? 'Automática' : 'Manual';
  if (Array.isArray(value)) return value.length ? value.map(item => logbookValue(field,item,currency,locale)).join('\n\n') : 'Ninguno';
  if (typeof value === 'object') {
    const item = value as Record<string, unknown>;
    if (field === 'events') return `${item.stage}: ${item.title}\n${String(item.date).replace('T',' ')} · ${item.public ? 'Para el cliente' : 'Solo interno'}${item.detail ? `\n${item.detail}` : ''}`;
    if (field === 'warranties') return `${item.name}: ${item.days} días`;
    if (field === 'parts') return `${item.name}: ${new Intl.NumberFormat(locale,{style:'currency',currency}).format(Number(item.price))}`;
    if (field === 'visibility') return `${item.label}: ${item.enabled ? 'Visible' : 'Oculto'}`;
    if (field === 'permissions') return `${item.section}: ${item.action}`;
    if (field === 'clauses') return `${item.title}: ${item.text}\nEntrada: ${item.entry?'Sí':'No'} · Salida: ${item.exit?'Sí':'No'}`;
    return 'Detalle no disponible';
  }
  return String(value);
}
