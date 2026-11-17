# Estado del backlog HRL Proyecto

Corte: 17 de noviembre de 2026. Revisión estática del árbol de trabajo; no se ejecutaron la aplicación, builds ni pruebas. Hecho significa respaldo en código de los criterios y reglas revisados, no certificación nativa. La existencia de una prueba no acredita que haya pasado. Las discrepancias de reglas se reportan aun cuando los criterios explícitos estén respaldados. Verificado en el CSV significa dictamen obtenido por lectura; No verificado exige ejecución o validación externa.

Fuentes originales leídas: docs/hrl-proyecto-backlog.md y docs/hrl-proyecto-jira.csv. Las rutas homónimas pedidas bajo docs/backlog/ no existen. docs/contexto-vistas-modulos.md tampoco existe: se leyó docs/contexto-funcional-vistas-y-estructura.md. Se leyeron AGENTS.md, arquitectura-objetivo.md y las seis especificaciones de negocio pedidas. La documentación se usa para contrastar; los dictámenes se basan en el código actual. Los cambios locales previos en documentación se consideran parte del árbol leído y no se modifican. Inventario de existencias excluido.

C1, C2, C3 siguen el orden de criterios del original. «Sin respaldo» diferencia ausencia o discrepancia de lo que requiere una comprobación externa. Las referencias sin función corresponden a plantillas, DDL o módulos declarativos.

## HRL-1 · Base técnica y arquitectura

### HRL-10 Esquema SQLite normalizado y migraciones

Estado: **En progreso**. Sprint 1 · 8 puntos.

Evidencia: `src-tauri/src/db/migrations.rs::migrator/apply`; `src-tauri/src/db/connection.rs::Database::open_file/connect`; `src-tauri/src/commands/mod.rs::BackendState::services`; `src-tauri/migrations/001_esquema_objetivo.sql::abonos_no_update/abonos_no_delete/devoluciones_no_update/auditoria_no_delete`.

Respaldo de criterios: C2: detecta Ordenes con datos_json y devuelve LegacyDatabase. C3: triggers BEFORE UPDATE/DELETE abortan mutaciones. C1: crea el archivo normalizado, pero aplica tres migraciones.

Criterios no cumplidos o sin verificar: C1: no aplica «solo 001 y 002»; incluye 003_antecedentes_equipo.sql.

Discrepancias: El backlog y la sección de cierre del cargador en arquitectura-objetivo.md describen 001–002; el cargador activo incluye 003.

### HRL-11 Arquitectura backend en capas

Estado: **Hecho**. Sprint 1 · 8 puntos.

Evidencia: `src-tauri/src/commands/mod.rs::create_order`; `src-tauri/src/services/orders.rs::OrderService::new/create`; `src-tauri/src/domain/money.rs::Money`; `src-tauri/src/services/access.rs::AccessService::require`; `src-tauri/src/db/migrations.rs::apply`.

Respaldo de criterios: C1: los comandos delegan y no contienen SQL. C2: Money encapsula i64 y los DTO usan price_minor. C3: los servicios llaman a require dentro de transacciones.

Criterios no cumplidos o sin verificar: Ninguno de los tres criterios explícitos en la lectura estática.

Discrepancias: AGENTS.md exige SQL exclusivamente en repositories; la detección del esquema anterior está en db/migrations.rs. Los tres criterios de aceptación de HRL-11 sí tienen respaldo, por eso se clasifica Hecho; la excepción arquitectónica queda reportada sin modificar código.

### HRL-12 Arquitectura frontend MVVM con Pinia

Estado: **Hecho**. Sprint 1 · 5 puntos.

Evidencia: `src/services/api/transport.ts::request`; `src/stores/useAuthStore.ts::clearSession/assertSessionVersion`; `src/stores/useOrderStore.ts::listOrders/acceptOrder`; `src/stores/useCompanyStore.ts::load`; `src/stores/useUsersStore.ts::load`; `src/main.ts`.

Respaldo de criterios: C1: la búsqueda de invoke en src solo encuentra transport.ts. C2: watches de sessionVersion limpian datos. C3: versiones de sesión impiden aceptar respuestas antiguas.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: El contexto funcional antiguo describe llamadas IPC dispersas y ausencia de Pinia; src/services/api y src/stores ya implementan la separación.

### HRL-13 Contrato IPC: errores tipificados y control de revisión

Estado: **Hecho**. Sprint 1 · 5 puntos.

Evidencia: `src-tauri/src/error.rs::AppError::code/serialize`; `src/services/api/errors.ts::normalizeIpcError`; `src-tauri/src/services/orders.rs::OrderService::edit`; `src/composables/usePaymentsPanel.ts::submit/retryPending`; `src/stores/useOrderStore.ts::acceptOrder`.

Respaldo de criterios: C1: códigos VALIDATION_ERROR, FORBIDDEN, SESSION_CHANGED, CONFLICT y RESULT_UNCERTAIN reconocidos por la UI. C2: expected_revision se compara antes del cambio. C3: se conserva el sobre financiero exacto en localStorage y se reutiliza.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: El contrato contiene más códigos que las cinco categorías resumidas en el backlog; no implica pérdida de esas categorías.

### HRL-14 Custodia de claves con Windows DPAPI

Estado: **No verificado**. Sprint 1 · 5 puntos.

Evidencia: `src-tauri/src/services/key_store.rs::OsKeyStore::open_key/protect`; `src-tauri/src/services/crypto.rs::CryptoService`; `src-tauri/src/services/mod.rs::BackendServices::with_os_key_store`.

Respaldo de criterios: C1: clave AES en archivo DPAPI externo; SQLite conserva el sobre. C2: ausencia/corrupción de custodia devuelve Crypto y no habilita texto claro. C3: CryptProtectData/CryptUnprotectData usa contexto de usuario.

Criterios no cumplidos o sin verificar: C3 no verificado: rechazo del descifrado desde otro perfil Windows.

Discrepancias: El respaldo portable ahora tiene código propio; copiar solamente SQLite sigue sin incluir archivos y custodia.

Para verificar: Ejecutar instalación Windows con dos perfiles; guardar una credencial y comprobar recuperación en el original y rechazo en el segundo, incluida ausencia/corrupción del archivo DPAPI.

### HRL-15 Retirar código anterior y cerrar la migración

Estado: **Hecho**. Sprint 4 · 5 puntos.

Evidencia: `src-tauri/src/lib.rs::run`; `src-tauri/src/commands/mod.rs::BackendState::services`; `src/WorkshopApp.vue::<script setup>/<template>`; `src/views/`; `legacy/frontend/`; `src-tauri/legacy/`.

