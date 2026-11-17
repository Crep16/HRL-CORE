# Etapas pendientes y nombres personalizados

Solicitud: permitir eliminar una etapa no completada y quitar del selector nombres personalizados creados por el taller.

## Parte implementada: quitar nombres del selector

En Reparaciones, pestaña Historial, abre Agregar avance y selecciona un nombre personalizado guardado. Aparece «Quitar nombre del selector». El diálogo permite cancelar o confirmar. Al confirmar, el nombre deja de ofrecerse como sugerencia en todas las órdenes; las etapas de las órdenes y los avances anteriores conservan el nombre y sus datos.

Las opciones del catálogo y las sugerencias predeterminadas permanecen disponibles. La operación requiere Visualizar Reparaciones y Editar Reparaciones, y una orden activa; una autorización temporal restringida a una orden no permite cambiar las sugerencias compartidas. Se revalida en Rust. Ocultar es una preferencia persistente independiente del catálogo, no un borrado del catálogo ni de la historia. Se puede escribir expresamente un nombre personalizado al crear otro avance; quitarlo del selector no prohíbe su uso como texto.

La migración nueva `004_ocultar_nombres_etapas.sql` crea `nombres_etapas_ocultos`. No se modifican migraciones aplicadas. El esquema objetivo refleja la nueva tabla. La operación de repositorio y la auditoría del servicio se confirman en la misma transacción; si falla la auditoría, la preferencia no cambia. Los comandos delegan y la vista pasa por composable, store y API. El servicio conserva composición e inyección de dependencias; no se añade herencia.

Evidencia: `RepairRepository::stages/hide_stage_name`, `RepairService::hide_stage_name`, comando `hide_normalized_stage_name`, `useRepairScreen::requestHideStageName/confirmStageAction` y `StageRemovalDialog.vue`.

Validación: `cargo check --offline --quiet`, `cargo test --offline --quiet` (101/101, con perfil real de Windows para DPAPI), `npm run test:phase5` (10/10), `npm run test:ipc` (14/14), `npm run test:browser` y `npm run build`, todos aprobados. La prueba nueva de servicio comprueba permisos, protección del catálogo, comparación sin distinguir mayúsculas, conservación de etapas/avances/revisión y rollback ante fallo de auditoría. Se conserva la comprobación de migraciones, ajustada a cuatro migraciones.

En Chrome aislado con IPC simulado se comprobó seleccionar un nombre personalizado, cancelar, confirmar y comprobar que desaparece del selector mientras permanece en el progreso. Captura revisada: `artifacts/phase5/remove-stage-name-mobile.png`, a 375 px. Se mantuvieron colores, fuentes y estructura existentes; el foco inicia en Cancelar, Escape cancela, Tab permanece en el diálogo y los botones miden al menos 44 px. Sin gradientes, iconos ni animaciones nuevos; energía/ritmo/movimiento: 1. La revisión de interfaz y accesibilidad pasó para el control añadido; no sustituye validación nativa.

## Parte implementada: eliminar una etapa pendiente del progreso

El usuario aprobó retirar una etapa pendiente del progreso conservando sus avances con el nombre original. En Reparaciones, Trabajo y notas, se abre «Administrar etapas pendientes» y se pulsa «Eliminar etapa» junto a la etapa intermedia. El diálogo confirma la acción; cancelar no cambia datos. El porcentaje usa la secuencia activa restante. Recibido y Entregado siguen siendo los extremos protegidos y se conserva el mínimo de dos etapas.

Se exige Editar Reparaciones en Rust, revisión vigente y orden activa. La validación de dominio impide retirar completadas, reordenar o reemplazar etapas, modificar el prefijo completado o combinar retirada con retroceso. La migración nueva `005_retirar_etapas_pendientes.sql` añade retirada lógica y una marca de completado previo: inicializa esta última a partir de las fechas de completado existentes y la conserva desde entonces aunque se retroceda una etapa. No reconstruye fechas que no estén guardadas ni importa bases anteriores.

La fila de `orden_etapas` se conserva con su identificador; el historial mantiene su FK y su nombre. El repositorio devuelve solo etapas activas para el progreso y todas las referencias históricas para los avances. Los guardados posteriores permiten esos avances antiguos y rechazan avances nuevos para una etapa retirada. Reutilizar explícitamente el nombre al agregar un avance reactiva la fila original, sin duplicarla. Se compactan posiciones de las retiradas para evitar colisiones al guardar repetidas veces.

La retirada, revisión y auditoría usan la transacción existente de `RepairService::save`; un fallo revierte todo. No hay borrado físico de avances ni cambios financieros. El historial muestra el nombre original y «Etapa eliminada del progreso» cuando ya no está en la secuencia activa. La retirada del progreso no oculta automáticamente su nombre del selector: son dos acciones independientes.

Pruebas nuevas: `pending_stages_retire_without_losing_history_and_completed_stages_cannot_retire` comprueba permiso Limitado denegado, rechazo de revisión desactualizada, rollback de auditoría, conservación de filas/avances, guardado posterior, rechazo de avances nuevos para retiradas, reutilización del identificador, protección de completadas y protección tras retroceso. `stage_retirement_migration_preserves_existing_completed_stages` aplica la migración sobre la versión 4 y conserva órdenes y fecha de completado.

El recorrido de navegador prueba cancelar, rechazo de guardado conservando etapa y diálogo, reintento exitoso, cambio de 33 % a 50 %, extremos sin eliminación e historial con el nombre original. Capturas revisadas: `artifacts/phase5/remove-pending-stage-error.png` y `pending-stage-removed-mobile.png`. Conserva el diálogo accesible y la apariencia actual; no agrega recursos visuales ni animación. La revisión de interfaz pasó sin desbordamiento del control a 375 px. Es una comprobación con IPC simulado, complementaria a las pruebas reales de servicios SQLite.

Validación final de esta segunda parte: `cargo check --offline --quiet`, `cargo test --offline --quiet` (103/103, con perfil real de Windows para DPAPI), `npm run test:phase5` (10/10), `npm run test:ui` (14/14), `npm run test:browser`, `npm run build` y revisión de espacios, todos aprobados. Persisten las advertencias históricas de Rust por código sin uso; no se deshabilitaron pruebas.

No se modificaron los dos archivos originales del backlog, los tres informes de auditoría ni los cambios de documentación que ya estaban en el árbol de trabajo.
