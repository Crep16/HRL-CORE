# HRL-24 [Epic] Módulo de Reparaciones

Fuente de alcance vigente: `docs/backlog-base (1).md` (80 ítems, sin HRL-66). Actualización documental del 12 de noviembre de 2026. Los estados de ítems sin cambio de alcance conservan la evidencia de la revisión estática previa; no se ejecutaron pruebas.

Estado: Hecho por inspección estática de las historias; validación ejecutada y nativa No verificada (HRL-64).
Prioridad: Alta, porque reúne seguimiento técnico, cobros, ajustes y devoluciones.
Puntos: 63 (suma de historias; epic sin cómputo adicional).
Sprint: 3 y 4, según historia.
Subtareas: agrupador de HRL-25 a HRL-38. No se añade trabajo a historias completas por lectura.

Criterios de aceptación del epic (fuentes: `docs/reparaciones-permisos.md`, `docs/reglas-pagos.md`):

- Dada una reparación autorizada, cuando registra trabajo técnico, entonces conserva revisión, historial y separación de notas públicas/internas.
- Dado un movimiento financiero válido, cuando confirma, entonces recalcula saldo y registra movimiento inmutable y auditoría sin duplicarlo al reintentar.
- Dado permiso insuficiente, revisión antigua o intento de devolución parcial, cuando guarda, entonces rechaza sin alterar el expediente ni el dinero.

Los puntos expresan alcance completo. Las pruebas citadas existen, pero no se ejecutaron. Reglas técnicas: `docs/reparaciones-permisos.md`; reglas monetarias: `docs/reglas-pagos.md`; independencia pago/entrega/avance: `docs/contexto-funcional-vistas-y-estructura.md` §2. Las menciones a migraciones anteriores describen la implementación histórica, no la ruta normalizada actual.

## HRL-25 [Historia] Abrir reparación sobre la orden seleccionada con contexto de cliente y equipo

Estado: Hecho (evidencia: `src/WorkshopApp.vue` montaje de `RepairWorkspace`, `src/composables/useRepairWorkspace.ts::load`, `src-tauri/src/services/repairs.rs::RepairService::load`, `src-tauri/src/services/orders.rs::OrderService::get`). Carga orden y seguimiento con autorización para Reparaciones.
Prioridad: Alta, porque habilita todas las operaciones técnicas de la orden.
Puntos: 3
Sprint: 3
Subtareas: ninguna faltante detectada; prueba presente `src-tauri/src/services/tests.rs::restored_repair_uses_relational_records_and_rejects_stale_changes`, no ejecutada.
Criterios de aceptación (fuente: `docs/reparaciones-permisos.md`):

- Dada una orden activa seleccionada y `repairs.view`, cuando abre gestión, entonces muestra el contexto de cliente/equipo y su seguimiento persistido.
- Dado acceso Limitado, cuando abre una reparación activa, entonces puede consultar sin permiso de administrar usuarios.
- Dada una orden en papelera o permiso insuficiente, cuando abre gestión, entonces se rechaza sin crear seguimiento.

## HRL-26 [Historia] Asignar técnico y registrar notas internas y públicas

Estado: Hecho (evidencia: `src/composables/useRepairScreen.ts::appendNote`, `src-tauri/src/domain/repair_contracts.rs::operations`, `src-tauri/src/services/repairs.rs::RepairService::save`, `src-tauri/src/repositories/repairs.rs::RepairRepository::save`). Diferencia anexar notas de reemplazar texto o cambiar técnico.
Prioridad: Media, porque documenta responsable y comunicación del trabajo.
Puntos: 5
Sprint: 3
Subtareas: ninguna faltante detectada en los contratos y flujo; pruebas de seguimiento en `src-tauri/src/services/tests.rs`, no ejecutadas.
Criterios de aceptación (fuente: `docs/reparaciones-permisos.md`):

