# Documentación vigente

Esta carpeta conserva las reglas y procedimientos que requiere el desarrollo
de HRL CORE. El seguimiento de tareas se consulta en Jira, proyecto
**HRL Proyecto (HRLP)**.

| Tema | Documento |
|---|---|
| Arquitectura y dependencias | [arquitectura-objetivo.md](arquitectura-objetivo.md) |
| Esquema base | [esquema-objetivo.sql](esquema-objetivo.sql) |
| SQLite y migraciones | [sqlite.md](sqlite.md) |
| Fases de refactorización | [prompts-fases.md](prompts-fases.md) |
| Accesos y cuentas | [accesos.md](accesos.md) |
| Validación de acceso | [acceso-validacion.md](acceso-validacion.md) |
| Acceso temporal | [acceso-temporal.md](acceso-temporal.md) |
| Perfiles de ventana | [ventana-por-sesion.md](ventana-por-sesion.md) |
| Reparaciones y permisos | [reparaciones-permisos.md](reparaciones-permisos.md) |
| Reglas de dinero | [reglas-pagos.md](reglas-pagos.md) |
| Finanzas | [bitacora.md](bitacora.md) |
| Evidencias | [evidencias.md](evidencias.md) |
| Antecedentes del equipo | [antecedentes-equipo.md](antecedentes-equipo.md) |
| Papelera de órdenes | [papelera-ordenes.md](papelera-ordenes.md) |
| Respaldos portables | [respaldos-portables.md](respaldos-portables.md) |
| Instalación y recuperación | [instalacion-recuperacion-local.md](instalacion-recuperacion-local.md) |
| Contrato del portal y adaptación HRL CORE | [contrato/CONTRATO-FIRESTORE.md](contrato/CONTRATO-FIRESTORE.md) |
| Logos y variantes conservados | [brand/index.html](brand/index.html) |

El DDL que ejecuta la aplicación se compila desde `src-tauri/migrations/`.
La línea vigente es 001–005. Los archivos de `src-tauri/migrations/legacy/`
se utilizan únicamente en pruebas históricas.

Los informes y documentos históricos de estadía se conservan localmente en
`memoria-datos/`; las versiones ya registradas siguen disponibles en Git.
Los diseños, exportaciones antiguas y capturas generadas no son requisitos
de compilación ni fuentes de reglas nuevas.