Respaldo de criterios: C1: el registro productivo solo incluye commands normalizados; módulos antiguos bajo cfg(test). C2: la ruta activa abre hrl_core_3fn.db, sin inspección de hrl_datos.db. C3: WorkshopApp compone vistas y delega a useWorkshop.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: El backlog presenta el retiro como pendiente; el árbol conserva el código histórico archivado, fuera del flujo productivo. El adaptador utils/legacyOrder.ts transforma DTO, no usa la base anterior.

## HRL-2 · Usuarios, acceso y permisos

### HRL-16 Configuración inicial: clave maestra y primer administrador

Estado: **Hecho**. Sprint 1 · 5 puntos.

Evidencia: `src/composables/useAccessForm.ts::view/check/submit`; `src-tauri/src/services/access.rs::AccessService::initialize`; `src-tauri/src/repositories/access.rs::bootstrap/setup_status`.

Respaldo de criterios: C1: setupComplete falso selecciona configuración, no login. C2: bootstrap y auditoría usan la misma transacción BEGIN IMMEDIATE. C3: bootstrap asigna rol administrator y nivel complete activos.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-17 Inicio y cierre de sesión

Estado: **Hecho**. Sprint 1 · 5 puntos.

Evidencia: `src-tauri/src/services/access.rs::AccessService::login/logout/Attempts::fail`; `src-tauri/src/repositories/access.rs::by_login/by_id`; `src/stores/useAuthStore.ts::clearSession/logoutUser`; `src/composables/useAccessForm.ts::clearSecrets`; `src/stores/useOrderStore.ts::acceptOrder`.

Respaldo de criterios: C1: by_login solo devuelve usuarios activos. C2: logout retira sesión backend y clearSession invalida stores; los formularios se desmontan y limpian secretos. C3: sessionVersion impide aceptar respuestas de la sesión anterior. Se mantiene espera de 5 segundos tras 5 fallos.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-18 Perfil de ventana según estado

Estado: **No verificado**. Sprint 1 · 2 puntos.

Evidencia: `src-tauri/src/windowing.rs::Profile::size/minimum/for_session/WindowProfileState::apply_to`; `src/composables/useWindowProfile.ts::useWindowProfile`; `src-tauri/src/services/access.rs::AccessService::synchronize_window`.

Respaldo de criterios: C1: perfiles declarados 1000×680, 1000×760 y 1280×760; taller mínimo 1120×680. C2: perfil depende de la sesión backend. Las pruebas de windowing usan Window simulado.

Criterios no cumplidos o sin verificar: C1 y C2 sin verificación sobre una ventana Tauri real.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

Para verificar: Abrir configuración, acceso y taller en Windows; maximizar taller, cerrar sesión y medir dimensiones, límites y capacidad de redimensionar.

### HRL-19 Niveles de acceso y permisos por apartado y acción

Estado: **Hecho**. Sprint 1 · 8 puntos.

Evidencia: `src-tauri/src/domain/permissions.rs::OPERATIONS/limited_allows`; `src-tauri/src/services/access.rs::AccessService::permanent/require`; `src-tauri/src/services/users.rs::UserService::save`.

Respaldo de criterios: C1: limited_allows restringe acciones y require rechaza fuera de las facultades. C2: payments.add se obtiene de permiso separado; repairs.edit no lo concede a custom. C3: save audita permisos anteriores/nuevos en la transacción.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-20 Gestión de usuarios

Estado: **Hecho**. Sprint 2 · 8 puntos.

Evidencia: `src-tauri/src/services/users.rs::UserService::save`; `src-tauri/migrations/001_esquema_objetivo.sql::usuarios_conservar_admin_al_editar/usuarios_conservar_admin_al_borrar`; `src-tauri/src/repositories/users.rs::UserRepository::save`.

Respaldo de criterios: C1: servicio y triggers impiden desactivar o degradar al último administrador. C2: verify_master precede a save. C3: no hay comando de eliminación de cuentas; las FK referenciadas usan RESTRICT. Contraseña vacía conserva hash y auditoría usa credentialsUpdated.

Criterios no cumplidos o sin verificar: Ninguno de los tres criterios explícitos en la lectura estática.

Discrepancias: La regla transversal del backlog exige revisión en toda edición; SaveUserPayload y UserService::save no incluyen expected_revision. Los criterios específicos de HRL-20 sí tienen respaldo; queda una brecha transversal que debe planificarse.

### HRL-21 Acceso temporal

Estado: **Hecho**. Sprint 2 · 8 puntos.

Evidencia: `src-tauri/src/services/access.rs::AccessService::temporary/require/finish_temporary/logout`; `src/composables/useWorkshop.ts::navigate/finishTemporary`; `src-tauri/src/services/users.rs::UserService::save`.

Respaldo de criterios: C1: temporary rechaza autoautorización y facultades que no posee el autorizador. C2: require compara scope y order_id y reconsulta al autorizador. C3: el préstamo no modifica usuario_permiso; cierre retira grant. Usuarios mantiene verify_master.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: El documento acceso-temporal.md describe tablas históricas y triggers 011; el flujo vigente usa accesos_temporales/acceso_temporal_permiso y AuditService.

### HRL-22 Recuperación de la clave maestra con soporte

Estado: **Hecho**. Sprint 2 · 5 puntos.

Evidencia: `src-tauri/src/services/recovery.rs::RecoveryState::issue_at/consume_at`; `src-tauri/src/services/access.rs::AccessService::verify_recovery_signature`; `src-tauri/src/services/tests.rs::recovery_resets_both_hashes_revokes_access_and_keeps_device_encryption_key`.

Respaldo de criterios: C1: TTL de 15 minutos y pending.take impiden vencidos/reuso. C2: firma Ed25519 válida habilita reset de hashes y auditoría en una transacción. C3: RecoveryState vive en memoria y se inicia vacío. Se firma token exacto; no hay envío automático de correo.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática del protocolo; el acto de firma de soporte real no se certifica aquí.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

## HRL-3 · Registro de clientes y recepción de órdenes

### HRL-23 Crear orden y asignar folio

Estado: **Hecho**. Sprint 2 · 8 puntos.

Evidencia: `src-tauri/src/services/orders.rs::OrderService::create`; `src-tauri/src/repositories/orders.rs::reserve_folio/insert_order`; `src-tauri/src/services/tests.rs::concurrent_receipts_use_distinct_folios_and_audit_authorship/failed_order_or_payment_audit_rolls_back_all_rows_and_the_folio_reservation`.

