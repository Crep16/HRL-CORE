# HRL-65 [Epic] Pendientes futuros (Backlog)

Estado: En progreso como agrupador: Herramientas ya tiene implementación, mientras las otras capacidades están pendientes o carecen de verificación suficiente.
Prioridad: Media para el conjunto; Alta para importación, respaldo y custodia por su impacto en datos y secretos.
Puntos: 45 (suma de historias; estimación preliminar, sin sumar otra vez el epic).
Sprint: Backlog, sin fecha.
Subtareas: conserva HRL-66 a HRL-70; nuevas subtareas HRL-94 a HRL-106.

Fuentes: `docs/contexto-completo-proyecto.md` §§6.4 y 9; `docs/contexto-funcional-vistas-y-estructura.md` §§1 y pendientes; `docs/evidencias.md`; `AGENTS.md` §6. No hay especificación suficiente para inventar autenticación del portal, formatos y tamaños de video/documento, resolución de conflictos de importación o transporte de claves entre equipos. Los criterios siguientes recogen límites ya documentados y condiciones de entrada; no sustituyen la especificación faltante. Los puntos son provisionales de alcance completo.

## HRL-66 [Historia] Portal web del cliente y publicación de avances

Estado: Pendiente (evidencia: `src-tauri/src/services/repairs.rs::RepairService::save` y `src-tauri/src/repositories/repairs.rs::RepairRepository::save` solo guardan preferencias locales; `src/services/api/workshop.ts` no contiene publicación remota; `src-tauri/src/commands/mod.rs` y `src-tauri/src/lib.rs::run` no registran un comando de publicación). La búsqueda en src y src-tauri/src no localizó implementación del portal. Una infraestructura externa no incluida en el repositorio está No verificada.
Prioridad: Media, porque amplía seguimiento al cliente sin bloquear la operación local existente.
Puntos: 13
Sprint: Backlog
División propuesta: alcance/presentación de información pública (5), servicio de publicación (5), validación de privacidad y fallos (3). Reestimar al tener contrato externo; no se selecciona proveedor ni autenticación.
Subtareas:

- HRL-94 [Vista] Definir con producto el contrato de consulta del cliente y construir la vista del portal después de aprobarlo, usando solo datos públicos previstos (5 puntos).
- HRL-95 [Servicio Rust] Implementar publicación real según contrato y autorización definidos, separada del guardado local y con trazabilidad de la operación efectiva (5 puntos).
- HRL-96 [Pruebas] Verificar distinción entre guardado local/publicación, aislamiento de notas internas y manejo del fallo de conexión conforme al contrato que se apruebe (3 puntos).

Criterios de aceptación documentados (fuentes: contexto funcional §§1 y 2, `docs/bitacora.md`, `docs/reparaciones-permisos.md`):

- Dadas preferencias de publicación guardadas localmente, cuando se guardan sin conexión al portal, entonces no se presentan como publicación realizada.
- Dadas notas internas y contraseña del equipo, cuando se prepara información para el cliente, entonces esos datos no forman parte del resumen público.
- Dado un operador sin permiso de modificar publicación, cuando intenta cambiar las preferencias o iniciar la operación protegida correspondiente, entonces se deniega sin alterar permisos permanentes.

No verificado: contrato de acceso del cliente, destino, transporte, reintentos y evidencia de despliegue externo. Se necesita especificación funcional/técnica y acceso de lectura al proyecto del portal, si existe.

## HRL-67 [Historia] Importación de datos desde la base legacy

Estado: No verificado para una importación a la base normalizada (evidencia: `src-tauri/src/orders.rs::import_orders` usa `access::pool` y sesión anteriores; `src/composables/useOrders.ts::importLegacy` importa desde localStorage al comando anterior; `src-tauri/src/db/migrations.rs::apply` rechaza una base con `Ordenes.datos_json`). Esos importadores no acreditan migración a 3FN. No se confirmó existencia, contenido ni necesidad de migrar datos reales.
Prioridad: Alta, porque una importación incorrecta puede perder expedientes y duplicar dinero/auditoría.
Puntos: 8
Sprint: Backlog
Subtareas:

- HRL-97 [Repositorio/SQL] Inventariar una copia autorizada de la base origen y definir correspondencias a 3FN, conservando libros, identificadores de referencia y fechas desconocidas; no escribir migración de datos sin confirmar el alcance.
- HRL-98 [Servicio Rust] Diseñar la importación transaccional al destino normalizado después de confirmar datos y reglas de conflictos, sin reutilizar la sesión ni el pool del importador anterior.
- HRL-99 [Pruebas] Con fixtures autorizadas, comprobar conservación de originales, dinero y auditoría, rechazo de datos ambiguos y ausencia de duplicación al repetir la importación según el contrato aprobado.

Criterios de aceptación documentados (fuentes: `AGENTS.md` §6, `docs/sqlite.md`, `docs/reparaciones-permisos.md`, `docs/bitacora.md`):

- Dada ausencia de confirmación de datos existentes, cuando se prepara esta historia, entonces no se escribe ni ejecuta una migración de negocio sobre datos supuestos.
- Dado origen confirmado, cuando se acuerda e implementa su importación, entonces conserva los originales y no inventa fechas o autorías históricas inexistentes.
- Dado un origen ilegible o ambiguo, cuando se intenta importar, entonces bloquea escritura y conserva los datos para revisión en vez de eliminarlos o renombrarlos automáticamente.

Falta para verificar: copia de origen autorizada, esquema/versiones efectivos, inventario de assets/claves, reglas de correspondencia y conflictos, y evidencia del resultado en destino. No se solicitó ni se alteró una base real en esta revisión.

## HRL-68 [Historia] Carga de video y documentos como evidencia

Estado: Pendiente como funcionalidad (evidencia: `src-tauri/src/domain/evidence_contracts.rs::validate` solo acepta JPEG/PNG/WebP; `src-tauri/src/services/evidence.rs::EvidenceService::relative` solo produce jpg/png/webp; `src/composables/useOrderEvidence.ts::checkPhoto` también rechaza otros MIME). La tabla evidencias contempla tipos adicionales y `src-tauri/src/db/tests.rs::evidence_paths_originals_and_encrypted_device_credentials_have_structural_constraints` contiene inserciones de prueba; no existe flujo de carga de esos tipos en el servicio/UI actual.
Prioridad: Baja, porque amplía formatos y no bloquea la evidencia fotográfica actual.
Puntos: 8
Sprint: Backlog
Subtareas:

- HRL-100 [Vista] Diseñar selección y consulta de archivos adicionales después de definir formatos, tamaños y comportamiento de visualización; no reutilizar silenciosamente el límite fotográfico de 20 MiB.
- HRL-101 [Composable/store] Extender contratos de evidencia y estados de reintento manteniendo identificadores, alcance y errores de sesión.
- HRL-102 [Servicio Rust] Ampliar validación y almacenamiento controlado por tipo junto al repositorio normalizado, una vez aprobada la política de archivo.
- HRL-103 [Pruebas] Cubrir tipos autorizados/no autorizados, tamaños definidos, rollback de metadatos/auditoría y conservación del original retirado.

Criterios de aceptación documentados (fuentes: `docs/evidencias.md`, contexto completo §9):

- Dado el alcance fotográfico actual, cuando se envía video o documento, entonces se rechaza y no se anuncia soporte solo porque la tabla admita el tipo.
- Dada una evidencia persistida, cuando se retira, entonces conserva archivo original, identidad e historial y permite restauración autorizada.
- Dado permiso insuficiente o una orden en papelera, cuando intenta agregar o modificar evidencia, entonces Rust rechaza.

No verificado: formatos, límites, reproducción/descarga y política de seguridad de los tipos nuevos. Falta una especificación aprobada antes de convertir esos aspectos en criterios de producto.

## HRL-69 [Historia] Herramientas (acceso temporal, consulta por referencia, actualización e historial)