- Dado permiso de editar Reparaciones, cuando cambia técnico o texto de notas, entonces persiste con revisión y auditoría.
- Dado acceso Limitado, cuando añade una nota pública o interna, entonces se conserva el texto anterior y se añade el nuevo.
- Dado acceso Limitado sin edición, cuando intenta reemplazar notas previas o cambiar técnico, entonces Rust deniega la modificación.
- Dada una nota interna, cuando consulta el resumen para el cliente, entonces no aparece en dicho resumen.

## HRL-27 [Historia] Etapas y avances con fecha (completar etapa no marca entrega)

Estado: Hecho (evidencia: `src/composables/useRepairScreen.ts::addEvent` y `complete`, `src-tauri/src/domain/repair_contracts.rs::operations`, `src-tauri/src/repositories/repairs.rs::RepairRepository::save`). Guarda avances con fecha y etapas relacionadas sin escribir la entrega de la orden.
Prioridad: Media, porque organiza el progreso técnico sin alterar dinero ni entrega.
Puntos: 5
Sprint: 3
Subtareas: ninguna faltante de implementación detectada; prueba relacional de seguimiento presente, sin ejecutar, en `src-tauri/src/services/tests.rs::restored_repair_uses_relational_records_and_rejects_stale_changes`.
Criterios de aceptación (fuentes: `docs/reparaciones-permisos.md`, contexto funcional §2):

- Dado permiso de agregar avances, cuando registra título, fecha y etapa, entonces conserva el avance asociado a la secuencia de esa orden.
- Dado permiso de avanzar, cuando completa una etapa, entonces aumenta el avance técnico sin marcar entrega ni modificar el cobro.
- Dado acceso Limitado sin edición, cuando intenta retroceder etapas o reemplazar avances guardados, entonces se rechaza.

## HRL-28 [Historia] Registro de refacciones utilizadas con costo

Estado: Hecho (evidencia: `src/composables/useRepairScreen.ts::addPart`, `src/services/api/workshop.ts::saveRepair`, `src-tauri/src/domain/repair_contracts.rs::validate` y `operations`, `src-tauri/src/repositories/repairs.rs::RepairRepository::save`). La API transforma el importe a centavos enteros; SQL escribe `precio_centavos`.
Prioridad: Alta, porque registra costos usados en el resultado financiero.
Puntos: 3
Sprint: 3
Subtareas: ninguna faltante detectada en el registro; pruebas presentes en `src-tauri/src/services/tests.rs::restored_repair_uses_relational_records_and_rejects_stale_changes`, sin ejecutar.
Criterios de aceptación (fuentes: `docs/reparaciones-permisos.md`, arquitectura objetivo §7):

- Dado permiso `parts.add`, cuando agrega una refacción con nombre y costo válido, entonces persiste vinculada a la orden con importe entero en centavos.
- Dado acceso Limitado, cuando agrega una pieza, entonces conserva las refacciones anteriores sin editarlas ni borrarlas.
- Dado costo negativo, nombre vacío o ausencia del permiso, cuando guarda, entonces rechaza sin registrar la pieza.

## HRL-29 [Historia] Registro de garantías (cobertura y días)

Estado: Hecho (evidencia: `src/composables/useRepairScreen.ts::addWarranty`, `src-tauri/src/domain/repair_contracts.rs::validate` y `operations`, `src-tauri/src/repositories/repairs.rs::RepairRepository::save`). Registra cobertura y días sin sustituir garantías anteriores.
Prioridad: Media, porque define lo que se comunica al entregar el equipo.
Puntos: 3
Sprint: 3
Subtareas: ninguna faltante detectada; pruebas de persistencia técnica presentes, sin ejecutar, en `src-tauri/src/services/tests.rs`.
Criterios de aceptación (fuente: `docs/reparaciones-permisos.md`; contexto funcional, notas de salida):