Respaldo de criterios: C1: reserva folio en backend con secuencia y UNIQUE. C2: creación, abono inicial y auditoría comparten BEGIN IMMEDIATE. C3: incremento atómico de secuencia y comprobación de colisión evitan compartir folio.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-24 Ficha de recepción

Estado: **Hecho**. Sprint 2 · 5 puntos.

Evidencia: `src/components/orders/OrderForm.vue::<template>`; `src/components/orders/OrderSidePanel.vue::<template>`; `src/composables/useReceptionForm.ts::createReceptionForm/submit`; `src-tauri/src/services/orders.rs::validate`.

Respaldo de criterios: C1: required, parseMinorUnits y validación Rust rechazan faltantes e importes inválidos. C2: readonly depende de canEdit y editar es explícito. C3: panel separa delivered y estado de pago.

Criterios no cumplidos o sin verificar: Ninguno de los tres criterios explícitos en la lectura estática.

Discrepancias: La regla de ficha pide golpes y líquidos independientes en pantalla; OrderForm usa Antecedentes del equipo unificado mediante historyNotes. Backend conserva los campos anteriores y añade history.notes con la migración 003.

### HRL-25 Listado lateral, búsqueda y consulta por folio

Estado: **Hecho**. Sprint 2 · 5 puntos.

Evidencia: `src-tauri/src/repositories/orders.rs::OrderRepository::list/find_id`; `src-tauri/src/services/orders.rs::OrderService::find`; `src/stores/useOrderStore.ts::listOrders`; `src/composables/useReceptionForm.ts::refreshList/goToOrder/lookup`; `src-tauri/src/error.rs::AppError::fmt`.

Respaldo de criterios: C1: SQL filtra según all/order/client/serial y frontend pide páginas de cinco. C2: listVersion y sessionVersion descartan respuestas antiguas. C3: referencia inexistente produce NOT_FOUND con mensaje de registro no disponible. goToOrder cruza páginas.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-26 Edición de orden con control de revisión

Estado: **Hecho**. Sprint 2 · 5 puntos.

Evidencia: `src-tauri/src/services/orders.rs::OrderService::edit`; `src/composables/useReceptionForm.ts::cancel/discard/submit`; `src/views/ReceptionView.vue::discardDialog`.

Respaldo de criterios: C1: compara revisión antes de actualizar. C2: cancel abre diálogo de descarte cuando hasChanges. C3: cambio de datos crea nuevo cliente/equipo y reasocia solo esta orden. La edición rechaza initial_payment.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-27 Contraseña del equipo cifrada

Estado: **Hecho**. Sprint 2 · 5 puntos.

Evidencia: `src-tauri/src/services/crypto.rs::CryptoService::encrypt/decrypt`; `src-tauri/src/services/orders.rs::OrderService::get/create/history`; `src-tauri/src/repositories/orders.rs::insert_credential/load_any`; `src/composables/useReceptionForm.ts::togglePassword`; `src/stores/useOrderStore.ts::acceptOrder`; `src/components/company/CompanyNote.vue`.

Respaldo de criterios: C1: DTO de listado no transporta clave, auditoría omite secreto y plantilla de nota no la imprime. C2: include_device_password explícito y require de clients.view o repairs.view. C3: credenciales_dispositivos guarda sobre AES-256-GCM, no texto claro.

Criterios no cumplidos o sin verificar: Ninguno de los criterios de implementación en la lectura estática. La custodia con otro usuario Windows se verifica en HRL-14/63.

Discrepancias: AGENTS.md aún presenta cifrar/hashear/retirar como pendiente; arquitectura-objetivo.md confirma cifrado y CryptoService ya lo implementa. No existe un permiso exclusivo de revelado: se utiliza el permiso de consulta de la orden.

### HRL-28 Guardas de borrador

Estado: **En progreso**. Sprint 2 · 3 puntos.

Evidencia: `src/composables/useReceptionForm.ts::prepareLeave/cancel`; `src/composables/useWorkshop.ts::prepareLeave/navigate/logout`; `src/composables/useRepairScreen.ts::save`.

Respaldo de criterios: C2: salir de Reparaciones espera save y bloquea si devuelve false. C1 parcialmente: detecta cambios en recepción/empresa e impide salir.

Criterios no cumplidos o sin verificar: C1: al cambiar de apartado o cerrar sesión no abre una elección para guardar o descartar. Recepción y Empresa muestran un aviso y bloquean; el descarte de recepción existe al pulsar Cancelar.

Discrepancias: El backlog exige solicitar guardar o descartar al navegar; useWorkshop::prepareLeave y createReceptionForm::prepareLeave solo devuelven false con aviso.

### HRL-29 Evidencias fotográficas: captura e incorporación

Estado: **Hecho**. Sprint 2 · 8 puntos.

Evidencia: `src-tauri/src/domain/evidence_contracts.rs::validate`; `src-tauri/src/services/evidence.rs::EvidenceService::add`; `src/composables/useOrderEvidence.ts::checkPhoto/importFiles/upload`; `src/composables/useEvidenceCamera.ts::enable/stop/take`.

Respaldo de criterios: C1: valida MIME, firma y límite 20 MiB en Rust; frontend además decodifica imagen. C2: ledger.active rechaza orden en papelera. C3: importFiles procesa por archivo, conserva válidas y acumula fallos. Cámara solicita audio:false y detiene tracks; funcionamiento físico reservado a HRL-63.

Criterios no cumplidos o sin verificar: Ninguno de los tres criterios explícitos en la lectura estática.

Discrepancias: evidencias.md describe Evidencias_orden y migración histórica 012/013; el flujo actual usa evidencias y archivos bajo assets-3fn/evidencias.

### HRL-30 Evidencias: descripción, retiro y restauración

Estado: **En progreso**. Sprint 3 · 5 puntos.

Evidencia: `src-tauri/src/services/evidence.rs::EvidenceService::list/read/update`; `src-tauri/src/repositories/evidence.rs::EvidenceRepository::update`; `src/composables/useOrderEvidence.ts::remove/toggleRemoved`.

Respaldo de criterios: C1: update cambia retirada sin borrar bytes y admite restaurar. C3: note.chars().count()>500 se rechaza. C2 parcialmente: retirar/restaurar y leer retiradas exigen evidence.delete.

