# Resumen de revisión del backlog

Corte: 17 de noviembre de 2026. Solo lectura de código; sin ejecución de pruebas, builds ni validación nativa. Revisión de 38/59 historias y tareas; los ítems aún sin revisar en commits intermedios no suman como Hecho. Epics excluidas de puntos y porcentajes; sus columnas nuevas quedan vacías para no atribuirles un estado no solicitado. Los cuatro ítems sin sprint forman Backlog.

| Sprint | Fechas 2026 | Ítems | Puntos totales | Puntos Hecho | Porcentaje |
| --- | --- | ---: | ---: | ---: | ---: |
| Sprint 1 | 26–30 oct | 13 | 70 | 36 | 51.4% |
| Sprint 2 | 2–6 nov | 12 | 70 | 57 | 81.4% |
| Sprint 3 | 9–13 nov | 17 | 76 | 52 | 68.4% |
| Sprint 4 | 16–20 nov | 13 | 69 | 29 | 42.0% |
| Backlog | sin fecha | 4 | 29 | 0 | 0.0% |

## Discrepancias entre documentación y código

- HRL-10: El backlog y la sección de cierre del cargador en arquitectura-objetivo.md describen 001–002; el cargador activo incluye 003. Evidencia: src-tauri/src/db/migrations.rs::migrator/apply.
- HRL-11: AGENTS.md exige SQL exclusivamente en repositories; la detección del esquema anterior está en db/migrations.rs. Se conserva el estado En progreso por esta regla arquitectónica. Evidencia: src-tauri/src/commands/mod.rs::create_order.
- HRL-12: El contexto funcional antiguo describe llamadas IPC dispersas y ausencia de Pinia; src/services/api y src/stores ya implementan la separación. Evidencia: src/services/api/transport.ts::request.
- HRL-13: El contrato contiene más códigos que las cinco categorías resumidas en el backlog; no implica pérdida de esas categorías. Evidencia: src-tauri/src/error.rs::AppError::code/serialize.
- HRL-14: El respaldo portable ahora tiene código propio; copiar solamente SQLite sigue sin incluir archivos y custodia. Evidencia: src-tauri/src/services/key_store.rs::OsKeyStore::open_key/protect.
- HRL-15: El backlog presenta el retiro como pendiente; el árbol conserva el código histórico archivado, fuera del flujo productivo. El adaptador utils/legacyOrder.ts transforma DTO, no usa la base anterior. Evidencia: src-tauri/src/lib.rs::run.
- HRL-20: La regla transversal del backlog exige revisión en toda edición; SaveUserPayload y UserService::save no incluyen expected_revision. Los criterios específicos de HRL-20 sí tienen respaldo; queda una brecha transversal que debe planificarse. Evidencia: src-tauri/src/services/users.rs::UserService::save.
- HRL-21: El documento acceso-temporal.md describe tablas históricas y triggers 011; el flujo vigente usa accesos_temporales/acceso_temporal_permiso y AuditService. Evidencia: src-tauri/src/services/access.rs::AccessService::temporary/require/finish_temporary/logout.
- HRL-24: La regla de ficha pide golpes y líquidos independientes en pantalla; OrderForm usa Antecedentes del equipo unificado mediante historyNotes. Backend conserva los campos anteriores y añade history.notes con la migración 003. Evidencia: src/components/orders/OrderForm.vue::<template>.
- HRL-27: AGENTS.md aún presenta cifrar/hashear/retirar como pendiente; arquitectura-objetivo.md confirma cifrado y CryptoService ya lo implementa. No existe un permiso exclusivo de revelado: se utiliza el permiso de consulta de la orden. Evidencia: src-tauri/src/services/crypto.rs::CryptoService::encrypt/decrypt.
- HRL-28: El backlog exige solicitar guardar o descartar al navegar; useWorkshop::prepareLeave y createReceptionForm::prepareLeave solo devuelven false con aviso. Evidencia: src/composables/useReceptionForm.ts::prepareLeave/cancel.
- HRL-29: evidencias.md describe Evidencias_orden y migración histórica 012/013; el flujo actual usa evidencias y archivos bajo assets-3fn/evidencias. Evidencia: src-tauri/src/domain/evidence_contracts.rs::validate.
- HRL-30: El criterio C2 contradice la regla de la misma historia que permite consultar con Visualizar Clientes. Código y evidencias.md distinguen activas de retiradas. Se marca En progreso con la redacción literal, sin resolver la ambigüedad ni cambiar la regla. Evidencia: src-tauri/src/services/evidence.rs::EvidenceService::list/read/update.
- HRL-31: La UI oculta Papelera sin orders.delete, pero OrderService::list con trashed=true solo exige clients.view; la restricción de consultar papelera no está comprobada con el mismo permiso en backend. Evidencia: src-tauri/src/services/orders.rs::OrderService::trash/list.
- HRL-33: reparaciones-permisos.md describe migración 006 y importación localStorage anteriores; el servicio activo usa tablas normalizadas y no importa aquel seguimiento. Evidencia: src-tauri/src/services/repairs.rs::RepairService::summary/save.
- HRL-34: Repair::default inicia solo Recibido y Entregado, no la secuencia sugerida completa. El catálogo de opciones sí está disponible y se incorporan etapas al agregar avances. El DTO Event no devuelve id_autor, aunque SQLite lo conserva. Evidencia: src/composables/useRepairScreen.ts::complete/addEvent.
- HRL-35: La regla del backlog pide pieza, cantidad y costo; Part y la captura solo ofrecen nombre y precio. El INSERT no recibe cantidad (se usa el valor predeterminado del esquema). No se evalúan existencias. Evidencia: src/composables/useRepairScreen.ts::addPart/selectWarrantyType/addWarranty.
- HRL-37: La regla admite entrega histórica sin fecha; update_order completa fecha_entrega con fecha UTC actual cuando delivered=true y fecha era NULL, incluso al editar otro dato. No permite conservar esa fecha desconocida durante dicho guardado. Evidencia: src-tauri/src/services/orders.rs::OrderService::edit.
- HRL-42: La persistencia de reintento en localStorage corresponde a movimientos del panel. El abono inicial conserva el payload solo en memoria de createReceptionForm; al cerrar se pierde y la UI pide mantener abierta la ventana. El backlog HRL-13/42 no distingue esta limitación. Evidencia: src-tauri/src/services/ledger.rs::LedgerService::record.
- HRL-43: AuditService no filtra por sí mismo campos secretos: cada servicio construye su lista de cambios. Las escrituras revisadas omiten claves y hashes; no se presume un filtro global que no existe. Evidencia: src-tauri/src/services/audit.rs::AuditService::record.
- HRL-44: bitacora.md describe tablas históricas separadas; el código consulta auditoria/auditoria_detalle. El nombre mostrado procede de usuarios.nombre mediante JOIN: renombrar una cuenta cambia el nombre mostrado en eventos antiguos, no existe snapshot histórico. Evidencia: src-tauri/src/services/reports.rs::ReportService::logbook.

