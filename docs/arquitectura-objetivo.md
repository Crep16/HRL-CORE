# Arquitectura objetivo de HRL CORE

## 1. Conceptos (para exponerlos ante el profesor)

| Concepto | Qué es | Dónde se aplica aquí |
|---|---|---|
| **POO** | Paradigma: modelar con objetos que encapsulan datos y comportamiento | Structs + `impl` + `trait` en Rust; clases de dominio en TypeScript |
| **Arquitectura en capas** | Patrón: separar responsabilidades en niveles que solo dependen hacia abajo | Backend Rust: commands → services → repositories → db |
| **MVC** | Patrón: Modelo-Vista-Controlador | Equivalente del backend: `commands` = controlador, `services`+`domain` = modelo |
| **MVVM** | Patrón: Modelo-Vista-VistaModelo, con enlace de datos reactivo | Frontend Vue: componentes = Vista, Pinia/composables = VistaModelo, tipos/servicios API = Modelo |

POO es un **paradigma**; capas, MVC y MVVM son **patrones de arquitectura**. Se combinan: la arquitectura define las
capas y la POO define cómo se construyen las piezas dentro de cada capa.

## 2. Vista general

```
┌──────────────────────── Frontend (Vue 3) · MVVM ────────────────────────┐
│  View (.vue)  ⇄  ViewModel (Pinia store / composable)  →  Model (tipos) │
│                                   │                                      │
│                        services/api  (único invoke de Tauri)             │
└───────────────────────────────────┼──────────────────────────────────────┘
                                    │ comandos Tauri
┌───────────────────────────────────▼──────────────────────────────────────┐
│ Backend (Rust) · Capas                                                    │
│  commands  →  services  →  repositories  →  db (SQLite)                   │
│                  │                                                        │
│               domain (structs, enums, reglas puras)                       │
└──────────────────────────────────────────────────────────────────────────┘
```

## 3. Responsabilidad de cada capa

**commands** — Traducen la petición a una llamada de servicio. Validan la forma (tipos, campos obligatorios),
nunca reglas de negocio.

**services** — Corazón del sistema. Ejemplos:
- `PaymentService`: valida permisos, no exceder el pendiente, devolución completa del saldo a favor,
  idempotencia por `id_solicitud`/`huella_solicitud`, escribe auditoría.
- `OrderService`: folio sin colisión (`secuencias_folio`), papelera (`eliminada_en`), historial de cambios.
- `RepairService`: guardado con control de revisión (rechaza guardados desactualizados).
- `AccessService`: autenticación, clave maestra, accesos temporales, último administrador.
- `AuditService`: único punto que escribe `auditoria` y `auditoria_detalle`.

Cada servicio abre **una transacción**, ejecuta el cambio y la auditoría, y confirma o revierte todo junto.

**repositories** — SQL puro y mapeo a structs. Sin reglas de negocio. Uno por agregado.

**domain** — Tipos como `Money` (centavos, no `f64`), `OrderStatus`, `PermissionKey { apartado, accion }`,
`Balance { precio, cobrado, ajustado, devuelto }` con métodos `net()`, `pending()`, `credit()`.

## 4. Estructura de carpetas propuesta

```
src-tauri/src/
  commands/   orders.rs payments.rs repairs.rs evidence.rs company.rs users.rs access.rs audit.rs
  services/   order_service.rs payment_service.rs repair_service.rs ... audit_service.rs
  repositories/ order_repo.rs payment_repo.rs ...
  domain/     money.rs balance.rs permissions.rs ...
  db/         connection.rs migrations.rs
  error.rs  main.rs
src-tauri/migrations/
  legacy/               migraciones 001–013 del esquema anterior (archivadas, no se ejecutan)
  001_esquema_objetivo.sql   nueva línea base

src/                    (estado actual → destino)
  assets/               se queda
  components/           se queda, pero solo presentación
  composables/          se queda; aquí vive la lógica de pantalla reutilizable (useOrderForm, useDraftGuard...)
  types/                se queda; es el MODEL (Order, Money, Payment...)
  utils/                se queda, solo funciones puras
  data/                 se queda, solo catálogos estáticos (por confirmar su contenido)
  views/        (nuevo) ClientsView.vue RepairsView.vue FinancesView.vue CompanyView.vue UsersView.vue AccessView.vue
  stores/       (nuevo) useOrdersStore.ts usePaymentsStore.ts useSessionStore.ts ...
  services/api/ (nuevo) ordersApi.ts paymentsApi.ts ...  ← único lugar con invoke()
  WorkshopApp.vue       se divide en views/ y queda como contenedor/navegación
  App.vue, main.ts      se quedan (main.ts registra Pinia)
```