Criterios no cumplidos o sin verificar: C2 literal «sin permiso de Borrar no ve evidencias» no se cumple para fotos activas: se ven con evidence.view. Si se pretendía decir «no ve evidencias retiradas», esa versión sí tiene respaldo.

Discrepancias: El criterio C2 contradice la regla de la misma historia que permite consultar con Visualizar Clientes. Código y evidencias.md distinguen activas de retiradas. Se marca En progreso con la redacción literal, sin resolver la ambigüedad ni cambiar la regla.

### HRL-31 Papelera de órdenes

Estado: **Hecho**. Sprint 3 · 5 puntos.

Evidencia: `src-tauri/src/services/orders.rs::OrderService::trash/list`; `src-tauri/src/repositories/orders.rs::OrderRepository::trash/load_any`; `src/views/ReceptionView.vue::<template>`; `src-tauri/src/services/evidence.rs::EvidenceService::add`; `src-tauri/src/services/repairs.rs::RepairService::load`.

Respaldo de criterios: C1: trash solo marca eliminada_en e incrementa revisión; conserva expediente y libros. C2: deleted=false restaura y audita orders.restore. C3: no escribe en tablas financieras ni altera saldos.

Criterios no cumplidos o sin verificar: Ninguno de los tres criterios explícitos en la lectura estática.

Discrepancias: La UI oculta Papelera sin orders.delete, pero OrderService::list con trashed=true solo exige clients.view; la restricción de consultar papelera no está comprobada con el mismo permiso en backend.

## HRL-4 · Reparaciones, pagos y entrega

### HRL-32 Abrir reparación sobre la orden seleccionada

Estado: **Hecho**. Sprint 3 · 3 puntos.

Evidencia: `src/views/RepairsView.vue::<template>`; `src/composables/useWorkshop.ts::navigate`; `src-tauri/src/services/repairs.rs::RepairService::load`; `src-tauri/src/repositories/ledger.rs::LedgerRepository::active`.

Respaldo de criterios: C1: recibe order seleccionado y presenta cliente, artículo, serie, marca y modelo. C2: load exige active; una orden en papelera se rechaza antes de trabajar.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-33 Técnico, notas y visibilidad de publicación

Estado: **Hecho**. Sprint 3 · 5 puntos.

Evidencia: `src-tauri/src/services/repairs.rs::RepairService::summary/save`; `src-tauri/src/repositories/repairs.rs::RepairRepository::save`; `src-tauri/src/domain/repair_contracts.rs::PublicSummary/Repair`; `src/views/RepairsView.vue::<template>`.

Respaldo de criterios: C1: PublicSummary contiene technician, public_note y warranties, sin internal_note. C2: save persiste modo y visibilidad en SQLite y no realiza ninguna llamada de red.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: reparaciones-permisos.md describe migración 006 y importación localStorage anteriores; el servicio activo usa tablas normalizadas y no importa aquel seguimiento.

### HRL-34 Etapas y avances

Estado: **Hecho**. Sprint 3 · 5 puntos.

Evidencia: `src/composables/useRepairScreen.ts::complete/addEvent`; `src-tauri/src/services/repairs.rs::RepairService::save`; `src-tauri/src/repositories/repairs.rs::RepairRepository::save/load`.

Respaldo de criterios: C1: complete incrementa completed y agrega avance; persistencia solo modifica seguimiento/etapas, sin entrega ni libro financiero. C2: INSERT avances guarda fecha_evento, id_autor y visible_cliente; admite etapas personalizadas sin cambiar catálogo.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: Repair::default inicia solo Recibido y Entregado, no la secuencia sugerida completa. El catálogo de opciones sí está disponible y se incorporan etapas al agregar avances. El DTO Event no devuelve id_autor, aunque SQLite lo conserva.

### HRL-35 Refacciones y garantías

Estado: **Hecho**. Sprint 3 · 5 puntos.

Evidencia: `src/composables/useRepairScreen.ts::addPart/selectWarrantyType/addWarranty`; `src/services/api/workshop.ts::saveRepair`; `src-tauri/src/repositories/repairs.rs::RepairRepository::save`; `src-tauri/src/repositories/reports.rs::ReportRepository::finances`.

Respaldo de criterios: C1: costo de refacciones se suma en finances; transporte convierte unidades a centavos enteros. C2: días iniciales y selectWarrantyType leen partsDays/laborDays de Empresa.

Criterios no cumplidos o sin verificar: Ninguno de los dos criterios explícitos en la lectura estática.

Discrepancias: La regla del backlog pide pieza, cantidad y costo; Part y la captura solo ofrecen nombre y precio. El INSERT no recibe cantidad (se usa el valor predeterminado del esquema). No se evalúan existencias.

### HRL-36 Permisos por operación y revisión en el guardado

Estado: **Hecho**. Sprint 3 · 5 puntos.

Evidencia: `src-tauri/src/domain/repair_contracts.rs::operations`; `src-tauri/src/services/repairs.rs::RepairService::save`; `src-tauri/src/services/access.rs::AccessService::require/permanent`.

Respaldo de criterios: C1: cambio de técnico requiere repairs.edit, ausente en limited_allows. C2: revisión se compara antes de guardar. operations distingue añadidos de reemplazos y prohíbe borrar registros previos; pagos usan su libro separado.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-37 Registro de entrega

Estado: **Hecho**. Sprint 3 · 3 puntos.

Evidencia: `src-tauri/src/services/orders.rs::OrderService::edit`; `src-tauri/src/repositories/orders.rs::OrderRepository::update_order`; `src-tauri/src/repositories/ledger.rs::LedgerRepository::state`; `src/components/orders/OrderSidePanel.vue`.

Respaldo de criterios: C1: update_order persiste delivered sin modificar abonos ni precio; la deuda sigue calculada por vista. C2: paid se deriva de pendiente y no escribe delivered.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: La regla admite entrega histórica sin fecha; update_order completa fecha_entrega con fecha UTC actual cuando delivered=true y fecha era NULL, incluso al editar otro dato. No permite conservar esa fecha desconocida durante dicho guardado.

### HRL-38 Panel de pagos y cálculo de saldos

Estado: **Hecho**. Sprint 3 · 5 puntos.

Evidencia: `src-tauri/migrations/001_esquema_objetivo.sql::vista_cobros_orden/vista_saldos_orden`; `src-tauri/src/repositories/ledger.rs::LedgerRepository::state/movements`; `src/components/repairs/PaymentsPanel.vue::<template>`.

