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
