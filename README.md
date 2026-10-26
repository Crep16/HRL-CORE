# HRL-CORE

Aplicación de escritorio de HRL, con Vue 3, TypeScript y Tauri.

## Flujo actual

- **Clientes** es la recepción de órdenes del mockup: lista a la izquierda, ficha central y estado/evidencias a la derecha.
- **Nueva orden** y **Editar** permiten escribir directamente en la ficha central. **Guardar cambios** confirma la captura y **Cancelar** descarta el borrador.
- **Reparaciones → Abrir gestión** abre el seguimiento de la orden seleccionada. **Volver a clientes** guarda el seguimiento y regresa a la recepción.
- Reparaciones incluye etapas, notas públicas e internas, línea de tiempo, abonos, garantías y refacciones con formularios dentro de la pantalla.

Los estilos proceden de `Clientes-html/Main.dc.html` y `Reparaciones (gestión detallada)-html/Reparaciones.dc.html`, compartidos en la carpeta de mockups. No se implementan otras ventanas por ahora.

## Almacenamiento y alcance

Esta primera implementación guarda órdenes y seguimiento por orden en `localStorage` (claves `hrl.orders.v1` y `hrl.repair.v1.<folio>`). Conserva los cambios al recargar en el mismo origen y perfil. Es una base de interfaz; la integración definitiva con SQLite todavía está pendiente.

Empresa permite configurar datos, logo, cláusulas, folios, garantías sugeridas, región y formatos de impresión. Se generan notas de entrada/salida; falta comprobar la impresión nativa con el equipo acordado. Véase `docs/auditoria-empresa.md`.

La conexión con Firebase, publicación en el portal, autenticación y captura/carga de evidencias están pendientes. Sus controles aparecen deshabilitados cuando corresponde. Las cuatro miniaturas de la recepción son ejemplos del mockup.

Si falla la recuperación de órdenes, se muestra un aviso y se bloquea la escritura de esa sesión para conservar los datos originales. Recupera el almacenamiento y vuelve a abrir la aplicación antes de continuar.

## Comandos

- `npm install`
- `npm run dev`
- `npm run build` — comprueba TypeScript y genera la compilación.
- `npm run tauri dev`

En PowerShell con scripts deshabilitados, usa `npm.cmd`.