- Dado permiso `warranties.add`, cuando registra cobertura y días no negativos, entonces conserva la garantía asociada a esa orden.
- Dada una garantía persistida, cuando prepara la nota de salida, entonces aparece su cobertura y duración.
- Dada cobertura vacía, días negativos o falta de permiso, cuando guarda, entonces se rechaza sin alterar garantías anteriores.

## HRL-30 [Historia] Control de revisión y permisos por operación en el guardado

Estado: Hecho (evidencia: `src-tauri/src/services/repairs.rs::RepairService::save`, `src-tauri/src/domain/repair_contracts.rs::operations`, `src/composables/useRepairWorkspace.ts::persist`). Compara revisión y deduce permisos del cambio real, no de lo declarado por la UI.
Prioridad: Alta, porque evita sobreescritura, escalamiento de facultades y cambios sin auditoría.
Puntos: 5
Sprint: 3
Subtareas: ninguna faltante detectada. Prueba presente `src-tauri/src/services/tests.rs::restored_repair_uses_relational_records_and_rejects_stale_changes`, no ejecutada.
Criterios de aceptación (fuentes: `docs/reparaciones-permisos.md`, `docs/acceso-temporal.md`):

- Dada una revisión vigente, cuando guarda varios cambios, entonces Rust comprueba cada permiso necesario antes de confirmar.
- Dada una revisión antigua, cuando guarda, entonces rechaza sin sobrescribir el seguimiento y conserva el borrador en pantalla.
- Dado un cambio sin autorización o un fallo de auditoría, cuando guarda, entonces revierte toda la transacción.

## HRL-31 [Historia] Panel de pagos: total, abonado, pendiente, saldo a favor y movimientos

Estado: Hecho (evidencia: `src/composables/usePaymentsPanel.ts::load` y `remaining`, `src/components/repairs/PaymentsPanel.vue` panel, `src-tauri/src/services/ledger.rs::LedgerService::list`, `src-tauri/src/repositories/ledger.rs::LedgerRepository::state` y `movements`). El saldo viene de `vista_saldos_orden`.
Prioridad: Alta, porque informa el saldo utilizado para cobrar o devolver.
Puntos: 3
Sprint: 4
Subtareas: ninguna faltante detectada; prueba presente `src-tauri/src/services/tests.rs::restored_edit_trash_finances_and_refund_preserve_original_books`, no ejecutada.
Criterios de aceptación (fuente: `docs/reglas-pagos.md`):

- Dadas cobros, correcciones y devoluciones, cuando abre el panel, entonces el neto es cobros más ajustes menos devoluciones.
- Dado precio menor al neto, cuando consulta, entonces muestra pendiente cero y saldo a favor separado.
- Dado un usuario sin acceso a la reparación, cuando solicita movimientos, entonces Rust deniega la consulta.

## HRL-32 [Historia] Registrar abonos (importe positivo dentro del pendiente)

Estado: Hecho (evidencia: `src/composables/usePaymentsPanel.ts::submit`, `src-tauri/src/services/ledger.rs::LedgerService::record`, `src-tauri/src/repositories/ledger.rs::LedgerRepository::insert`). Revalida saldo y `payments.add` dentro de la transacción.
Prioridad: Alta, porque registra cobros y debe impedir exceder la deuda.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante detectada; prueba de libro/reintentos en `src-tauri/src/services/tests.rs::restored_ledger_retries_exact_request_and_rolls_back_audit_failure`, sin ejecutar.
Criterios de aceptación (fuente: `docs/reglas-pagos.md`):

- Dado permiso de cobro y pendiente vigente, cuando registra un importe positivo no mayor al pendiente con método libre, entonces agrega abono y auditoría sin modificar movimientos previos.
- Dado importe cero, negativo, superior al pendiente o saldo esperado antiguo, cuando confirma, entonces rechaza sin cobrar.
- Dado permiso de devolución sin permiso de abono, cuando intenta cobrar, entonces deniega la operación.

## HRL-33 [Historia] Correcciones de abonos (con motivo, permiso de acceso completo)