Nota: el proyecto ya tiene `components`, `composables` y `types`, así que el frontend no se rehace; se
**completa** con las capas que faltan (`stores`, `services/api`, `views`) y se adelgaza `WorkshopApp.vue`.

## 5. Del esquema anterior al nuevo (qué se mueve a dónde)

| Antes | Ahora |
|---|---|
| `Ordenes.datos_json` | Columnas de `ordenes` + `clientes` + `equipos` + `abonos` |
| `Seguimiento_reparaciones.datos_json` | `seguimientos`, `orden_etapas`, `avances`, `notas`, `refacciones`, `garantias`, `seguimiento_publicacion` |
| `Configuracion_empresa.datos_json` | `empresa`, `configuracion_documentos`, `clausulas`, `secuencias_folio` |
| `Historial_ordenes`, `Cambios_reparaciones`, `Historial_papelera`, `Bitacora_ordenes`, `Bitacora_sistema` | `auditoria` + `auditoria_detalle` |
| `Movimientos_abonos` (tipo adjustment/refund) | `ajustes_abono` y `devoluciones` |
| `Solicitudes_abonos` (JSON) | `solicitudes_financieras` con id global y huella; cada libro referencia la solicitud |
| `Permisos_usuarios` con CHECK | `permisos` (catálogo) + `usuario_permiso` |
| `Accesos_temporales.operaciones_json` | `acceso_temporal_permiso` |
| `Importaciones_anteriores` | Eliminada (residuo de migración) |
| `Ordenes.id` = folio | `ordenes.id` interno + `ordenes.folio` UNIQUE |
| Papelera por eventos | `ordenes.eliminada_en` + eventos en `auditoria` |

## 6. Puntos a defender ante el profesor

1. **Normalización**: 1FN (sin listas/JSON en celdas), 2FN (sin dependencias parciales; claves compuestas solo en tablas puente),
   3FN (sin datos derivados: saldos en vistas, nombres vía JOIN, `id_orden` de ajustes vía `abonos`).
2. **Excepción justificada**: `auditoria.id_entidad` es una referencia polimórfica sin FK (identifica registros de varias tablas).
3. **Por qué SQL solo protege integridad**: FK, CHECK, UNIQUE e inmutabilidad en la base; reglas de negocio en `services`,
   donde se pueden probar y leer.
4. **Dinero en centavos enteros** para evitar errores de redondeo.
5. **Transacción única cambio + auditoría**: nada queda sin rastro.

## 7. Decisiones de implementación de la Fase 1

Confirmadas en la conversación del 4 de noviembre de 2026:

- Ampliar el esquema existente sin sustituir los niveles `complete`, `limited` y `custom` ni los permisos vigentes. El rol de trabajo Administrador/Técnico se modela separado del nivel de autorización.
- Mantener la clave maestra como hash independiente. El estado de configuración inicial se deriva de su existencia y de un administrador activo, mediante `vista_configuracion_acceso`.
- Conservar la contraseña del equipo recuperable mediante cifrado. `credenciales_dispositivos` guarda un sobre AES-256-GCM; la clave no se almacena en SQLite. Su gestión y conexión al guardado se implementarán con los servicios, no en esta fase de modelos.
- Conservar estados y etapas personalizados sin modificar los catálogos al operar: texto alternativo en la orden o en su secuencia de etapas. Se preservan las siete opciones visuales actuales y se añaden Pendiente, Reparado y Cancelado de la petición, además de Irreparable.
- Conservar método de pago libre, moneda EUR, modo de publicación y opciones actuales de documentos. Los costos de refacciones mantienen su significado vigente.
- La conservación transitoria del cargador anterior finalizó el 13 de noviembre de 2026 (Jira HRL-134). Las migraciones 001–013 están archivadas en `src-tauri/migrations/legacy/` y no se registran en el plugin SQL. Solo las pruebas del sistema anterior las ejecutan explícitamente sobre bases aisladas mediante `test_support`. El cargador `db/migrations.rs` aplica en orden las migraciones registradas del esquema normalizado (actualmente 001–005) y rechaza bases del esquema anterior. El retiro de comandos y del plugin anterior sigue pendiente en HRL-135.

Los identificadores Rust de las nuevas capas están en inglés; tablas y columnas SQL en español. `Money` encapsula centavos y operaciones comprobadas; las credenciales protegidas encapsulan su representación y no implementan `Serialize` ni `Debug`. Los structs de registros describen los datos de dominio. La inyección de repositorios y los servicios pertenecen a las fases siguientes.

El SQL entregable es `schema.sql` en la raíz y su copia versionada de migración en `src-tauri/migrations/001_esquema_objetivo.sql`. Esta referencia documentada contiene el mismo DDL. El cargador no depende de que la carpeta `docs/` esté disponible.

