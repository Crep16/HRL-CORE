# HRL-39 [Epic] Módulo de Bitácora / Auditoría

Fuente de alcance vigente: `docs/backlog-base (1).md` (80 ítems, sin HRL-66). Actualización documental del 12 de noviembre de 2026. Los estados de ítems sin cambio de alcance conservan la evidencia de la revisión estática previa; no se ejecutaron pruebas.

Estado: En progreso, por detalles de cambios incompletos en HRL-42. El resto tiene implementación localizada por inspección estática.
Prioridad: Alta, porque garantiza trazabilidad y presenta resultados financieros.
Puntos: 39 (suma de historias; no duplicar al sumar sprints).
Sprint: 1 y 4.
Subtareas: agrupador de HRL-40 a HRL-47; las nuevas claves comienzan en HRL-82.

Criterios de aceptación del epic (fuentes: `docs/bitacora.md`, `docs/reglas-pagos.md`):

- Dado un cambio de negocio confirmado, cuando consulta auditoría con permiso, entonces identifica operación, autor y diferencias disponibles sin secretos.
- Dado un conjunto financiero filtrado, cuando muestra indicadores, entonces conserva separación de monedas y calcula neto, pendiente y ganancia provisional según las reglas documentadas.
- Dado fallo de auditoría o falta de autorización, cuando intenta confirmar un cambio o consultar el historial, entonces se revierte el cambio o se deniega la lectura respectivamente.

Fuente funcional principal: `docs/bitacora.md`; integridad monetaria: `docs/reglas-pagos.md`; arquitectura transaccional: `docs/arquitectura-objetivo.md`. Las pruebas existentes se leyeron, no se ejecutaron. Los puntos describen alcance completo, no trabajo restante.

## HRL-40 [Historia] Servicio de auditoría dentro de la misma transacción del cambio

Estado: Hecho (evidencia: `src-tauri/src/services/audit.rs::AuditService::record`, `src-tauri/src/repositories/audit.rs::AuditRepository::insert` y `detail`, llamados con la transacción de `OrderService::create`, `LedgerService::record`, `RepairService::save` y `CompanyService::save`). El servicio de auditoría no abre una transacción independiente.
Prioridad: Alta, porque su fallo debe impedir confirmar cualquier cambio de negocio.
Puntos: 5
Sprint: 1
Subtareas: ninguna faltante detectada en el mecanismo. Pruebas presentes, no ejecutadas: `src-tauri/src/services/tests.rs::failed_order_or_payment_audit_rolls_back_all_rows_and_the_folio_reservation` y `restored_ledger_retries_exact_request_and_rolls_back_audit_failure`.
Criterios de aceptación (fuentes: `docs/bitacora.md`, arquitectura objetivo):

- Dada una operación válida, cuando confirma, entonces cambio y evento con detalles usan la misma transacción.
- Dado fallo al insertar evento o detalle, cuando se intenta confirmar, entonces revierte el cambio de negocio completo.
- Dada auditoría persistida, cuando se intenta actualizar o borrar, entonces los triggers de inmutabilidad lo impiden.
- Dado un operador que puede modificar pero no consultar Bitácora, cuando realiza una operación, entonces la auditoría se registra igualmente.

## HRL-41 [Historia] Bitácora global con filtros por texto, autor, operación, orden y fechas

Estado: Hecho (evidencia: `src/composables/useLogbookWorkspace.ts::load` y `apply`, `src-tauri/src/services/reports.rs::ReportService::logbook`, `src-tauri/src/repositories/reports.rs::ReportRepository::logbook`). SQL parametriza los filtros, pagina de 50 en 50 y comprueba `logbook.view` en cada llamada.
Prioridad: Alta, porque permite supervisar operaciones protegidas.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante de consulta detectada. Pruebas existentes de recepción e historial en `src-tauri/src/services/tests.rs`, sin ejecutar.
Criterios de aceptación (fuente: `docs/bitacora.md`):

- Dado permiso Visualizar Bitácora, cuando combina texto literal, autor, operación, folio y fechas, entonces consulta únicamente eventos coincidentes con paginación de 50.
- Dada la zona horaria de Empresa, cuando filtra un día local, entonces convierte sus límites a UTC sin desplazar eventos entre días.
- Dado un permiso revocado o una sesión inexistente, cuando solicita otra página, entonces Rust deniega la consulta.
- Dado un rango invertido, cuando aplica filtros, entonces muestra el error y permite corregirlo.

## HRL-42 [Historia] Detalle de cambios (valores anterior y nuevo, sin contraseñas)

