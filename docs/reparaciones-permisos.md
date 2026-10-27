# Reparaciones con permisos locales

La pestaña Reparaciones y Abrir gestión requieren `repairs.view` y una orden
activa seleccionada. Ya no exigen administración de usuarios. El acceso
Limitado puede consultar el seguimiento, añadir notas públicas o internas,
agregar avances, completar etapas, registrar garantías y refacciones.

Las notas anteriores son de solo lectura para Limitado; un formulario añade
texto sin reemplazarlas. No puede cambiar técnico, retroceder etapas, editar
registros anteriores, borrar, registrar pagos o modificar publicación.
Personalizado respeta Visualizar/Crear/Editar en Reparaciones. Crear permite
agregar registros técnicos; Editar permite modificar técnico y texto de notas.

La migración 006 añade la ficha de seguimiento con revisión y un registro
inmutable de operaciones, autor y fecha. Cada guardado se autoriza en Rust
comparándolo con la ficha almacenada, dentro de una transacción. Los registros
de avances, notas, garantías, refacciones y pagos se confirman junto con ella.
Una revisión desactualizada o un fallo del historial revierte todo el cambio.

Los pagos adicionales del acceso autorizado usan el mismo libro SQLite que
el abono inicial de recepción y no pueden superar el saldo. No se implementan
correcciones o devoluciones, ni eliminación de registros anteriores.

## Seguimiento anterior

Si no hay ficha SQLite y existe `hrl.repair.v1.<folio>`, se valida y se importa
al abrir la reparación con un administrador. El original se conserva sin
reescribirlo. La revisión evita volver a importar o duplicar abonos/notas.
Un usuario restringido puede abrir la pantalla y consultar el original, pero
si aún necesita importación deberá abrirlo primero un administrador para
continuar trabajando. Los datos ilegibles bloquean la escritura y se conservan.

Las notas de impresión consultan el resumen público desde SQLite y esperan
su recuperación antes de imprimir. El backend devuelve únicamente técnico,
nota pública y garantías; no devuelve notas internas en ese resumen. Si todavía
no hay seguimiento importado se conserva la lectura pública del origen anterior.

## Comprobaciones

- PASS: 21 pruebas Rust; seis nuevas cubren acceso Limitado, rechazo de
  reemplazos y pagos, revocación, importación administrativa sin duplicados,
  rollback, persistencia al reabrir y edición personalizada de notas.
- PASS: 22 pruebas Node; la comprobación de componentes incluye la pestaña
  Reparaciones del usuario Limitado, además de los controles de eliminación.
- PASS: compilación TypeScript/Vite y revisión de espacios.
- PASS: lectura de accesibilidad nativa del acceso con Maximizar deshabilitado.

No se introdujeron credenciales reales ni se alteraron registros reales en
las pruebas. La captura nativa continúa fallando con timeout; las transiciones
de ventana se prueban con una ventana simulada que aplica límites. Queda
pendiente el recorrido visual completo con sesiones reales.
