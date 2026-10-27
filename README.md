# HRL-CORE

Aplicación de escritorio de HRL, con Vue 3, TypeScript y Tauri.

## Flujo actual

- **Clientes** es la recepción de órdenes del mockup: lista a la izquierda, ficha central y estado/evidencias a la derecha.
- **Nueva orden** y **Editar** permiten escribir directamente en la ficha central. **Guardar cambios** confirma la captura y **Cancelar** descarta el borrador.
- **Reparaciones → Abrir gestión** abre el seguimiento de la orden seleccionada. **Volver a clientes** guarda el seguimiento y regresa a la recepción.
- Reparaciones incluye etapas, notas públicas e internas, línea de tiempo, abonos, garantías y refacciones con formularios dentro de la pantalla.

Los estilos de recepción y reparaciones proceden de `Clientes-html/Main.dc.html`
y `Reparaciones (gestión detallada)-html/Reparaciones.dc.html`, compartidos en
la carpeta de mockups. El acceso Completo también dispone de Usuarios.

## Almacenamiento y alcance

La app de escritorio configura la clave maestra y el primer administrador,
guarda las cuentas con hashes de contraseña y exige iniciar sesión. Al cerrar
la app se elimina la sesión, conservando las cuentas. No existe recuperación
automática de la clave maestra; el procedimiento manual sigue por definir.

Las órdenes se guardan en SQLite mediante comandos Rust con autorización.
Cliente, equipo, orden, abono inicial e historial se confirman en una misma
transacción. La ficha completa se conserva además como JSON tipado en la
misma fila para mantener los campos y formatos de recepción. No se cargan
órdenes de ejemplo al iniciar una base vacía.

Si hay órdenes antiguas en `hrl.orders.v1`, el acceso Completo muestra
**Importar órdenes guardadas**. La importación se hace una sola vez sobre
una base sin órdenes, conserva el original de localStorage y revierte todos
los cambios si falla un registro. No combina ni sobrescribe bases existentes.
El seguimiento por orden (`hrl.repair.v1.<folio>`) y Empresa siguen en
localStorage; su migración a SQLite está pendiente.

El acceso Limitado consulta todas las órdenes y crea recepciones sin pagos.
La administración de usuarios requiere acceso Completo y clave maestra.
Reparaciones conserva por ahora su implementación anterior, disponible solo
para el acceso Completo mientras se conectan sus operaciones al backend.
El borrado está deshabilitado para conservar el historial. La reducción de
precios y corrección de pagos esperan las reglas pendientes de HRL-9.

Empresa permite configurar datos, logo, cláusulas, folios, garantías sugeridas, región y formatos de impresión. Se generan notas de entrada/salida; falta comprobar la impresión nativa con el equipo acordado. Véase `docs/auditoria-empresa.md`.

La conexión con Firebase, publicación en el portal y captura/carga de evidencias
están pendientes. Sus controles aparecen deshabilitados cuando corresponde.
Las cuatro miniaturas de la recepción son ejemplos del mockup.

Si falla la recuperación o importación, se muestra un aviso y no se permite
crear órdenes hasta resolver el problema. Un fallo al guardar conserva el
borrador; una edición desactualizada se rechaza para evitar sobrescribir cambios.

## Comandos

- `npm install`
- `npm run dev`
- `npm run build` — comprueba TypeScript y genera la compilación.
- `npm run tauri dev`
- `node --experimental-strip-types --test tests/*.test.mjs`
- `cargo test --offline --manifest-path src-tauri/Cargo.toml --lib`

`npm run dev` sirve la interfaz web; el acceso a cuentas y órdenes requiere
ejecutar la app con Tauri. Véanse `docs/sqlite.md`, `docs/accesos.md` y
`docs/acceso-validacion.md` para la evidencia y los límites de la validación.

En PowerShell con scripts deshabilitados, usa `npm.cmd`.