Las rutas originales reales están en docs/, no en docs/backlog/. El contexto solicitado se sustituyó por el documento funcional existente. Los totales declarados por el original se contrastan con las filas del CSV, sin corregir los originales.

## No verificado

- HRL-14: Ejecutar instalación Windows con dos perfiles; guardar una credencial y comprobar recuperación en el original y rechazo en el segundo, incluida ausencia/corrupción del archivo DPAPI.
- HRL-18: Abrir configuración, acceso y taller en Windows; maximizar taller, cerrar sesión y medir dimensiones, límites y capacidad de redimensionar.

## Sprint 4 (16–20 noviembre): trabajo que no está Hecho

- HRL-52 Cláusulas y textos de las notas (5 pts): Aún sin revisar en este commit intermedio.
- HRL-53 Hoja de impresión (3 pts): Aún sin revisar en este commit intermedio.
- HRL-54 Notas de entrada y de salida (8 pts): Aún sin revisar en este commit intermedio.
- HRL-58 Integrar la publicación con el servicio web (8 pts): Aún sin revisar en este commit intermedio.
- HRL-62 Pruebas de bitácora, finanzas y empresa (5 pts): Aún sin revisar en este commit intermedio.
- HRL-63 Validación nativa: cámara, impresora y DPAPI (3 pts): Aún sin revisar en este commit intermedio.
- HRL-64 Instalación y recuperación local (5 pts): Aún sin revisar en este commit intermedio.
- HRL-65 Pruebas piloto con personal del taller (3 pts): Aún sin revisar en este commit intermedio.

## Alcance y trazabilidad

Solo se generan estos tres archivos dentro de docs/backlog/. Se conservan las 68 filas originales, las claves de referencia HRL, los títulos, los padres, sprints, prioridades, puntos y todas las columnas originales. La verificación del CSV compara valores de celdas; las comillas y los saltos físicos pueden cambiar. No se elimina ni ejecuta código histórico. No se incluyen existencias de refacciones.

El usuario autorizó un proyecto nuevo HRLP, porque HRL ya pertenece a HRL CORE; las referencias HRL se conservarán en las descripciones de Jira. Estado de creación/sincronización: pendiente de completar mediante las herramientas disponibles.

Control de entrega documental: PASS, los dictámenes incluyen referencias a código y distinguen comprobaciones externas. PASS, los puntos se calculan desde el CSV original. PASS, no se atribuyen resultados a builds o pruebas no ejecutados. Las reglas visuales de antislop no aplican a esta auditoría sin cambios de UI.

