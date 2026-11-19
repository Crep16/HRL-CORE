# Finanzas

## Alcance vigente

Decisión del usuario del 19 de noviembre de 2026: el apartado antes llamado
Bitácora pasa a llamarse **Finanzas**. Se retira de este apartado la consulta
de auditoría: historial de cambios, filtros por operación y autor, comparación
antes/después y enlace al historial de una orden. Finanzas abre directamente
el resumen financiero, sin pestañas adicionales.

La auditoría interna sigue siendo inmutable y se confirma desde los servicios
en la misma transacción que cada cambio, conforme a `AGENTS.md`. El historial
de una orden disponible en Clientes conserva su comportamiento. Este recorte
no elimina registros, tablas, comandos internos ni reglas de dinero.

## Acceso

El nombre visible es Finanzas en la navegación, permisos de Usuarios y acceso
temporal. Se conservan los códigos internos `logbook` y `logbook.view` para
mantener las autorizaciones existentes. Las consultas se autorizan en Rust.
Registrar otros gastos directos requiere además `repairs.edit` y una orden
que no esté cancelada ni en papelera.

## Totales y desglose

Por cada moneda se presentan total de órdenes activas, cobrado, saldo pendiente,
costos directos y ganancia estimada total. La ganancia también se divide entre
trabajos pagados al completo y trabajos con deuda, independientemente de su
entrega. El desglose muestra cantidad, total, cobrado, pendiente, costos y
ganancia para estas cuatro clases:

| Clase | Regla |
|---|---|
| Pagadas y entregadas | Cobro neto igual o superior al total y equipo entregado. |
| Pagadas sin entregar | Cobro neto igual o superior al total y equipo sin entregar. |
| Entregadas con saldo pendiente | Cobro neto inferior al total y equipo entregado. |
| Sin liquidar ni entregar | Cobro neto inferior al total y equipo sin entregar. |

Los totales corresponden al conjunto filtrado, no solo a la página visible.
Nunca se mezclan monedas. Pago, entrega y estado técnico se consultan por
separado. El filtro de trabajos con pago o entrega pendientes agrupa las tres
últimas clases.

Cobro neto = cobros originales + ajustes − devoluciones. Pendiente = máximo
entre total − cobro neto y cero; saldo a favor = máximo entre cobro neto − total
y cero. Ganancia estimada = total − costos directos registrados, incluyendo
refacciones y otros gastos directos. Es provisional y antes de gastos generales;
la ganancia de trabajos con deuda no significa que ese importe ya se haya cobrado.
Se conserva el desglose de cobros originales, ajustes, devoluciones y saldo a favor.

## Filtros y detalle

Se conservan búsqueda por folio o cliente, clasificación, fechas de recepción,
movimientos de pago o entrega, y órdenes activas, excluidas o todas. Las fechas
respetan la zona horaria de Empresa. La paginación muestra 50 órdenes por página.

Al filtrar por movimientos de pago, el cobrado del periodo suma solo movimientos
de esas fechas, también en el desglose por clase. El saldo, los costos y la
ganancia usan el acumulado de las órdenes seleccionadas. Los pagos sin fecha
conocida no se asignan a un periodo. En recepción o entrega se muestra el cobrado
acumulado, sin atribuirlo a esas fechas.

Las órdenes canceladas o en papelera se excluyen de los totales activos y del
desglose de ganancias; sus cobros se muestran por separado al incluirlas en la
consulta. El detalle conserva pago completo, entrega, fechas, cobros, ajustes,
devoluciones, saldo a favor, movimientos de pago, costos y registro de otros
gastos directos. Los libros conservan su inmutabilidad y las reglas de
`reglas-pagos.md`; no se modifican cobros originales para corregirlos.
