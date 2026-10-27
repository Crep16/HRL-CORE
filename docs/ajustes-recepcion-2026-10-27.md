# Ajustes de recepción y sesión

## Cabecera y consulta

Rama `fix/HRL-13-session-header-layout`.

La cabecera muestra el nombre real de la sesión y su nivel (Completo,
Limitado o Personalizado) junto a Cerrar sesión. La distribución separa
navegación, impresión y sesión; a menor anchura pasan a filas independientes.
Los nombres largos pueden partirse, sin ocultar el nivel de acceso.

Se añaden SVG al sistema existente para Clientes (ficha), Bitácora (libro),
Empresa (edificio), Usuarios (personas), sesión (persona), estado (lista)
y Reparaciones (herramienta). Identifican apartados y no sustituyen sus
etiquetas. Son decorativos para accesibilidad y no reciben foco.

En consulta, Pagado y Entregado se muestran como valores de solo lectura.
El selector de estado no presenta una flecha si no se puede editar. Se
retira la indicación de escribir en modo consulta y la mención de publicación
web todavía no implementada. Las notas vacías se identifican sin inventar
contenido. No se modifican órdenes al aplicar estos cambios.

Dirección: aplicación de trabajo HRL para el personal del taller, conservando
marca, tipografía y paleta actuales. ENERGY 1 / RHYTHM 1 / MOTION 1. El azul
se reserva para acciones y selección; los iconos usan el color del texto.

## Verificación del incremento

- PASS: compilación TypeScript/Vite.
- PASS: la accesibilidad de la app nativa muestra «Hector», «Acceso completo»
  y Cerrar sesión en ese orden, así como los estados de consulta actualizados.
- PASS: los iconos mantienen etiquetas visibles y `aria-hidden`.
- PASS: revisión de espacios con `git diff --check`.
- Los contrastes se calculan para texto de cabecera, metadatos y controles
  deshabilitados sobre el fondo de HRL; no se infieren a partir de una captura.

La captura nativa devolvió `window capture timed out`, incluso después de
recargar la vista. La inspección visual completa y de tamaños de ventana
sigue pendiente; no se presenta como una comprobación visual aprobada.
La consulta nativa no modificó registros ni configuró credenciales.

## Eliminación y papelera

Rama `fix/HRL-19-order-deletion`, basada en los ajustes anteriores.

Se encontró un botón Eliminar deshabilitado de forma fija y un manejador
que solo mostraba un error. Se sustituyen por una acción autorizada en Rust,
un diálogo de confirmación y un listado Papelera con Restaurar orden.
La confirmación explica qué se conserva y permite cancelar con Escape;
durante el guardado sus controles quedan deshabilitados. Cancelar recibe
el foco inicial; al confirmar se dirige el foco a la orden siguiente.

La eliminación conserva registros e historial. El folio no se libera y el
seguimiento local no se borra. Una orden en Papelera no se edita ni abre
Reparaciones hasta restaurarla. Eliminar y restaurar requieren permiso,
comprobado en cada transacción; se registran actor y fecha.

## Comprobación final

- PASS: 22 pruebas Node, incluida renderización de componentes con sesiones
  de prueba Completo, Limitado y Personalizado. Comprueba nombre/nivel y que
  Eliminar/Restaurar no estén habilitados para Limitado.
- PASS: 13 pruebas Rust; cuatro nuevas cubren eliminación y recuperación,
  reapertura, preservación de cobros e historial, permisos y fallos.
- PASS: TypeScript/Vite y `cargo check --offline`.
- PASS: contrastes calculados de cabecera: blanco 12.35:1, metadatos 9.62:1,
  controles deshabilitados 6.35:1; texto secundario en blanco 7.63:1 y texto
  de estados confirmados 7.43:1.
- PASS: revisión de espacios y etiquetas/foco definidos en el diálogo.

El diálogo y los tamaños de ventana no tienen un recorrido nativo completo
certificado porque la captura de la ventana no está disponible. La prueba
de componentes es renderización con datos temporales, no una captura visual
ni una eliminación real. Las limitaciones de captura anteriores siguen vigentes.
