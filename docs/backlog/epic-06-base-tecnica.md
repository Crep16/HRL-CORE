# HRL-57 [Epic] Base técnica transversal

Fuente de alcance vigente: `docs/backlog-base (1).md` (80 ítems, sin HRL-66). Actualización documental del 12 de noviembre de 2026. Los estados de ítems sin cambio de alcance conservan la evidencia de la revisión estática previa; no se ejecutaron pruebas.

Estado: En progreso. Hay base normalizada y rutas nuevas operativas por lectura, pero persisten capas anteriores y reglas fuera de su capa. La validación de ejecución se concentra en HRL-77/HRL-64.
Prioridad: Alta, porque sostiene integridad, seguridad y cierre de la refactorización.
Puntos: 53 (suma de tareas; no sumar el epic de nuevo).
Sprint: 1 y 4.
Subtareas: agrupador de HRL-58 a HRL-63; HRL-64 pertenece ahora al epic HRL-77.

Criterios de aceptación del epic (fuentes: arquitectura objetivo, `docs/sqlite.md`, `docs/reglas-pagos.md`):

- Dada la ruta activa de aplicación, cuando procesa una petición, entonces recorre viewmodel/API/comando/servicio/repositorio/base con las dependencias exigidas.
- Dados datos monetarios y eventos, cuando persisten, entonces usan centavos enteros, referencias activas y libros inmutables con auditoría transaccional.
- Dado fallo de autorización, auditoría o custodia, cuando ocurre, entonces no confirma cambios parciales ni usa la ruta anterior como alternativa de éxito.

Fuentes de aceptación: arquitectura objetivo, `AGENTS.md`, `docs/sqlite.md`, `docs/evidencias.md`, `docs/ventana-por-sesion.md` y reglas funcionales de cada módulo. Las subtareas siguientes son trabajo propuesto; no se modificó código, configuración, pruebas ni migraciones. Los puntos estiman alcance completo.

## HRL-58 [Tarea] Esquema SQLite 3FN y migraciones (001 y 002)

Estado: Hecho para la nueva base (evidencia: `src-tauri/src/db/migrations.rs::migrator` y `apply`, `src-tauri/src/db/connection.rs::Database::connect`, `src-tauri/migrations/001_esquema_objetivo.sql` DDL y `002_prefijo_documentos.sql` ALTER de configuración). La base 001 coincide byte a byte por SHA-256 con `docs/esquema-objetivo.sql` y `schema.sql`. El archivo 002 añade `prefijo_folio`. Archivar y retirar el cargador anterior pertenece a HRL-63.
Prioridad: Alta, porque define integridad, saldos y referencias de todos los agregados.
Puntos: 8
Sprint: 1
Subtareas: ninguna faltante detectada en el nuevo cargador. Pruebas presentes `src-tauri/src/db/tests.rs::every_pool_connection_enforces_foreign_keys_and_file_reopen_preserves_data` y `all_books_and_audit_rows_reject_edits_and_deletion`, no ejecutadas.
Criterios de aceptación (fuentes: arquitectura objetivo, `docs/sqlite.md`, `docs/reglas-pagos.md`):

- Dada una base nueva, cuando se abre con `Database`, entonces aplica solo 001 y 002 de la línea normalizada y activa FK en cada conexión antes de transacciones.
- Dados libros financieros y auditoría, cuando intenta editar o borrar sus entradas, entonces SQLite aborta y conserva el original.
- Dado saldo de una orden, cuando se consulta, entonces deriva de `vista_saldos_orden` con dinero entero en centavos y no de un saldo guardado.
- Dada una base legacy o migración aplicada con checksum alterado, cuando se abre con este cargador, entonces se rechaza sin sustituir datos ni ejecutar una migración de negocio improvisada.

## HRL-59 [Tarea] Arquitectura backend en capas: commands, services, repositories, db

Estado: En progreso (evidencia: `src-tauri/src/commands/mod.rs::create_order`, `src-tauri/src/services/mod.rs::BackendServices::new`, `src-tauri/src/services/repairs.rs::RepairService::load`, `src-tauri/src/services/reports.rs::ReportService::expense`). La ruta nueva separa SQL y comandos, pero algunos servicios construyen repositorios colaboradores directamente (`LedgerRepository`, `OrderRepository`) en vez de recibirlos; `src-tauri/src/lib.rs::run` mantiene además comandos anteriores con SQL fuera de repositories.
Prioridad: Alta, porque la separación y la inyección permiten controlar y probar permisos y transacciones.
Puntos: 8
Sprint: 1
Subtareas:

- HRL-88 [Servicio Rust] Recibir por constructor los repositorios colaboradores que hoy se instancian en RepairService, EvidenceService y ReportService, conservando la transacción única y la composición.
- HRL-89 [Pruebas] Probar las colaboraciones mediante contratos de repositorio sustituibles donde permitan simular fallos, sin convertir los comandos en acceso directo a SQL. El retiro de rutas anteriores se concentra en HRL-63.