Véase [SQLite y migraciones](sqlite.md) para el esquema vigente y `src-tauri/src/db/tests.rs` para las pruebas de integridad y transición.

## Recepción unificada sobre 3NF

`App.vue` presenta acceso o recepción a partir de `useAuthStore`; no existen
espacios de trabajo alternativos ni bases seleccionables en la interfaz.
Se conserva el diseño original mediante las hojas de estilo existentes.
La recuperación Ed25519 comparte `BackendState` y la base `hrl_core_3fn.db`.

Listado, búsqueda, consulta por referencia e historial recorren
`View → composable/store → services/api → commands → services → repositories → db`.
`OrderService` encapsula validación y autorización, y recibe repositorio,
auditoría, acceso y cifrado por constructor (composición e inyección).
`OrderRepository` abstrae las consultas SQL y conserva las relaciones del
esquema objetivo. Los DTO nuevos mantienen identificadores ingleses y
serialización camelCase; las tablas conservan sus nombres españoles.
Los componentes no importan la API ni los composables del sistema anterior.

## Restauración de apartados sobre 3FN (6 de noviembre de 2026)

WorkshopApp.vue queda como contenedor. useWorkshop coordina navegación,
borradores, impresión y autorización temporal. Empresa, Usuarios, Reparaciones,
Pagos, Evidencias, Bitácora y Finanzas conservan sus vistas originales; sus
viewmodels residen en composables y sus consultas pasan por stores Pinia.
useCompanyStore, useUsersStore y useWorkshopDataStore complementan los stores
de acceso, recuperación y órdenes. legacyOrder solo adapta DTO a presentación.

Los servicios Rust encapsulan autorización, concurrencia y auditoría, reciben
su repositorio principal y dependencias por constructor y usan composición.
Los repositorios abstraen SQL; los DTO separan almacenamiento y transporte.
Los identificadores de código siguen en inglés/camelCase; SQL en español.
Los importes permanecen enteros y no se añade JSON de negocio a SQLite.

El cargador incorpora 002_prefijo_documentos.sql, además de la base 001.
Los recursos binarios se guardan en assets-3fn con nombres controlados por el
servicio y referencias en SQLite. No se migran ni modifican automáticamente
las bases anteriores. La cobertura vigente está en `tests/` y en las pruebas Rust de `src-tauri/src/`; el procedimiento de validación está en `README.md`.

## Cierre del backend normalizado del 13 de noviembre de 2026

`RepairService`, `EvidenceService` y `ReportService` reciben sus repositorios colaboradores en el constructor. Los contratos `LedgerReader`, `OrderReader` y `AuditWriter` permiten sustituir lecturas y escrituras de auditoría en las pruebas. La composición conserva una transacción por operación. Los servicios encapsulan la autorización y el flujo de cambios, los traits abstraen el acceso a datos y sus implementaciones aportan polimorfismo para probar errores después de una escritura.

La edición de órdenes audita los cambios de correo y entrega estimada con valores opcionales anteriores y nuevos. El contrato de Bitácora transporta `entity` y `entityId` desde las columnas existentes. Empresa valida en Rust el correo opcional y las direcciones web con dominio, siguiendo las reglas del formulario.

El ejecutable registra únicamente comandos de `commands/` y el estado normalizado. Los módulos anteriores y sus pruebas se conservan en `src-tauri/legacy/`, compilados solo con `cfg(test)`. El plugin SQL queda como dependencia de desarrollo para esas pruebas. Los permisos puros están en `domain/permissions.rs`. La producción abre `hrl_core_3fn.db` mediante `db/`.

`BackendServices::with_os_key_store` consulta al repositorio si hay credenciales cifradas antes de abrir DPAPI. Cuando existen, exige el archivo protegido original mediante `OsKeyStore::open_existing`. Las pruebas cubren ausencia, corrupción, conservación del archivo y recuperación al restituir la custodia correcta con el mismo usuario Windows. El procedimiento manual de instalación y respaldo conjunto se documenta en `docs/instalacion-recuperacion-local.md`.

## Cierre de la Fase 4 del 13 de noviembre de 2026

`src/models/finances.ts` contiene el modelo de filtros y agregación de lectura. `src/utils/finances.ts` conserva la conversión de fechas y límites UTC. Los nombres de clasificación son un catálogo de presentación en `src/data/financialClasses.ts`. El modelo conserva las reglas de periodos, moneda y exclusiones ya documentadas. Las escrituras financieras recorren los servicios Rust.

El único `invoke` del árbol activo está en `src/services/api/transport.ts`. El frontend anterior, sus composables, órdenes de demostración y pruebas históricas se conservan en `legacy/frontend/`, fuera de las entradas de compilación. El adaptador `utils/legacyOrder.ts` sigue transformando DTO normalizados para las vistas vigentes.