Respaldo de criterios: C1: max(precio-neto,0) y max(neto-precio,0) producen 0 pendiente y 15000 centavos a favor para 75000 cobrados/60000 precio. C2: panel muestra tres abonos recientes y libro completo desplegable.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-39 Registrar abonos (incluye el abono inicial)

Estado: **Hecho**. Sprint 3 · 5 puntos.

Evidencia: `src-tauri/src/services/ledger.rs::LedgerService::record`; `src-tauri/src/services/orders.rs::OrderService::create`; `src-tauri/src/repositories/orders.rs::initial_payment`; `src-tauri/src/repositories/ledger.rs::LedgerRepository::state`.

Respaldo de criterios: C1: receipt positivo y <=pendiente recalculado en transacción. C2: create comprueba payments.add si hay initial_payment y rechaza la creación completa si falta permiso. C3: tras insertar vuelve a consultar la vista de saldos.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-40 Correcciones de abonos

Estado: **Hecho**. Sprint 3 · 3 puntos.

Evidencia: `src-tauri/src/services/ledger.rs::LedgerService::record`; `src-tauri/src/repositories/ledger.rs::LedgerRepository::insert`; `src-tauri/src/services/access.rs::AccessService::permanent/require`; `src/composables/usePaymentsPanel.ts::submit`.

Respaldo de criterios: C1: frontend calcula diferencia y backend inserta ajustes_abono ligado al abono; no actualiza el original. C2: payments.adjust solo para complete o préstamo de quien tiene la facultad; motivo obligatorio.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-41 Devolución del saldo a favor

Estado: **Hecho**. Sprint 3 · 3 puntos.

Evidencia: `src-tauri/src/services/ledger.rs::LedgerService::record`; `src-tauri/src/repositories/ledger.rs::LedgerRepository::insert/state`; `src-tauri/migrations/001_esquema_objetivo.sql::vista_saldos_orden`.

Respaldo de criterios: C1: inserta devolución por crédito vigente y resta del neto; tras 15000 devueltos crédito queda cero. C2: amount_minor distinto de credit_minor se rechaza. Exige método, motivo y payments.refund.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-42 Reintento seguro de operaciones financieras

Estado: **Hecho**. Sprint 3 · 5 puntos.

Evidencia: `src-tauri/src/services/ledger.rs::LedgerService::record`; `src-tauri/src/repositories/ledger.rs::LedgerRepository::request/insert`; `src/composables/usePaymentsPanel.ts::submit/retryPending`; `src-tauri/src/services/orders.rs::OrderService::create`.

Respaldo de criterios: C1: solicitud existente con mismo autor y huella devuelve saldo sin insertar. C2: misma clave con huella distinta devuelve Conflict. UI guarda solicitud exacta en hrl.3fn.money.pending.<orden> antes de enviarla.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: La persistencia de reintento en localStorage corresponde a movimientos del panel. El abono inicial conserva el payload solo en memoria de createReceptionForm; al cerrar se pierde y la UI pide mantener abierta la ventana. El backlog HRL-13/42 no distingue esta limitación.

## HRL-5 · Bitácora, auditoría y finanzas

### HRL-43 Servicio de auditoría transaccional

Estado: **Hecho**. Sprint 1 · 8 puntos.

Evidencia: `src-tauri/src/services/audit.rs::AuditService::record`; `src-tauri/src/repositories/audit.rs::AuditRepository::insert/detail`; `src-tauri/src/services/users.rs::UserService::save`; `src-tauri/migrations/001_esquema_objetivo.sql::auditoria_no_update/auditoria_no_delete`; `src-tauri/src/services/tests.rs::injected_audit_failure_rolls_back_repairs_expenses_and_evidence_including_its_file`.

Respaldo de criterios: C1: record recibe la conexión de la misma transacción; error se propaga antes de commit y provoca rollback. C2: usuarios registra credentialsUpdated, sin contraseña/hash. C3: triggers bloquean UPDATE y DELETE de evento y detalle.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: AuditService no filtra por sí mismo campos secretos: cada servicio construye su lista de cambios. Las escrituras revisadas omiten claves y hashes; no se presume un filtro global que no existe.

### HRL-44 Bitácora global e historial de una orden

Estado: **Hecho**. Sprint 4 · 8 puntos.

Evidencia: `src-tauri/src/services/reports.rs::ReportService::logbook`; `src-tauri/src/repositories/reports.rs::ReportRepository::logbook`; `src-tauri/src/services/orders.rs::OrderService::history`; `src-tauri/src/repositories/orders.rs::history`; `src/composables/useLogbookWorkspace.ts::load/date`.

Respaldo de criterios: C1: author_id y límites UTC se enlazan al SQL; frontend convierte límites con zona de Empresa. C2: servicio exige logbook.view. C3: historial consulta auditoria por id_orden con detalle anterior/nuevo y paginación.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: bitacora.md describe tablas históricas separadas; el código consulta auditoria/auditoria_detalle. El nombre mostrado procede de usuarios.nombre mediante JOIN: renombrar una cuenta cambia el nombre mostrado en eventos antiguos, no existe snapshot histórico.

### HRL-45 Finanzas: filtros y clasificación de órdenes

Estado: **Hecho**. Sprint 4 · 8 puntos.

Evidencia: `src-tauri/src/services/reports.rs::ReportService::finances`; `src/models/finances.ts::filterFinancialOrders/financialTotals`; `src/composables/useFinancialWorkspace.ts::useFinancialWorkspace`.

Respaldo de criterios: C1: paid=true y delivered=true clasifica completed. C2: dateBasis payment suma únicamente movimientos dentro de fechas. C3: excluded aparta retiradas/canceladas de totales activos y conserva excludedCollected.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-46 Indicadores financieros por moneda

Estado: **Hecho**. Sprint 4 · 5 puntos.

Evidencia: `src/models/finances.ts::financialTotals`; `src/composables/useFinancialWorkspace.ts::filtered/totals/visible`; `src-tauri/src/services/reports.rs::ReportService::finances`; `src/views/FinancesView.vue`.

Respaldo de criterios: C1: Map agrupa por currency sin conversión. C2: totals recibe filtered completo, mientras visible aplica paginación después. Incluye precio, neto, pendiente, costos, ganancia provisional y ganancia de completadas.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-47 Gastos directos

Estado: **Hecho**. Sprint 4 · 3 puntos.

Evidencia: `src-tauri/src/services/reports.rs::ReportService::expense`; `src-tauri/src/repositories/reports.rs::ReportRepository::expense/finances`; `src-tauri/migrations/001_esquema_objetivo.sql::gastos_directos_no_update/gastos_directos_no_delete`.

