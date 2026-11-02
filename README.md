# HRL-CORE

Aplicación de escritorio de HRL, con Vue 3, TypeScript y Tauri.

## Flujo actual

La decisión vigente es conservar solo papelera recuperable, sin Archivo. Cancelar
una captura o edición con cambios pide confirmación antes de descartar el borrador.
Véase [tratamiento de órdenes](docs/papelera-ordenes.md).

- **Clientes** es la recepción de órdenes del mockup: lista a la izquierda, ficha central y estado/evidencias a la derecha.
- **Nueva orden** y **Editar** permiten escribir directamente en la ficha central. **Guardar cambios** confirma la captura y **Cancelar** descarta el borrador.
- **Reparaciones → Abrir gestión** abre el seguimiento de la orden seleccionada. **Volver a clientes** guarda el seguimiento y regresa a la recepción.
- Reparaciones incluye etapas, notas públicas e internas, línea de tiempo, abonos, garantías y refacciones con formularios dentro de la pantalla.

Los estilos de recepción y reparaciones proceden de `Clientes-html/Main.dc.html`
y `Reparaciones (gestión detallada)-html/Reparaciones.dc.html`, compartidos en
la carpeta de mockups. El acceso Completo también dispone de Usuarios.

## Almacenamiento y alcance

El esquema usa nombres en español desde la migración 013: `Clientes`, `Equipo`,
`Ordenes`, `Abonos`, `Usuarios` y las tablas de historial/configuración. Las columnas
siguen la propuesta aprobada (`id_cliente`, `articulo`, `precio_centavos`, etc.).
Las migraciones anteriores permanecen intactas. Véanse
[estructura actual sin datos](docs/estructura-db.sql) y
[detalle del cambio](docs/cambio-nombres-db.md).

Las evidencias de una orden guardada se agregan desde cámara web o selección
múltiple de archivos JPG, PNG y WebP (hasta 20 MB por foto). Tauri conserva una
copia en su directorio local de datos, con una carpeta por orden; SQLite guarda
referencias, descripciones y autoría. Galería, retiro recuperable y restauración
se conectan al backend y registran cambios en Bitácora. El respaldo debe incluir
tanto SQLite como la carpeta de imágenes. Véase [Evidencias locales](docs/evidencias.md).
Este incremento tiene compilación comprobada; no se ejecutaron pruebas ni un
recorrido con cámara física por indicación del usuario.

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

Pagado se calcula desde el libro de cobros, no desde una marca manual. Cada
guardado de Reparaciones actualiza la consulta de Clientes. Las órdenes
entregadas muestran pago y entrega; el estado técnico reaparece al editar.
Los avisos comparten un contenedor centrado debajo de la navbar. Véase
`docs/actualizacion-pagos-y-avisos.md`.
El seguimiento de Reparaciones también usa SQLite con autorización por
operación. El seguimiento anterior (`hrl.repair.v1.<folio>`) se importa al
abrirlo con un administrador, conservando el original. Empresa recupera su
configuración de SQLite; el primer guardado incorpora la configuración local
anterior. Los nuevos guardados incluyen auditoría en la misma transacción.

El acceso Limitado consulta todas las órdenes y crea recepciones; por defecto
no registra pagos. Usuarios permite habilitar abonos y devoluciones con permisos
independientes. La administración de usuarios requiere su permiso y clave maestra.
Reparaciones está disponible para el acceso Limitado: consulta, nuevas notas,
avances de etapa, garantías y refacciones, sin edición de registros anteriores
por defecto. También tiene una pestaña propia con la orden activa seleccionada.
Eliminar requiere el permiso correspondiente y una confirmación. La orden
pasa a **Papelera**, desde donde puede restaurarse; se conservan su folio,
historial y cobros. El acceso Limitado no recibe eliminación ni restauración como
permiso permanente. Reducir el precio puede generar saldo a favor; se devuelve
completo de forma explícita. Las correcciones son ajustes ligados al abono
original, sin borrar sus movimientos. Véase `docs/reglas-pagos.md`.

Una acción restringida permite solicitar autorización temporal con usuario y
contraseña de quien posea el permiso. La sesión del operador se conserva; el
préstamo termina al salir del apartado, cambiar de orden de Reparaciones o
cerrar sesión. Bitácora registra quién autorizó, quién operó y el inicio, uso y
fin del permiso. Véase `docs/acceso-temporal.md`.

Empresa permite configurar datos, logo, cláusulas, folios, garantías sugeridas, región y formatos de impresión. Se generan notas de entrada/salida; falta comprobar la impresión nativa con el equipo acordado. Véase `docs/auditoria-empresa.md`.

Bitácora permite consultar operaciones internas con el permiso `logbook.view`.
Incluye Historial de cambios y Resumen financiero de órdenes, con filtros,
clasificaciones independientes de pago y entrega, costos directos y totales
separados por moneda. El historial incorpora también usuarios, permisos y
Empresa. Los registros no pueden editarse ni borrarse. Véanse alcance y
límites históricos en `docs/bitacora.md`.

La conexión con Firebase, publicación en el portal y captura/carga de evidencias
están pendientes. Sus controles aparecen deshabilitados cuando corresponde.
Las cuatro miniaturas de la recepción son ejemplos del mockup.

Si falla la recuperación o importación, se muestra un aviso y no se permite
crear órdenes hasta resolver el problema. Un fallo al guardar conserva el
borrador; una edición desactualizada se rechaza para evitar sobrescribir cambios.

## Archivo de vistas para Figma

`npm.cmd run export:figma-views -- AAAA-MM-DD` genera el inventario de vistas
desde la primera apertura en `artifacts/figma-views/`. `index.html` permite
revisar los HTML estáticos. Incluye estilos, logos SVG, medidas, estados,
variables de diseño y fuentes del código; usa exclusivamente datos ficticios.
No crea un archivo nativo de Figma ni accede a la base real. Las fuentes web
mantienen su dependencia de conexión. `npm.cmd run validate:figma-views`
comprueba la integridad y los estados principales del paquete generado.

## Comandos

El acceso tiene tamaño fijo de 1000 × 680 píxeles lógicos. Configurar el primer
administrador usa 1000 × 760. Al entrar, el taller abre a 1280 × 760 y permite
reducirse hasta 1120 × 680. Al cerrar sesión vuelve al tamaño fijo de acceso.
Usuarios utiliza paneles independientes para lista y editor; su guardado
permanece visible aunque el contenido necesite desplazarse.

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
