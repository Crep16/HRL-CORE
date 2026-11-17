# Estado del backlog HRL Proyecto

Corte: 17 de noviembre de 2026. Revisión estática del árbol de trabajo; no se ejecutaron la aplicación, builds ni pruebas. Hecho significa respaldo en código de los criterios y reglas revisados, no certificación nativa. La existencia de una prueba no acredita que haya pasado. En progreso también incluye criterios respaldados con reglas obligatorias incompletas. Verificado en el CSV significa dictamen obtenido por lectura; No verificado exige ejecución o validación externa.

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

Estado: **En progreso**. Sprint 1 · 8 puntos.

Evidencia: `src-tauri/src/commands/mod.rs::create_order`; `src-tauri/src/services/orders.rs::OrderService::new/create`; `src-tauri/src/domain/money.rs::Money`; `src-tauri/src/services/access.rs::AccessService::require`; `src-tauri/src/db/migrations.rs::apply`.

Respaldo de criterios: C1: los comandos delegan y no contienen SQL. C2: Money encapsula i64 y los DTO usan price_minor. C3: los servicios llaman a require dentro de transacciones.

Criterios no cumplidos o sin verificar: Ningún criterio explícito sin respaldo; la regla «repositorios contienen el SQL» no se cumple de forma exclusiva: db/migrations.rs::apply ejecuta SELECT de reconocimiento del esquema.

Discrepancias: AGENTS.md exige SQL exclusivamente en repositories; la detección del esquema anterior está en db/migrations.rs. Se conserva el estado En progreso por esta regla arquitectónica.

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