Estado: Hecho (evidencia: `src-tauri/src/services/ledger.rs::LedgerService::record`, `src-tauri/src/services/access.rs::AccessService::permanent` y `require`, `src/composables/usePaymentsPanel.ts::submit`). Escribe ajuste ligado al abono original y exige `payments.adjust`, permanente de Completo o préstamo válido de quien lo posee.
Prioridad: Alta, porque modifica cobro efectivo con trazabilidad inmutable.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante detectada; pruebas del libro presentes en `src-tauri/src/services/tests.rs`, sin ejecutar.
Criterios de aceptación (fuentes: `docs/reglas-pagos.md`, `docs/acceso-temporal.md`):

- Dado un abono de 500 cuyo importe correcto es 300 y facultad de Completo, cuando corrige con motivo, entonces agrega ajuste de −200 ligado al original y conserva el abono de 500.
- Dado Limitado o Personalizado sin préstamo válido de la facultad de Completo, cuando corrige aunque pueda cobrar o devolver, entonces rechaza.
- Dado un abono ajeno a la orden, importe esperado desactualizado o corrección que produciría efectivo negativo, cuando confirma, entonces rechaza sin movimiento.

## HRL-34 [Historia] Devolución del saldo a favor (método, motivo y permiso propio)

Estado: Hecho (evidencia: `src-tauri/src/services/ledger.rs::LedgerService::record`, `src/composables/usePaymentsPanel.ts::submit`, `src-tauri/src/repositories/ledger.rs::LedgerRepository::state`). Compara el importe con el saldo a favor recalculado dentro de la transacción.
Prioridad: Alta, porque devuelve dinero y no permite devolución parcial ni duplicada.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante detectada; prueba presente `src-tauri/src/services/tests.rs::restored_edit_trash_finances_and_refund_preserve_original_books`, no ejecutada.
Criterios de aceptación (fuente: `docs/reglas-pagos.md` R02/R03):

- Dado saldo a favor de 150 y permiso de devolución, cuando confirma con método y motivo, entonces devuelve exactamente 150 y registra el movimiento explícito.
- Dada una devolución parcial o un saldo que cambió, cuando confirma, entonces se rechaza sin devolver.
- Dado Limitado con permiso de devolución pero sin permiso de abono, cuando devuelve el excedente total, entonces no se rechaza por su nivel de acceso.
- Dado un usuario que puede cobrar pero no devolver, cuando solicita devolución, entonces se deniega.

## HRL-35 [Historia] Reintento seguro de operaciones financieras (ID de solicitud y huella)

Estado: Hecho (evidencia: `src-tauri/src/services/ledger.rs::LedgerService::record`, `src-tauri/src/services/orders.rs::OrderService::create`, `src-tauri/src/repositories/ledger.rs::LedgerRepository::request`, `src/composables/usePaymentsPanel.ts::retryPending`). El libro usa ID global y huella; el panel conserva la solicitud exacta pendiente en almacenamiento local.
Prioridad: Alta, porque evita duplicar cobros, ajustes y devoluciones.
Puntos: 8
Sprint: 4
Subtareas: ninguna faltante detectada en deduplicación. Prueba presente `src-tauri/src/services/tests.rs::restored_ledger_retries_exact_request_and_rolls_back_audit_failure`, no ejecutada.
Criterios de aceptación (fuentes: `docs/reglas-pagos.md`, arquitectura objetivo y `docs/acceso-temporal.md`):

- Dado un movimiento confirmado cuyo resultado no llegó a la UI, cuando reintenta con mismo ID y contenido, entonces no duplica movimiento, auditoría ni uso del préstamo.
- Dado un ID ya usado, cuando cambia contenido o autor, entonces rechaza por conflicto.
- Dado un fallo previo a confirmar la auditoría, cuando termina la transacción, entonces no queda movimiento parcial y el reintento sigue siendo controlado.
- Dado un movimiento pendiente del panel, cuando vuelve a abrirlo, entonces puede reenviar la solicitud exacta conservada sin generar otro ID.