Respaldo de criterios: C1: rango de amount_minor empieza en 1. C2: load_any(...false) exige activa y se rechaza estado que empieza con cancelad. Exige logbook.view y repairs.edit; movimiento y auditoría en la misma transacción.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

## HRL-6 · Empresa y documentos impresos

### HRL-48 Configuración de empresa: borrador, revisión y auditoría

Estado: **En progreso**. Sprint 1 · 5 puntos.

Evidencia: `src/composables/useCompanyWorkspace.ts::save/discard/hasPending`; `src/composables/useWorkshop.ts::prepareLeave`; `src-tauri/src/services/company.rs::CompanyService::save`; `src-tauri/src/repositories/company.rs::CompanyRepository::save`.

Respaldo de criterios: C2: expected_revision se compara con empresa.revision. C3: save calcula diferencias y llama AuditService dentro de transacción; excluye bytes de logo. C1 parcialmente: detecta borrador y bloquea salida.

Criterios no cumplidos o sin verificar: C1: salir de Empresa no abre elección guardar/descartar; muestra «Guarda o descarta…» y bloquea navegación. Guardar y Descartar son acciones dentro de Empresa.

Discrepancias: La guarda declarada en el backlog implica una pregunta al salir; useWorkshop::prepareLeave solo informa y bloquea. El borrador y el descarte explícito sí existen.

### HRL-49 Región, impuestos y moneda

Estado: **Hecho**. Sprint 1 · 3 puntos.

Evidencia: `src-tauri/src/services/orders.rs::OrderService::edit`; `src-tauri/src/repositories/company.rs::CompanyRepository::save`; `src/composables/useLogbookWorkspace.ts::date/load`; `src/composables/useCompanyNote.ts::money`.

Respaldo de criterios: C1: guardar Empresa cambia predeterminado, no las órdenes; editar orden rechaza cambio de currency. C2: fechas y límites de Bitácora utilizan company.timezone y locale.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-50 Folios y garantías sugeridas

Estado: **Hecho**. Sprint 1 · 3 puntos.

Evidencia: `src-tauri/src/repositories/company.rs::CompanyRepository::save`; `src-tauri/src/repositories/orders.rs::OrderRepository::reserve_folio`; `src-tauri/src/services/orders.rs::OrderService::create`; `src/composables/useReceptionForm.ts::begin`.

Respaldo de criterios: C1: begin toma prefix de Empresa para nuevas órdenes; no renombra folios anteriores. C2: reserva incrementa secuencia atómicamente bajo BEGIN IMMEDIATE y evita folios existentes.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-51 Datos generales y logo

Estado: **Hecho**. Sprint 2 · 5 puntos.

Evidencia: `src/composables/useCompanyWorkspace.ts::readLogo/save`; `src-tauri/src/services/company.rs::CompanyService::save_logo/save/read_logo`; `src/components/company/CompanyNote.vue::<template>`.

Respaldo de criterios: C1: readLogo actualiza draft.logo y logoPrint; CompanyNote usa settings.logo. C2: nombre requerido en backend y companyErrors bloquea formulario incompleto. Logo se almacena como archivo PNG/JPG de hasta 1 MiB.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática de la vista previa.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-52 Cláusulas y textos de las notas

Estado: **Hecho**. Sprint 4 · 5 puntos.

Evidencia: `src/composables/useCompanyClauses.ts::move/apply`; `src/composables/useCompanyNote.ts::clauses`; `src/components/company/CompanyNote.vue::<template>`; `src-tauri/src/repositories/company.rs::CompanyRepository::load/save`.

Respaldo de criterios: C1: cláusulas se filtran por entry/exit. C2: move altera el array y la vista previa conserva ese orden; repositorio guarda posicion y carga ORDER BY posicion,id.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: La posición pie/reverso es global (settings.clausePosition), no individual por cláusula como podría entenderse en las reglas del backlog. Los dos criterios explícitos sí se cumplen.

### HRL-53 Hoja de impresión

Estado: **No verificado**. Sprint 4 · 3 puntos.

Evidencia: `src/utils/printLayout.ts::printLayout`; `src/utils/printCompanyNote.ts::printCompanyNote`; `src/utils/paginatePrintCopies.ts::paginatePrintCopies`; `src/assets/styles/company-note.css`.

Respaldo de criterios: C2: copias se clonan; carta distribuye grupos y pagina contenido. C1 parcialmente: hay anchos 60 y 80 mm con margen, sin opción específica 58 mm.

Criterios no cumplidos o sin verificar: C1: no se ha comprobado que el documento de 60 mm no se corte con impresora/controlador reales; no hay configuración explícita 58 mm.

Discrepancias: Las reglas dicen ticket 80 o 60, pero C1 añade 58. printLayout solo contempla 60/80; la impresión física no queda acreditada por el CSS.

Para verificar: Imprimir notas largas con varias copias en impresoras/controladores 60 y 80 mm; aclarar si se requiere 58 mm y probar esa anchura antes de cerrar C1.

### HRL-54 Notas de entrada y de salida

Estado: **Hecho**. Sprint 4 · 8 puntos.

Evidencia: `src/components/company/CompanyNote.vue::<template>`; `src/composables/useCompanyNote.ts::prepare/money`; `src-tauri/src/services/repairs.rs::RepairService::summary`; `src/utils/printCompanyNote.ts::printCompanyNote`; `src/composables/useWorkshop.ts::print`.

Respaldo de criterios: C1: template consume solo resumen público y no incluye contraseña ni nota interna. C2: money prioriza order.currency. C3: impresión prepara resumen y DOM y abre print; no invoca escritura de pago ni entrega.

Criterios no cumplidos o sin verificar: Ninguno de los tres criterios explícitos en la lectura estática; la impresión física se reserva a HRL-53/63.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

## HRL-7 · Sincronización con el portal web

### HRL-55 Acordar el contrato de sincronización con el portal

Estado: **No verificado**. Sprint 3 · 3 puntos.

Evidencia: `src-tauri/src/services/repairs.rs::RepairService::summary/save`; `src-tauri/src/domain/repair_contracts.rs::Repair`; `búsqueda de archivos y de firebase/outbox en src/, src-tauri/src/, migraciones y tests/`.

Respaldo de criterios: No se encontró un contrato de intercambio implementado ni un artefacto de aprobación en el código. PublicSummary limita datos para impresión local, no define un contrato web.