Criterios de aceptación (fuente: arquitectura objetivo §§3, 4 y restauración de apartados):

- Dada una petición de UI, cuando llega a un comando, entonces valida forma y delega en un servicio sin SQL ni acceso directo a repositorios.
- Dado un servicio, cuando necesita persistencia, entonces recibe sus dependencias por constructor y solo los repositorios realizan SQL de negocio.
- Dado un fallo de repositorio o auditoría, cuando guarda, entonces revierte la misma transacción sin confirmar una operación parcial.
- Dadas las capas, cuando se inspeccionan dependencias, entonces no existen referencias de repositorio a servicio ni I/O en domain.

## HRL-60 [Tarea] Frontend MVVM con stores Pinia y capa services/api

Estado: En progreso (evidencia: `src/main.ts` registra Pinia, `src/stores/useAuthStore.ts`, `src/stores/useOrderStore.ts` y `src/services/api/transport.ts::request`; contraejemplos: `src/composables/useAccess.ts::login` y `useOrders.ts::saveOrder` invocan Tauri directamente; `src/utils/finances.ts::financialTotals` y `filterFinancialOrders` contienen reglas de negocio). La ruta normalizada ya usa stores/API; el cumplimiento todavía no cubre todo el árbol.
Prioridad: Alta, porque evita autoridades de permisos y dinero dispersas en frontend.
Puntos: 8
Sprint: 1
Subtareas:

- HRL-90 [Composable/store] Cerrar la coexistencia de acceso a Tauri directo en composables con HRL-63 y mantener `services/api` como único adaptador IPC de aplicación.
- HRL-91 [Servicio Rust] Trasladar las reglas financieras de exclusión, periodos y agregación de `utils/finances.ts` a servicio/modelo de dominio según arquitectura, dejando formato y fechas puras en utils.
- HRL-92 [Pruebas] Verificar equivalencia de los filtros e indicadores, denegaciones en backend y descarte de respuestas de una sesión anterior tras completar la separación.

Criterios de aceptación (fuente: arquitectura objetivo y `docs/bitacora.md`):

- Dada una vista, cuando requiere datos o cambios, entonces usa store/composable y la única llamada a `invoke` pertenece a `services/api`.
- Dadas reglas de dinero o permiso, cuando se confirma una operación, entonces la decisión autoritativa reside en Rust y conserva las reglas documentadas.
- Dada sesión nueva o respuesta tardía, cuando se resuelve una solicitud anterior, entonces no incorpora datos ni permisos a la sesión actual.
- Dado fallo de IPC, cuando una escritura tiene resultado incierto, entonces no simula éxito y conserva la solicitud financiera para reintento seguro.

## HRL-61 [Tarea] Mover vistas restantes de components/ a views/ y adelgazar WorkshopApp

Estado: En progreso (evidencia: `src/WorkshopApp.vue` es contenedor con `useWorkshop`; `src/views/ReceptionView.vue` existe; Empresa, Usuarios, Reparaciones, Bitácora y Acceso siguen como `*Workspace.vue` bajo components. `src/components/orders/EvidenceCamera.vue::enable` y `take`, `src/components/company/CompanyClauses.vue::apply` y `move` aún conservan lógica de pantalla dentro de componentes).
Prioridad: Media, porque completa MVVM sin cambiar reglas funcionales.
Puntos: 8
Sprint: 4
Subtareas:

- HRL-93 [Vista] Trasladar las pantallas completas restantes a views y actualizar la composición de WorkshopApp conservándolo como contenedor.
- HRL-94 [Composable/store] Extraer estado y operaciones de cámara y editor de cláusulas; llevar al viewmodel la selección de días sugeridos que hoy calcula la plantilla de RepairWorkspace.
- HRL-95 [Pruebas] Verificar guardas, eventos, foco y errores de las pantallas trasladadas sin cambiar permisos ni comportamiento del taller.

Criterios de aceptación (fuente: arquitectura objetivo, `docs/acceso-validacion.md`, `docs/evidencias.md`):

- Dadas las pantallas principales, cuando se inspecciona su composición, entonces residen en views y WorkshopApp solo coordina navegación/contenedores mediante su viewmodel.
- Dada una cámara o formulario, cuando el usuario actúa, entonces la vista emite/presenta y el composable/store gestiona estado, validación y persistencia.
- Dado un error de guardado o un borrador pendiente, cuando intenta salir de la pantalla trasladada, entonces conserva las guardas y datos del flujo documentado.

## HRL-62 [Tarea] Custodia de claves con DPAPI y estrategia de respaldo (base + assets + claves)

Estado: En progreso (evidencia: `src-tauri/src/services/key_store.rs::OsKeyStore::open` y `protect`, `src-tauri/src/commands/mod.rs::BackendState::services`, `src-tauri/src/db/connection.rs::Database::open_file`). DPAPI protege una clave externa en `keys/device-password-v1.dpapi`; base normalizada y `assets-3fn` están definidos. No se localizó una estrategia completa de recuperación de ese conjunto entre instalaciones; la protección depende de la identidad Windows. El funcionamiento real de DPAPI es No verificado en esta revisión.
Prioridad: Alta, porque perder la clave puede volver irrecuperables las contraseñas cifradas.
Puntos: 8
Sprint: 1
Subtareas:

