# Tratamiento de órdenes y cancelación de cambios

Decisión del usuario del 29 de octubre de 2026 para HRL-19:
**«Por ahora, conservar solo la papelera».** No se añade un apartado Archivo.

## Órdenes activas y papelera

Una orden permanece en el listado activo hasta que una persona autorizada la
envía expresamente a la papelera. Entregarla, cobrarla o cambiar su estado técnico
no la envía automáticamente a papelera. No existe un plazo de borrado automático.

Enviar a papelera retira la orden del listado activo y permite restaurarla.
Conserva el folio, cliente, equipo, precio, abonos, ajustes, devoluciones,
seguimiento, garantías, fotografías y Bitácora. No cancela la deuda ni devuelve
dinero. Su folio continúa ocupado y no puede asignarse a otra orden.

La papelera no es un mecanismo de borrado físico. El código no elimina las filas
de la orden ni los archivos de sus fotografías. La restauración recupera la
visibilidad y no crea otra orden. Para modificar la ficha, registrar seguimiento
o agregar evidencias, primero debe restaurarse.

Consultar la papelera requiere Visualizar Clientes. Enviar y restaurar requieren
Borrar Clientes, comprobado en Rust; el acceso Completo lo tiene y Personalizado
lo obtiene mediante ese permiso. Una autorización temporal válida puede prestar
la facultad dentro de su alcance. Cada acción conserva autor, fecha y folio en
`Historial_papelera` y se refleja en Bitácora.

La eliminación pide confirmación. Si la ficha cambió desde que se abrió, el backend
rechaza la operación para no actuar sobre una versión desactualizada.

## Cancelar una captura o edición

Cancelar una orden sin cambios cierra el borrador. Si se modificaron datos,
el diálogo pregunta si se desean descartar. Seguir editando o presionar Escape
conserva el borrador; Descartar cambios lo retira y conserva lo guardado antes.
Los botones se bloquean durante una operación de guardado.

La confirmación aplica tanto a una recepción nueva como a la edición de una orden.
Descartar una recepción nueva no crea una fila en SQLite. No revierte abonos ni
otras operaciones que ya se hayan confirmado por separado.

## Referencias y estado

- Búsqueda por folio, cliente y serie: `src/composables/useOrders.ts`.
- Edición, selección y cancelación: `src/WorkshopApp.vue`.
- Confirmación de descarte: `src/components/orders/OrderDiscardDialog.vue`.
- Confirmación de papelera: `src/components/orders/OrderDeleteDialog.vue`.
- Persistencia y autorización: `src-tauri/src/orders.rs`.
- Historia física actual: `Historial_papelera`, procedente de las migraciones 005 y 013.

La compilación TypeScript/Vite completó después de incorporar la confirmación.
No se ejecutaron suites ni recorridos funcionales, conforme a la instrucción del
usuario. HRL-19 queda con el acuerdo funcional documentado y la implementación
preparada; este documento no acredita una prueba nativa ni una revisión visual.