Criterios no cumplidos o sin verificar: C1: aprobación del responsable del portal no comprobable leyendo código. C2: no se identificó un contrato web que enumere campos prohibidos, autenticación, conflictos y estrategia offline.

Discrepancias: La presencia de sync_mode y preferencias de visibilidad solo acredita almacenamiento local, coherente con el portal «no conectado».

Para verificar: Localizar o acordar el contrato de integración y obtener aprobación explícita del responsable del portal; adjuntar documento/versionado y evidencia de esa aprobación.

### HRL-56 Publicar cambios autorizados

Estado: **Pendiente**. Sprint 3 · 8 puntos.

Evidencia: `src-tauri/src/repositories/repairs.rs::RepairRepository::save (solo seguimiento_publicacion)`; `src-tauri/migrations/001_esquema_objetivo.sql::seguimiento_publicacion`; `src-tauri/src/lib.rs::run`.

Respaldo de criterios: No hay cola local de cambios publicables ni productor/consumidor de mensajes en las rutas activas revisadas.

Criterios no cumplidos o sin verificar: C1: no existe encolado que excluya avances privados. C2: no existe cola durable offline ni proceso de envío diferido.

Discrepancias: Las preferencias y visible_cliente son configuración local, no una publicación autorizada en cola.

### HRL-57 Pruebas del contrato de intercambio

Estado: **Pendiente**. Sprint 3 · 3 puntos.

Evidencia: `tests/ (inventario de archivos)`; `src-tauri/src/services/tests.rs (suite local)`; `src-tauri/src/services/repairs.rs::PublicSummary`.

Respaldo de criterios: Las pruebas de CompanyNote comprueban impresión pública; no se encontró una suite de mensajes de intercambio con portal.

Criterios no cumplidos o sin verificar: C1: falta prueba de contrato web que falle ante contraseña o nota interna.

Discrepancias: Las pruebas de resumen público local no sustituyen pruebas del intercambio web.

### HRL-58 Integrar la publicación con el servicio web

Estado: **Pendiente**. Sprint 4 · 8 puntos.

Evidencia: `src-tauri/src/lib.rs::run`; `src/services/api/workshop.ts::workshopApi`; `src-tauri/src/services/repairs.rs::RepairService::save`; `búsqueda firebase/reqwest/axios/fetch/outbox en el árbol activo`.

Respaldo de criterios: Solo hay IPC local y persistencia SQLite. La única coincidencia fetch de tests/phase5-browser.mjs controla un navegador local de pruebas.

Criterios no cumplidos o sin verificar: C1: falta envío al servicio real y consulta del cliente. C2: falta estrategia de reintento y recuperación de publicación ante caída.

Discrepancias: El estado «no conectado» documentado sí coincide con la ausencia de integración en código.

### HRL-59 Controles de publicación ampliados

Estado: **Pendiente**. Backlog · 5 puntos.

Evidencia: `src-tauri/src/domain/repair_contracts.rs::Visibility`; `src-tauri/src/repositories/repairs.rs::RepairRepository::save`; `src/views/RepairsView.vue::<template>`.

Respaldo de criterios: Hay controles locales de visibilidad, sin estado remoto publicado ni operación de despublicación.

Criterios no cumplidos o sin verificar: C1: despublicar no puede ocultar una orden al cliente porque no existe integración remota.

Discrepancias: Las preferencias locales existentes no equivalen a despublicar ni consultar lo ya publicado.

## HRL-8 · Pruebas, instalación y entrega

### HRL-60 Pruebas de recepción, evidencias y permisos

Estado: **Hecho**. Sprint 2 · 5 puntos.

Evidencia: `src-tauri/src/services/tests.rs::integrated_evidence_revision_restore_and_revoked_authorization_preserve_original/reception_history_requires_logbook_permission_and_list_rechecks_active_users/limited_accounts_require_independent_payment_permission_and_temporary_use_is_audited`; `tests/ipc-stores.test.ts`; `tests/phase4.test.ts`; `tests/phase5.test.ts`.

Respaldo de criterios: C1: pruebas Rust llaman al servicio con cuentas restringidas y verifican Forbidden/Unauthorized y ausencia de cambios. Hay suites de creación/listado/revisión y cámara simulada.

Criterios no cumplidos o sin verificar: Ninguno en cuanto a existencia y contenido de pruebas; no se ejecutaron.

Discrepancias: Las pruebas de cámara usan MediaStream y diálogo simulados. Su existencia no cierra HRL-63.

### HRL-61 Pruebas de reparaciones, pagos y totales

Estado: **Hecho**. Sprint 3 · 5 puntos.

Evidencia: `src-tauri/src/services/tests.rs::restored_ledger_retries_exact_request_and_rolls_back_audit_failure/integrated_notes_warranties_parts_receipts_adjustments_refunds_and_currencies/restored_repair_uses_relational_records_and_rejects_stale_changes`; `src-tauri/src/domain/money.rs::tests::credit_adjustment_refund_and_overflow_use_integer_arithmetic`; `tests/payments-form.test.ts`.

Respaldo de criterios: C1: repite el ID, comprueba un solo abono y rechaza distinto contenido. C2: prueba reduce precio por debajo de cobros y afirma crédito correcto. Incluye notas, garantías, refacciones y devoluciones.

Criterios no cumplidos o sin verificar: Ninguno en cuanto a existencia y contenido de pruebas; no se ejecutaron.

Discrepancias: No se identificaron discrepancias adicionales en el alcance revisado.

### HRL-62 Pruebas de bitácora, finanzas y empresa

Estado: **En progreso**. Sprint 4 · 5 puntos.

Evidencia: `tests/finances-model.test.ts::classification, search, scope and missing dates preserve documented filtering`; `tests/company-notes.test.ts::exit note renders the public summary and warranty from delivery without printing secrets`; `src-tauri/src/services/tests.rs::restored_edit_trash_finances_and_refund_preserve_original_books`; `src-tauri/legacy/finances_tests.rs`.

Respaldo de criterios: C2: renderiza CompanyNote real con contraseña y nota interna de prueba y afirma que no aparecen. C1 parcialmente: pruebas activas afirman completed y filtran pending; hay separación de monedas y periodos.

Criterios no cumplidos o sin verificar: C1: no se encontró prueba de las cuatro condiciones en ReportService normalizado: paid_uncollected y delivered_unpaid solo aparecen cubiertas conjuntamente en pruebas del backend histórico.

Discrepancias: La cobertura histórica de las cuatro combinaciones no comprueba la implementación normalizada activa. No se considera una prueba vigente equivalente por conservarla bajo cfg(test).

