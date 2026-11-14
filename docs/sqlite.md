# Esquema local

La migración 003 añade `ordenes.antecedentes_equipo` como TEXT nullable para
la captura libre autorizada por el usuario. Conserva las columnas y valores
anteriores; NULL y texto vacío tienen significados distintos de presentación.
Véase [antecedentes del equipo](antecedentes-equipo.md). La base 001 permanece
sin modificaciones y las extensiones se cargan numeradas desde `db/`.

## Arranque unificado 3NF

La interfaz de `App.vue` consulta únicamente los contratos de `commands/`
sobre `hrl_core_3fn.db`. Ya no existen los contenedores `LegacyApp.vue` ni
`NormalizedWorkspace.vue`, ni una selección de bases o espacios en la UI.
Se retiró la precarga de `sqlite:hrl_datos.db` en `tauri.conf.json`; no se
importan ni eliminan sus datos. Las descripciones de las migraciones antiguas
que siguen a continuación son antecedentes del sistema anterior.

La recepción consulta `list_reception_orders` con búsqueda literal por folio,
cliente o serie y páginas de hasta 100 filas. `find_reception_order` encuentra
el folio exacto y, en su defecto, un ID numérico; `get_order` obtiene la ficha
completa. Todas las consultas revalidan `clients.view` en Rust. El listado
lee los saldos de `vista_saldos_orden` y omite los secretos del equipo.
La consulta explícita de contraseña conserva el cifrado AES-256-GCM y solo
descifra tras autorizar el detalle. Los DTO financieros siguen usando i64.

`get_reception_order_history` exige `logbook.view`, pagina hasta 50 eventos
de la orden y obtiene auditoría, operación, autor y cambios desde las tablas
3NF. Los nombres mostrados se consultan en `usuarios`; no se añade una copia
histórica del nombre. Se excluyen los campos sensibles del resultado.
Estas consultas son de lectura y no modifican libros ni auditoría.

La configuración y recuperación Ed25519 usan el mismo `BackendState` y la
misma base 3NF. Ninguna vista llama a la API directamente: pasa por Pinia
o por un composable, y solo `services/api` invoca los comandos.

## Esquema y cargador vigentes

La fuente de verdad es `docs/esquema-objetivo.sql`. La migración base
`001_esquema_objetivo.sql` y el entregable `schema.sql` contienen el mismo DDL.
El cargador explícito `src-tauri/src/db/migrations.rs` aplica esa base y
`002_prefijo_documentos.sql` mediante la tabla de control `_sqlx_migrations`.
Las migraciones anteriores están archivadas y se ejecutan únicamente en las
pruebas del sistema histórico.

Cada conexión activa `PRAGMA foreign_keys=ON` antes de las transacciones.
El cargador rechaza una base del esquema anterior y las sumas de comprobación
impiden modificar una migración aplicada. Las pruebas usan bases SQLite
temporales y conexiones en memoria.

| Agregado | Tablas principales |
| --- | --- |
| Acceso | `usuarios`, `roles`, `niveles_acceso`, `permisos`, `usuario_permiso`, `claves_maestras` |
| Autorización temporal | `accesos_temporales`, `acceso_temporal_permiso` |
| Recepción | `clientes`, `equipos`, `ordenes`, `credenciales_dispositivos`, `secuencias_folio` |
| Seguimiento | `seguimientos`, `orden_etapas`, `avances`, `notas`, `refacciones`, `garantias`, `seguimiento_publicacion` |
| Dinero | `abonos`, `ajustes_abono`, `devoluciones`, `gastos_directos`, `solicitudes_financieras` |
| Evidencias | `evidencias` |
| Empresa | `empresa`, `configuracion_documentos`, `clausulas` |
| Auditoría | `auditoria`, `auditoria_detalle`, `operaciones_auditoria` |

El dinero usa centavos enteros. `vista_saldos_orden` calcula los saldos a partir
de los libros inmutables. Los servicios confirman el cambio y su auditoría en
la misma transacción. Los reintentos financieros usan identificador único y
huella de solicitud.

Las rutas actuales parten del directorio de configuración de Tauri. SQLite
se guarda en `hrl_core_3fn.db`, las fotos y logos en `assets-3fn/`, y la clave
externa protegida por DPAPI en `keys/device-password-v1.dpapi`. El arranque
exige esa clave existente cuando hay credenciales cifradas. El procedimiento
de recuperación conjunta está en `instalacion-recuperacion-local.md`.

`cargo test --offline` verifica migraciones, claves foráneas, libros y auditoría
inmutables, concurrencia, idempotencia, revisión, permisos y reversión de cambios.
El ejecutable registra los comandos normalizados de `commands/`.

## Antecedentes del esquema anterior

Esta sección conserva las referencias históricas de las migraciones archivadas en
`src-tauri/migrations/legacy/` y del código de regresión de `src-tauri/legacy/`.
La especificación actual del esquema es `docs/esquema-objetivo.sql`.

Desde la migración 013, los nombres físicos están en español. Los nombres ingleses
que aparecen más abajo describen la evolución histórica de las migraciones 001 a 012;
sus equivalencias están en `propuesta-nombres-db.md`. La referencia de esa etapa era
`estructura-db.sql`. `cambio-nombres-db.md` documenta el cambio y su alcance.