Estado: En progreso (evidencia: `src-tauri/src/services/orders.rs::OrderService::edit`, `src-tauri/src/services/repairs.rs::RepairService::save`, `src-tauri/src/repositories/reports.rs::ReportRepository::logbook`, `src/composables/useLogbookWorkspace.ts::changes`). Hay detalle anterior/nuevo y se omiten secretos, pero la edición persiste email y entrega estimada sin incorporarlos a `changes`; además el DTO global no expone `entidad`/`id_entidad` aunque SQL los lee.
Prioridad: Alta, porque una escritura auditada sin su diferencia completa reduce trazabilidad.
Puntos: 5
Sprint: 4
Subtareas:

- HRL-82 [Servicio Rust] Completar en `OrderService::edit` las diferencias de email y entrega estimada, y revisar cobertura de cada campo editable sin registrar el secreto del equipo.
- HRL-83 [Repositorio/SQL] Transportar entidad e identificador afectado desde la consulta de auditoría en el contrato de detalle; no alterar el libro inmutable existente.
- HRL-84 [Composable/store] Mostrar la identidad del registro y los nuevos campos de diferencia en el detalle de Bitácora.
- HRL-85 [Pruebas] Cubrir edición de email/entrega estimada, comparación anterior/nuevo y ausencia de contraseñas en todos los eventos y DTO públicos de auditoría.

Criterios de aceptación (fuente: `docs/bitacora.md`):

- Dado un evento nuevo de edición, cuando consulta el detalle, entonces identifica registro, autor, fecha, resultado, motivo cuando exista y valores anterior/nuevo de los cambios registrados.
- Dadas notas, etapas o colecciones técnicas añadidas, cuando consulta sus cambios, entonces muestra los elementos nuevos sin repetir todo el historial.
- Dada una contraseña, hash o contenido binario de logo/foto, cuando se registra o consulta auditoría, entonces no aparece ese contenido sensible.
- Dado un usuario sin Visualizar Bitácora, cuando intenta consultar detalle, entonces se deniega la consulta.

Discrepancia D-08: la cobertura de cambios anunciada por `docs/bitacora.md` no alcanza todos los campos que `OrderService::edit` modifica. El detalle global tampoco devuelve la identidad del registro afectado que exige la documentación.

## HRL-43 [Historia] Historial de una orden

Estado: Hecho (evidencia: `src-tauri/src/services/orders.rs::OrderService::history`, `src-tauri/src/repositories/orders.rs::OrderRepository::history`, `src/composables/useReceptionForm.ts::openHistory` y `loadHistory`, `src/composables/useLogbookWorkspace.ts::orderHistory`). La consulta exacta incluye órdenes en papelera y filtra nombres de campos sensibles.
Prioridad: Alta, porque permite rastrear el expediente y sus movimientos.
Puntos: 3
Sprint: 4
Subtareas: ninguna faltante del historial detectada; la exhaustividad de detalles tiene tareas en HRL-42. Prueba presente `src-tauri/src/services/tests.rs::reception_history_requires_logbook_permission_and_list_rechecks_active_users`, no ejecutada.
Criterios de aceptación (fuentes: `docs/bitacora.md`, `docs/papelera-ordenes.md`):

- Dada una orden guardada y permiso de Bitácora, cuando abre historial desde recepción o finanzas, entonces consulta eventos de esa orden exacta.
- Dada una orden en papelera, cuando consulta historial con autorización, entonces conserva sus eventos sin crear otra orden.
- Dado permiso insuficiente, cuando solicita el historial aunque pueda ver Clientes, entonces Rust lo rechaza.
- Dado un evento con un campo de secreto, cuando prepara la respuesta de historial, entonces no devuelve ese campo.

## HRL-44 [Historia] Finanzas: filtros por folio, cliente, clasificación, periodo y alcance

Estado: Hecho (evidencia: `src-tauri/src/services/reports.rs::ReportService::finances`, `src/composables/useFinancialWorkspace.ts::filtered` y `totals`, `src/utils/finances.ts::filterFinancialOrders`, `inDateRange` y `calendarDate`). El backend devuelve órdenes autorizadas y los filtros actuales se calculan en frontend. La ubicación de reglas en utils es deuda de HRL-60.
Prioridad: Alta, porque determina qué dinero y órdenes entran en los indicadores.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante funcional detectada por lectura; la corrección arquitectónica se registra en HRL-60.
Criterios de aceptación (fuente: `docs/bitacora.md`):

- Dado permiso de Bitácora, cuando filtra folio/cliente, clase, periodo y alcance, entonces muestra las órdenes coincidentes con totales del conjunto completo, no solo de la página.
- Dado periodo por movimientos, cuando hay cobros fuera del intervalo, entonces el cobrado del periodo solo incluye movimientos dentro de él y los saldos usan el acumulado.
- Dada fecha desconocida, cuando filtra un periodo, entonces no inventa fecha para incluir el movimiento.
- Dado permiso revocado, cuando actualiza finanzas, entonces Rust deniega y se retiran los datos de la consulta fallida.