Estado: Hecho por inspección estática (evidencia: `src/components/layout/AppHeader.vue` menú Herramientas, `src/composables/useWorkshop.ts::requestAccess` y `receptionTool`, `src/composables/useReceptionForm.ts::lookup`, `refreshList` y `openHistory`). Las cuatro acciones están conectadas, con habilitación contextual de las de Clientes y permiso independiente de historial.
Prioridad: Media, porque reúne accesos rápidos a funciones existentes.
Puntos: 3
Sprint: Backlog (clasificación original conservada, aunque ya tiene implementación).
Subtareas: ninguna faltante detectada en el menú y sus conexiones; HRL-9, HRL-14, HRL-16 y HRL-43 cubren sus funciones y HRL-64 su recorrido nativo. No duplicar esas implementaciones como trabajo futuro.
Criterios de aceptación (fuentes: `docs/contexto-completo-proyecto.md` §§3 y 4, `docs/acceso-temporal.md`, `docs/bitacora.md`):

- Dado el apartado Clientes, cuando abre Herramientas, entonces puede consultar por folio/ID, actualizar listado y abrir historial según permisos y selección.
- Dada una operación restringida, cuando solicita acceso temporal, entonces pide autorización de otra persona y motivo sin cambiar la identidad del operador.
- Dado un usuario sin permiso de Bitácora o sin orden seleccionada, cuando intenta abrir historial, entonces no obtiene el historial protegido.
- Dado un borrador modificado, cuando consulta otra referencia, entonces exige resolver los cambios antes de reemplazar la ficha.

Discrepancia D-16: HRL-69 permanece en el epic futuro y el contexto funcional declara Herramientas pendiente, pero el menú ya está conectado. Solapa con HRL-9/14/16/43: conviene cerrar el agrupador sin volver a estimar o implementar sus funciones. Se conservan clave y 3 puntos de la estimación para que la tabla base sea reproducible.

## HRL-70 [Historia] Respaldo y migración automática entre instalaciones

Estado: Pendiente (evidencia: `src-tauri/src/schema_backup.rs::before_spanish_migration` solo crea un respaldo preventivo de `hrl_datos.db` previo a la 013; `src-tauri/src/db/connection.rs::Database::open_file` abre DB/assets locales; `src-tauri/src/services/key_store.rs::OsKeyStore::open` usa DPAPI del usuario). No se localizó un comando, servicio o vista de respaldo/restauración integral normalizada ni transferencia de claves entre instalaciones.
Prioridad: Alta, porque un respaldo incompleto pierde fotos o acceso a credenciales cifradas.
Puntos: 13
Sprint: Backlog
División propuesta: contrato y custodia de recuperación (5), paquete consistente DB/assets y restauración (5), validación entre instalaciones (3). No se elige formato, cifrado de exportación, frecuencia ni política de retención sin definición del producto.
Subtareas:

- HRL-104 [Servicio Rust] Definir e implementar, tras aprobar la estrategia de HRL-62, el procedimiento autorizado para recuperar claves junto con la instalación, distinguiéndolo del restablecimiento Ed25519 de contraseñas de acceso (5 puntos).
- HRL-105 [Repositorio/SQL] Implementar captura y restauración consistentes de DB normalizada y assets, conservando referencias, libros y secuencias; el respaldo legacy existente no sustituye esta entrega (5 puntos).
- HRL-106 [Pruebas] Probar restauración conjunta en origen/destino y rechazos por paquete incompleto, clave no recuperable o esquema incompatible, sin alterar el original hasta validar el resultado (3 puntos).

Criterios de aceptación documentados (fuentes: contexto completo §6.4, `docs/evidencias.md`, `AGENTS.md` §6):

- Dada una instalación con fotos, logo y secretos cifrados, cuando se define y valida su respaldo operativo, entonces incluye DB, recursos y custodia de claves; solo el `.db` no se declara suficiente.
- Dado cambio de usuario Windows o instalación, cuando se evalúa restauración, entonces no se supone que DPAPI permite descifrar por copiar el archivo protegido.
- Dado restablecimiento de acceso con Ed25519, cuando se recuperan credenciales de usuario, entonces no se presenta ese procedimiento como migración de la clave AES.
- Dada ausencia de datos de producción confirmados, cuando se propone migración entre instalaciones, entonces no se escribe ni ejecuta importación sobre datos supuestos.

No verificado: portabilidad de claves, esquema de paquete, autorización, frecuencia, retención y compatibilidad entre versiones. Se necesita decisión funcional/técnica y dos instalaciones de prueba con identidades Windows conocidas. No se inventan estas políticas.