La migración 012 añade `order_evidence`: referencia por orden, identificador de
archivo, tipo/tamaño, origen, descripción, autor, fecha, retiro y revisión.
Las imágenes se guardan fuera de SQLite en el directorio local de datos de Tauri;
véase `evidencias.md`. Los cambios de metadatos y Bitácora se confirman juntos.
Los archivos originales y su autoría se conservan al retirar una foto.

La migración 001 conserva usuarios y permisos. La migración 002 añade
clientes, equipos, órdenes, avances, notas, refacciones, garantías, pagos
e historial. Ambas están registradas para `sqlite:hrl_datos.db`.

Los importes se almacenan en unidades menores enteras (centavos para
MXN). Los abonos originales son positivos. La migración 010 incorpora ajustes
con signo y devoluciones totales del saldo a favor en un libro separado;
véase `reglas-pagos.md`. La migración 011 incorpora el acceso temporal auditado,
sin guardar contraseñas ni modificar los permisos permanentes.
No se infiere que un estado de reparación salde o elimine un adeudo.

Las relaciones impiden borrar registros referenciados. Desactivar a un
usuario conserva su autoría. El historial no admite editar ni borrar
eventos y su catálogo de campos excluye claves. Los comandos de recepción
confirman el historial junto con la orden. El esquema por sí solo no crea
eventos automáticamente; los flujos restantes deben conectarse al backend.

La garantía guarda cobertura y duración; su inicio debe derivarse de la
entrega de la orden. No se almacena una fecha de inicio independiente.

Pruebas: `node --test tests/workshop-schema.test.mjs` verifica la actualización
sobre usuarios existentes, reapertura de una orden con sus relaciones,
restricciones e historial, y rollback ante un evento con autor inválido.
`cargo check --offline --manifest-path src-tauri/Cargo.toml` comprueba el
registro de la migración en Rust.

## Recepción conectada, HRL-14

La migración 004 añade la ficha completa serializada y el registro de la
importación anterior. Los comandos de recepción mantienen ficha y relaciones
en una transacción; la ficha conserva formatos y campos que todavía no
tienen operaciones propias. Las cantidades normalizadas se guardan en
centavos junto con la moneda. Una orden existente no cambia de moneda.

La consulta revalida permisos. Un permiso de crear sin visualizar no devuelve
órdenes existentes, pero permite solicitar un folio disponible y crear una
recepción. Folios repetidos y ediciones con una ficha original desactualizada
se rechazan. La interfaz conserva el borrador ante errores de guardado.

La importación de `hrl.orders.v1` requiere un administrador y una base vacía.
Se conserva el original; el marcador de importación se confirma junto con
todas las órdenes. Los autores son los operadores de la importación, no
autores históricos supuestos. Los datos anteriores no indican la fecha real
de cada cobro o entrega: el abono inicial histórico usa fecha vacía y la
entrega histórica no recibe una fecha inventada. La garantía no debe iniciar
automáticamente con una fecha de importación. La moneda antigua se toma de
la configuración de Empresa al importar, pues antes no se guardaba por orden.

El abono inicial nuevo requiere permiso de pagos y método libre. No se corrige
un pago editando su ficha: se añade un ajuste ligado al original. Reducir el
precio conserva los movimientos y puede producir saldo a favor. La migración 006 incorpora el seguimiento y
sus abonos adicionales al mismo libro de pagos, con importación administrativa
del origen anterior sin duplicación; véase `reparaciones-permisos.md`.

El historial de alta y cambios de campos compatibles se confirma junto con
la orden y excluye la clave del equipo. La clave del equipo se conserva en la
ficha local; no debe confundirse con las claves de acceso protegidas por hash.

Pruebas Rust adicionales: reapertura con orden completa, rechazo sin sesión,
rechazo de edición desactualizada, rollback si falla el historial, importación
duplicada sin residuos, conservación de fechas históricas desconocidas y
rechazo de pagos/edición/importación para acceso Limitado. La verificación
nativa completa con una cuenta real y la revisión visual siguen pendientes.

## Papelera recuperable, HRL-19

La migración 005 añade `order_visibility_events`. Cada eliminación o
restauración guarda orden, autor, acción y fecha en un registro inmutable.
La acción más reciente determina si aparece en recepción o en Papelera.
No se borran físicamente cliente, equipo, orden, cobros ni historial.

El comando revalida `orders.delete` en SQLite y compara la ficha enviada
con la guardada para rechazar acciones desactualizadas. El acceso Completo
tiene ese permiso; Personalizado lo obtiene con Borrar en Clientes.
Limitado no lo recibe. Consultar Papelera requiere además Visualizar Clientes.

Las órdenes eliminadas no se editan; primero deben restaurarse. Sus folios
continúan ocupados para no enlazar un seguimiento anterior a una orden nueva.
Repetir una solicitud ya aplicada no duplica el registro de eliminación.

Cuatro pruebas adicionales verifican eliminación/restauración, cobros e
historial conservados, folio no reutilizable, autoría, permisos revocados,
rechazo de solicitudes desactualizadas, fallo de registro y reapertura del
archivo SQLite. Todas usan datos temporales; no se eliminaron registros reales.
