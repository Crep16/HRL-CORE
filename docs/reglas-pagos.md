# Reglas de pagos y ajustes

## R02: reducción del precio con saldo a favor

Decisión explícita del usuario, 28 de octubre de 2026:
«Registrar saldo a favor y permitir una devolución posterior».

Si una orden tiene $750 cobrados y su precio baja a $600, conserva los cobros
originales y presenta $150 de saldo a favor del cliente, con $0 pendiente de
pago. La modificación del precio no registra una devolución automática.
La devolución posterior será una operación explícita y trazable.

## R03: devolución total del saldo a favor

Decisión explícita del usuario, 28 de octubre de 2026:
«No, devolver siempre todo el saldo a favor».

Cada devolución debe cubrir el saldo a favor completo que tenga la orden al
confirmar la operación. No se permiten devoluciones parciales. Por ejemplo,
si el saldo a favor vigente es $150, la devolución es de $150; no se permite
devolver $50 y dejar $100 para después. La devolución sigue siendo posterior
y explícita, no automática al reducir el precio.

El backend deberá recalcular ese saldo dentro de la transacción de devolución
para no usar un importe desactualizado ni devolver el mismo excedente dos veces.

## Permiso para registrar devoluciones

Decisión explícita del usuario, 28 de octubre de 2026:
«completo limitado y personalizado en caso de que tenga el permiso habilitado».

Los tres tipos de acceso pueden registrar la devolución si tienen habilitado
el permiso correspondiente. El acceso Limitado no debe rechazarse únicamente
por su tipo de acceso cuando tenga ese permiso. Esta decisión reemplaza la
restricción anterior del Limitado específicamente para devoluciones.

Decisión adicional explícita del usuario, 28 de octubre de 2026:
«Sí, permisos separados para abonos y devoluciones».

Registrar abonos y devolver saldo a favor requieren permisos independientes.
Habilitar uno no habilita el otro. Por tanto, un usuario Limitado puede tener
permiso de devolución sin tener permiso de cobro. La ausencia del permiso de
devolución debe impedir esa operación aunque el usuario pueda registrar abonos.
Ambas autorizaciones deben comprobarse en Rust al confirmar cada operación.

Esta decisión no concede por sí sola permisos para
editar órdenes, eliminar registros, administrar usuarios ni otras operaciones.

## R03: corrección de abonos

Decisiones explícitas del usuario, 28 de octubre de 2026:
«Registrar un ajuste de −$200 ligado al abono original» y
«Solo usuarios con acceso Completo».

Un abono capturado por $500 cuyo importe correcto era $300 se corrige mediante
un movimiento de ajuste de −$200, vinculado al abono original. El abono original
se conserva sin editarlo ni borrarlo. La corrección registra motivo, autor y
fecha, y modifica el cobro efectivo utilizado para calcular deuda y saldo a favor.
No representa una devolución de dinero al cliente.

Solo el acceso Completo puede registrar correcciones. Los permisos de abonos y
devoluciones no habilitan correcciones para Limitado ni Personalizado. Esta
restricción debe comprobarse también en el backend.

La implementación debe rechazar el reenvío de una misma operación para no
duplicar cobros, ajustes o devoluciones. Antes de confirmar debe recalcular los
importes vigentes y mantener movimiento e historial en una sola transacción.
Estas son condiciones de integridad; no se deben borrar movimientos para
resolver un reintento.

Para la implementación, cobro neto = cobros originales + ajustes − devoluciones:

- Pendiente de pago = máximo entre precio menos cobros netos y cero.
- Saldo a favor = máximo entre cobros netos menos precio y cero.

El cambio de precio debe conservar autor, fecha y valores anterior y nuevo
en la auditoría. El saldo a favor debe distinguirse de la deuda del cliente.

## Estado de implementación

Implementado localmente el 29 de octubre de 2026, con migración 010. Se permite
reducir el precio conservando cobros; el excedente se muestra separado del
pendiente. `payments` conserva los abonos originales inmutables y
`payment_operations` guarda ajustes ligados al abono y devoluciones totales.
`payment_requests` identifica cada solicitud para conservar los reintentos sin
duplicar movimientos. Operación, protección de reintento y auditoría se guardan
en la misma transacción. Dos abonos concurrentes calculados contra el mismo
saldo no pueden confirmarse ambos.

Usuarios permite conceder Registrar abonos y Devolver saldo a favor por separado
a Limitado o Personalizado. La migración conserva la facultad de cobro de los
Personalizados que ya podían cobrar mediante Editar en Reparaciones, sin darles
devoluciones. Un Personalizado nuevo no recibe cobro por marcar Editar.

Los formularios nuevos capturan método libre; devolución y corrección requieren
motivo. Los errores inciertos conservan la solicitud exacta en almacenamiento
local para reintentarla, incluso al volver a abrir el panel. El método de los
movimientos históricos permanece desconocido. Clientes, Reparaciones y Bitácora
usan cobro neto, deuda y saldo a favor coherentes. Los filtros por movimientos
separan cobros, ajustes y devoluciones del periodo del acumulado de la orden.

La migración 011 permite una autorización temporal de quien sí posee la facultad
requerida, con ambas identidades auditadas y sin cambiar la sesión del operador.
Una corrección sigue necesitando la facultad del acceso Completo: los demás
accesos pueden solicitar que una persona con esa facultad la autorice para la
ventana y orden actuales. Véase `docs/acceso-temporal.md`.

Las pruebas automatizadas no sustituyen el recorrido visual nativo pendiente.

## R04: estados especiales

Decisión explícita del usuario, 28 de octubre de 2026:
«Conservarlos; cualquier cambio de precio o devolución se registra aparte».

Cancelar una orden o marcarla irreparable, abandonada o diagnóstico no
autorizado conserva precio y movimientos. El estado técnico no cancela deuda,
reduce el precio ni genera una devolución automática.

## Método de pago

Decisión explícita del usuario, 28 de octubre de 2026:
«Método escrito libremente por quien registra».

Los formularios de nuevos abonos y devoluciones permiten escribir el método,
sin imponer un catálogo de efectivo, transferencia o tarjeta. Los movimientos
anteriores cuyo método no se registró deben mostrarlo como desconocido.

No se considera autorizado usar el saldo de una orden en otra ni se define
todavía ese flujo. Referencias Jira: HRL-9, HRL-22 y HRL-27.