- HRL-96 [Servicio Rust] Precisar el contrato de custodia/restauración conjunta de DB, assets y clave protegida, distinguiendo recuperación con el mismo usuario Windows de portabilidad. No elegir exportación o cambio de protección sin definición de HRL-70.
- HRL-97 [Pruebas] Verificar reapertura DPAPI con la identidad real de destino y fallos por archivo ausente/corrupto o identidad no válida; comprobar que no aparece material de clave en SQLite ni auditoría.

Criterios de aceptación (fuentes: arquitectura objetivo §7, `docs/evidencias.md`, alcance HRL-62 del backlog):

- Dada una credencial de equipo, cuando persiste, entonces su clave AES queda fuera de SQLite y protegida por DPAPI en Windows.
- Dada la misma custodia válida, cuando reabre la instalación, entonces puede recuperar las credenciales cifradas sin regenerar la clave de registros existentes.
- Dado respaldo, cuando se define y valida su recuperación, entonces contempla DB, evidencias/logo y claves; copiar solo DB no se presenta como respaldo completo.
- Dada custodia no recuperable, cuando se intenta abrir o descifrar, entonces informa error sin exponer secretos ni afirmar recuperación exitosa.

Discrepancia D-12: `docs/evidencias.md` define respaldo de base y carpeta de fotos del sistema anterior; el código normalizado necesita también `assets-3fn/company`, `assets-3fn/evidencias` y la clave DPAPI externa. La portabilidad entre usuarios/equipos no se puede inferir de `OsKeyStore`.

## HRL-63 [Tarea] Retirar código legacy (comandos anteriores, plugin SQL 001-013, archivos Legacy)

Estado: En progreso (evidencia: `src-tauri/src/lib.rs::run` registra plugin SQL y comandos anteriores; `src-tauri/migrations/001_access.sql` a `013_spanish_schema_names.sql` siguen fuera de legacy; `src/components/access/LegacyAccessWorkspace.vue`, `src/components/orders/LegacyOrderForm.vue`, `src/composables/useOrders.ts` y `useAccess.ts` siguen presentes). La UI de arranque usa la ruta normalizada y no precarga la base anterior, pero no se ha retirado todo el código anterior.
Prioridad: Alta, porque mantiene rutas alternativas de autorización y persistencia fuera de la arquitectura objetivo.
Puntos: 13
Sprint: 4
División propuesta: tres entregas de 5, 5 y 3 puntos representadas por HRL-98, HRL-99 y HRL-100; HRL-101 verifica las tres y no añade puntos al padre.
Subtareas:

- HRL-98 [Repositorio/SQL] Archivar migraciones anteriores en legacy y retirar su registro del cargador anterior sin ejecutar ni migrar datos reales (5 puntos de los 13 del padre).
- HRL-99 [Servicio Rust] Retirar comandos/módulos anteriores de lib.rs y el plugin ligado a esa ruta, conservando solo servicios normalizados y el mecanismo de seguridad vigente (5 puntos).
- HRL-100 [Composable/store] Retirar composables/componentes Legacy y datos de demostración sin uso como fuente de verdad, tras comprobar referencias y conservar utilidades puras aún necesarias (3 puntos).
- HRL-101 [Pruebas] Comprobar ausencia de invocaciones antiguas y regresión de permisos, recepción, reparación y dinero sobre la única ruta vigente (incluida en las entregas anteriores).

Criterios de aceptación (fuentes: arquitectura objetivo, `AGENTS.md` §5, `docs/sqlite.md`):

- Dada una instalación nueva, cuando arranca, entonces no registra ni ejecuta migraciones 001-013 del esquema anterior y usa la línea normalizada.
- Dadas referencias al código anterior, cuando se completa el retiro, entonces ninguna pantalla o comando depende de una sesión o base alternativa.
- Dada una base previa existente, cuando se retira compatibilidad, entonces no se importan ni destruyen sus datos sin confirmación de migración.
- Dado un fallo en un flujo normalizado, cuando ocurre, entonces no recurre a datos de demostración ni a la ruta anterior para aparentar éxito.

Discrepancia D-13: `AGENTS.md` exige archivar 001-013; arquitectura §7 documenta una conservación transitoria autorizada. El código conserva registro y archivos anteriores después de añadir las rutas nuevas. Se reporta deuda; no se ejecuta una retirada en esta tarea documental.

Hallazgo adicional D-14: `src/data/orders.ts::initialOrders` contiene órdenes, contactos, importes y una contraseña de demostración. No es un catálogo estático de presentación; la búsqueda de referencias en src solo localiza la definición de `initialOrders`. No se usó como evidencia de órdenes reales ni como fuente de verdad.