`test:company` ejecuta las pruebas actuales de notas en `tests/company-notes.test.ts`. La suite renderiza el componente real mediante SSR y prepara el resumen con IPC simulado, incluyendo denegación, datos públicos y garantías. `test:finances` comprueba el modelo de lectura. Las pruebas IPC incluyen la invalidación de respuestas de finanzas, Bitácora y resumen público al cambiar de sesión.

## Fase 5 del 14 de noviembre de 2026

`BackupService` compone el repositorio de snapshots, el servicio de acceso y
`KeyProvider`. Encapsula autorización, captura y restauración. `Records<T>` usa
los contratos `Read`/`Write` para cifrar y verificar por bloques con memoria
acotada; el proveedor de claves abstrae la custodia del perfil Windows. Los
comandos delegan las operaciones y adaptan el selector nativo. `useBackupStore`
orquesta el diálogo y mantiene las credenciales solo mientras está abierto;
`useBackupClose` espera al resultado confirmado antes de cerrar. No hay SQL
de respaldos fuera de su repositorio ni llamadas IPC desde componentes.

La captura unificada de antecedentes utiliza `history.notes` y la migración
003. `equipmentHistoryText` solo formatea la lectura histórica; el composable
de recepción conserva el borrador y el servicio Rust autoriza, valida y audita
la escritura. Se mantienen los identificadores en inglés de los contratos y
las columnas SQL en español. El servicio encapsula la transacción y el
repositorio abstrae la persistencia, siguiendo la composición ya existente.

Las pantallas están en `src/views/` como AccessView, ReceptionView, RepairsView, FinancesView, CompanyView y UsersView. `WorkshopApp.vue` contiene la composición y utiliza `useWorkshop` para navegación y guardas. Cámara y cláusulas delegan estado, validación y ciclo de vida en `useEvidenceCamera` y `useCompanyClauses`. Los días sugeridos de garantía y la actualización tras pagos se orquestan en `useRepairScreen`.

Los composables encapsulan sus referencias reactivas y componen callbacks de eventos. Los componentes conservan su presentación y contratos. Desde el 19 de noviembre de 2026, Finanzas sustituye a la pantalla de Bitácora y su viewmodel de consulta de auditoría. `useFinancialWorkspace` orquesta la pantalla y `models/finances.ts` agrega los totales por moneda, pago y entrega. Se conservan los contratos internos `logbook` y `logbook.view`, así como la auditoría transaccional y el historial de órdenes en Clientes. Las pruebas de vistas están en `tests/phase4.test.ts` y `tests/phase5.test.ts`; el renderizado con IPC simulado no acredita cámara, impresora ni instalación nativa.

## Migraciones vigentes y criterio de instalación (17 de noviembre de 2026)

El usuario aprobó actualizar el criterio de HRLP-10: una instalación nueva crea `hrl_core_3fn.db` y aplica en orden las migraciones registradas del esquema actual, sin ejecutar `legacy/`. La antigua expresión «solo 001 y 002» describía una versión anterior; no impide incorporar migraciones nuevas para cambios autorizados. El backlog original y su auditoría estática conservan aquella redacción como referencia histórica.

| Versión | Archivo registrado | Cambio |
|---|---|---|
| 001 | `001_esquema_objetivo.sql` | Línea base normalizada, FK, vistas y restricciones de inmutabilidad. |
| 002 | `002_prefijo_documentos.sql` | Prefijo para documentos. |
| 003 | `003_antecedentes_equipo.sql` | Antecedentes de equipo en texto unificado. |
| 004 | `004_ocultar_nombres_etapas.sql` | Preferencia de nombres personalizados ocultos del selector. |
| 005 | `005_retirar_etapas_pendientes.sql` | Retirada lógica de etapas pendientes y marca de completado previo. |

El registro está en `src-tauri/src/db/migrations.rs::migrator`. No depende de leer `docs/` en ejecución. `schema.sql` y `BASELINE_SQL` siguen representando la misma línea base 001; las ampliaciones se aplican mediante las migraciones posteriores, sin editar una ya aplicada. `docs/esquema-objetivo.sql` refleja las ampliaciones documentadas.

Las pruebas de base comprueban la aplicación reentrante de cinco migraciones, reapertura, integridad y FK por conexión, rechazo de esquema anterior/checksum modificado y restricciones sobre libros y auditoría. La prueba de migración de etapas comprueba actualización desde 004 a 005 conservando órdenes y fechas. La suite completa de 105 pruebas Rust y `cargo check` pasaron al validar HRLP-67, junto con build y pruebas frontend. No se modificó código ni se deshabilitaron pruebas para ajustar este criterio.