## HRL-36 [Historia] Registro de entrega independiente de pago y avance técnico

Estado: Hecho (evidencia: `src/composables/useReceptionForm.ts::submit` y estado `delivered`, `src-tauri/src/services/orders.rs::OrderService::edit`, `src-tauri/src/repositories/orders.rs::OrderRepository::update_order`). El campo entrega se guarda por edición autorizada de la orden.
Prioridad: Media, porque distingue custodia del equipo de deuda y trabajo técnico.
Puntos: 3
Sprint: 3
Subtareas: ninguna faltante detectada en la separación de escrituras; validación de recorrido nativo en HRL-64.
Criterios de aceptación (fuentes: contexto funcional §2, `docs/reglas-pagos.md` R04, `docs/papelera-ordenes.md`):

- Dada una orden activa con permiso de editar, cuando registra entrega, entonces cambia entrega sin crear pagos ni completar etapas.
- Dada una orden entregada con deuda, cuando consulta finanzas, entonces conserva el pendiente.
- Dado permiso insuficiente o revisión antigua, cuando cambia entrega, entonces rechaza sin modificar la orden.
- Dado pago completo o última etapa completada, cuando se registra, entonces no marca entrega automáticamente.

## HRL-37 [Historia] Nota de salida con resumen técnico público y garantías

Estado: Hecho por inspección del flujo (evidencia: `src-tauri/src/services/repairs.rs::RepairService::summary`, `src/composables/useCompanyNote.ts::prepare`, `src/components/company/CompanyNote.vue` rama de salida, `src/composables/useWorkshop.ts::print`). La impresora física queda No verificada (HRL-64).
Prioridad: Media, porque comunica trabajo público y cobertura al entregar.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante de implementación detectada; no se repite la validación nativa transversal de HRL-64.
Criterios de aceptación (fuentes: `docs/reparaciones-permisos.md`, contexto funcional §2):

- Dado seguimiento guardado, cuando solicita nota de salida, entonces espera técnico, nota pública y garantías desde SQLite antes de imprimir.
- Dadas notas internas y contraseña del equipo, cuando prepara el resumen público, entonces no las incluye.
- Dado un fallo al recuperar seguimiento o permiso insuficiente, cuando prepara la nota, entonces no emite un resumen incompleto.
- Dada una nota impresa, cuando termina la impresión, entonces no marca entrega ni pago automáticamente.

## HRL-38 [Historia] Preferencias de publicación (solo guardado local, sin portal web)

Estado: Hecho (evidencia: `src-tauri/src/domain/repair_contracts.rs::operations`, `src-tauri/src/services/repairs.rs::RepairService::save`, `src-tauri/src/repositories/repairs.rs::RepairRepository::save` escribe `seguimiento_publicacion`, `src/components/repairs/RepairWorkspace.vue` controles de publicación). No se localizó una conexión a portal en esta ruta.
Prioridad: Baja, porque prepara configuración local para una integración futura.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante detectada para el alcance local. El portal queda separado en HRL-71.
Criterios de aceptación (fuentes: `docs/reparaciones-permisos.md`, contexto funcional §§1 y 2):

- Dado permiso `publication.edit`, cuando cambia modo manual/automático y visibilidad, entonces conserva las preferencias localmente con revisión y auditoría.
- Dado acceso Limitado sin edición de publicación, cuando intenta cambiarlas, entonces Rust rechaza.
- Dadas preferencias guardadas, cuando se vuelve a abrir el seguimiento, entonces se recuperan sin afirmar que se publicó información en un portal.

Discrepancia D-07: `docs/reparaciones-permisos.md` presenta ficha JSON/migración 006 y cobros dentro del guardado; `RepairRepository::save` persiste tablas relacionadas y `operations` rechaza añadir pagos al seguimiento, que se registran por `LedgerService`.