### HRL-63 Validación nativa: cámara, impresora y DPAPI

Estado: **No verificado**. Sprint 4 · 3 puntos.

Evidencia: `src/composables/useEvidenceCamera.ts::enable/take/stop`; `src/utils/printCompanyNote.ts::printCompanyNote`; `src-tauri/src/services/key_store.rs::protect`; `tests/phase5.test.ts::camera`; `src-tauri/src/services/tests.rs::windows_key_custody_reopens_dpapi_key_without_storing_it_in_sqlite`.

Respaldo de criterios: Hay implementaciones y pruebas de cámara simulada, impresión DOM y DPAPI condicionada a Windows. No acreditan los dispositivos ni el entorno instalado del taller.

Criterios no cumplidos o sin verificar: C1: falta resultado observado en instalación real para cámara, impresora y DPAPI. No se ejecutó nada durante esta auditoría.

Discrepancias: Los PASS de documentos anteriores son antecedentes fechados; no prueban por sí solos la validación nativa completa solicitada.

Para verificar: Recorrido Tauri instalado en Windows: permisos de cámara concedidos/denegados, captura y liberación; impresión física carta y tickets; custodia DPAPI con perfil original y otro perfil. Registrar versión, entorno, resultados y fallos.

### HRL-64 Instalación y recuperación local

Estado: **No verificado**. Sprint 4 · 5 puntos.

Evidencia: `src-tauri/tauri.conf.json::bundle/identifier`; `src-tauri/src/commands/mod.rs::BackendState::services`; `src-tauri/src/services/backups.rs::BackupService`; `src-tauri/src/services/tests.rs::portable_backups_restore_books_assets_and_device_key_without_the_original_installation`; `docs/instalacion-recuperacion-local.md (procedimiento, no evidencia ejecutada)`.

Respaldo de criterios: C1 parcialmente: hay restauración de paquete y prueba aislada que recupera datos/recursos/clave. C2: el procedimiento enumera base, assets y custodia; solo .db no contiene los archivos. El comportamiento instalado no se comprobó.

Criterios no cumplidos o sin verificar: C1: recuperación en una instalación real con el mismo usuario sigue sin comprobarse. C2 es requisito documental, no demostrable desde el código; el procedimiento existente exige el conjunto completo.

Discrepancias: El backlog presenta solo respaldo manual; BackupService ya implementa paquete portable y automático al cerrar. La copia directa del archivo DPAPI sigue ligada al perfil, mientras el paquete porta la clave cifrada con contraseña.

Para verificar: Instalar el paquete de entrega y restaurar una copia autorizada bajo el mismo perfil; verificar órdenes, fotos y revelado de contraseña. Revisar el aviso explícito de pérdida de recursos al copiar solo .db y registrar el resultado.

### HRL-65 Pruebas piloto con personal del taller

Estado: **No verificado**. Sprint 4 · 3 puntos.

Evidencia: `tests/ui-restored.ts (escenario con datos e IPC simulados)`; `src/WorkshopApp.vue (flujo disponible)`; `no se encontró un acta de piloto comprobable en código`.

Respaldo de criterios: El código ofrece el ciclo local, pero no demuestra participación ni aceptación de personas del taller.

Criterios no cumplidos o sin verificar: C1: falta lista priorizada de hallazgos derivada de un piloto real.

Discrepancias: Una prueba automatizada o escenario de demostración no equivale a un piloto con el personal.

Para verificar: Realizar recepción, reparación, cobro, entrega y emisión de notas con personal autorizado; registrar participantes, versión, incidencias, prioridad y decisión de aceptación.

## HRL-9 · Pendientes futuros

### HRL-66 Importación de datos desde la base anterior

Estado: **Pendiente**. Backlog · 8 puntos.

Evidencia: `src-tauri/src/db/migrations.rs::apply`; `src-tauri/src/lib.rs::run`; `src-tauri/src/services/backups.rs::BackupService::restore`.

Respaldo de criterios: El cargador rechaza explícitamente el esquema anterior; restaurar un paquete normalizado no transforma hrl_datos.db.

Criterios no cumplidos o sin verificar: C1: no hay importador activo que conserve folios y movimientos de la base anterior.

Discrepancias: Coincide con la regla de no migrar datos anteriores sin definir su conservación.

### HRL-67 Video y documentos como evidencia

Estado: **Pendiente**. Backlog · 8 puntos.

Evidencia: `src-tauri/src/domain/evidence.rs::AttachmentKind`; `src-tauri/src/domain/evidence_contracts.rs::validate`; `src-tauri/src/services/evidence.rs::EvidenceService::relative`; `src/composables/useOrderEvidence.ts::checkPhoto`.

Respaldo de criterios: El dominio contempla Video/Document, pero validate/relative/checkPhoto solo admiten JPEG, PNG y WebP.

Criterios no cumplidos o sin verificar: C1: video y documento se rechazan; no hay carga, lectura ni validación de esos formatos bajo el flujo de evidencias.

Discrepancias: Que existan variantes de dominio y soporte previsto en esquema no acredita implementación utilizable; el backlog reconoce esa limitación.

### HRL-68 Respaldo y migración automáticos entre instalaciones

Estado: **En progreso**. Backlog · 8 puntos.

Evidencia: `src-tauri/src/services/backups.rs::BackupService::automatic/capture/restore`; `src-tauri/src/services/backup_archive.rs`; `src/stores/useBackupStore.ts::restore`; `src/composables/useBackupClose.ts`; `src-tauri/src/services/tests.rs::portable_backups_restore_books_assets_and_device_key_without_the_original_installation`.

Respaldo de criterios: C1 parcialmente: hay captura automática al cerrar (si se habilita), paquete completo cifrado y restauración que reprotege clave con DPAPI del destino. Existe prueba de instalaciones temporales con el mismo usuario.

Criterios no cumplidos o sin verificar: C1: no se restaura «sin pasos manuales»: requiere elegir archivo, escribir clave y reiniciar. Falta validación entre perfiles/equipos Windows reales.

Discrepancias: El backlog y el contexto funcional presentan respaldo automático como futuro; ya hay implementación en BackupService y UI. La portabilidad de custodia está implementada al importar, pero no probada aquí entre identidades reales.

Para verificar: Para cerrar la parte nativa, restaurar en otro perfil/equipo autorizado; para cumplir C1 literal debe definirse y respaldarse la restauración sin selección/clave/reinicio manuales.