## HRL-45 [Historia] Clasificación de órdenes (completadas, pagadas sin entregar, entregadas con pendiente, sin liquidar)

Estado: Hecho (evidencia: `src-tauri/src/services/reports.rs::ReportService::finances`, `src/utils/finances.ts::financialClasses` y `filterFinancialOrders`). Clasifica con pago calculado y entrega; excluye canceladas y papelera de los totales activos.
Prioridad: Alta, porque distingue deuda, custodia y órdenes completadas.
Puntos: 3
Sprint: 4
Subtareas: ninguna faltante funcional detectada; pruebas actuales no ejecutadas.
Criterios de aceptación (fuentes: `docs/bitacora.md`, `docs/reglas-pagos.md`):

- Dado cobro neto mayor o igual al precio, cuando clasifica, entonces resulta Completada si entregada o Pagada sin entregar si no entregada.
- Dado cobro neto menor al precio, cuando clasifica, entonces resulta Entregada con pendiente o Sin liquidar ni entregar según entrega.
- Dada una orden cancelada o en papelera, cuando muestra totales activos, entonces la excluye y permite consultarla por alcance de trazabilidad.
- Dado un intento de usar el estado técnico como pago, cuando calcula clasificación, entonces conserva el saldo del libro y no liquida ni devuelve automáticamente.

## HRL-46 [Historia] Seis indicadores financieros agrupados por moneda

Estado: Hecho (evidencia: `src/utils/finances.ts::financialTotals`, `src/composables/useFinancialWorkspace.ts::totals`, `src/components/logbook/FinancialWorkspace.vue` indicadores). Agrupa por código de moneda; suma total, cobrado, pendiente, costos, ganancia y ganancia de completadas, con cobros excluidos separados. El cálculo sigue en utils (HRL-60).
Prioridad: Alta, porque comunica dinero y ganancia provisional del taller.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante funcional detectada; traslado de reglas separado en HRL-60.
Criterios de aceptación (fuente: `docs/bitacora.md`):

- Dadas órdenes en monedas distintas, cuando muestra indicadores, entonces entrega grupos separados sin sumar monedas entre sí.
- Dado precio total de 1500 y costos directos de 600, cuando calcula ganancia estimada, entonces muestra 900 como provisional y antes de gastos generales.
- Dadas varias páginas de órdenes filtradas, cuando muestra totales, entonces incluye todo el conjunto filtrado y separa ganancia de completadas.
- Dadas órdenes excluidas, cuando calcula totales activos, entonces no mezcla sus importes y muestra sus cobros aparte para trazabilidad.

## HRL-47 [Historia] Registro de gastos directos (descripción, importe, motivo y autorización)

Estado: Hecho (evidencia: `src-tauri/src/services/reports.rs::ReportService::expense`, `src-tauri/src/repositories/reports.rs::ReportRepository::expense`, `src/composables/useFinancialWorkspace.ts::addCost`, `src-tauri/migrations/001_esquema_objetivo.sql::gastos_directos_no_update` y `gastos_directos_no_delete`). Comprueba `logbook.view` y `repairs.edit` en ámbito Bitácora, además de orden activa no cancelada.
Prioridad: Alta, porque registra egresos usados para calcular ganancia.
Puntos: 8
Sprint: 4
Subtareas: ninguna faltante de implementación detectada; pruebas del resumen financiero presentes en `src-tauri/src/services/tests.rs::restored_edit_trash_finances_and_refund_preserve_original_books`, no ejecutadas.
Criterios de aceptación (fuentes: `docs/bitacora.md`, `docs/acceso-temporal.md`):

- Dados Visualizar Bitácora y Editar Reparaciones, cuando registra concepto, importe positivo y motivo sobre orden activa, entonces crea gasto inmutable y auditoría en una transacción.
- Dado permiso de Bitácora sin permiso adicional de edición, cuando intenta agregar gasto, entonces Rust lo deniega.
- Dado importe inválido, concepto o motivo vacío, orden excluida o fallo de auditoría, cuando registra, entonces no confirma el gasto.
- Dado gasto confirmado, cuando calcula resultados, entonces se agrega a los costos junto con refacciones y reduce la ganancia estimada.

Discrepancia D-09: `docs/bitacora.md` dice que conserva el nombre del autor en el momento del evento; `AuditRepository::insert` guarda solo `id_autor` y `ReportRepository::logbook` resuelve el nombre actual por JOIN. Conservar nombres históricos requiere decisión explícita, no se agrega una copia por iniciativa de esta revisión.
