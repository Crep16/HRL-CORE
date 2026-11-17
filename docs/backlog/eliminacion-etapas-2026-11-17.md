# Etapas pendientes y nombres personalizados

Solicitud: permitir eliminar una etapa no completada y quitar del selector nombres personalizados creados por el taller.

## Parte implementada: quitar nombres del selector

En Reparaciones, pestaña Historial, abre Agregar avance y selecciona un nombre personalizado guardado. Aparece «Quitar nombre del selector». El diálogo permite cancelar o confirmar. Al confirmar, el nombre deja de ofrecerse como sugerencia en todas las órdenes; las etapas de las órdenes y los avances anteriores conservan el nombre y sus datos.

Las opciones del catálogo y las sugerencias predeterminadas permanecen disponibles. La operación requiere Visualizar Reparaciones y Editar Reparaciones, y una orden activa; una autorización temporal restringida a una orden no permite cambiar las sugerencias compartidas. Se revalida en Rust. Ocultar es una preferencia persistente independiente del catálogo, no un borrado del catálogo ni de la historia. Se puede escribir expresamente un nombre personalizado al crear otro avance; quitarlo del selector no prohíbe su uso como texto.

La migración nueva `004_ocultar_nombres_etapas.sql` crea `nombres_etapas_ocultos`. No se modifican migraciones aplicadas. El esquema objetivo refleja la nueva tabla. La operación de repositorio y la auditoría del servicio se confirman en la misma transacción; si falla la auditoría, la preferencia no cambia. Los comandos delegan y la vista pasa por composable, store y API. El servicio conserva composición e inyección de dependencias; no se añade herencia.

Evidencia: `RepairRepository::stages/hide_stage_name`, `RepairService::hide_stage_name`, comando `hide_normalized_stage_name`, `useRepairScreen::requestHideStageName/confirmStageAction` y `StageRemovalDialog.vue`.

Validación: `cargo check --offline --quiet`, `cargo test --offline --quiet` (101/101, con perfil real de Windows para DPAPI), `npm run test:phase5` (10/10), `npm run test:ipc` (14/14), `npm run test:browser` y `npm run build`, todos aprobados. La prueba nueva de servicio comprueba permisos, protección del catálogo, comparación sin distinguir mayúsculas, conservación de etapas/avances/revisión y rollback ante fallo de auditoría. Se conserva la comprobación de migraciones, ajustada a cuatro migraciones.

En Chrome aislado con IPC simulado se comprobó seleccionar un nombre personalizado, cancelar, confirmar y comprobar que desaparece del selector mientras permanece en el progreso. Captura revisada: `artifacts/phase5/remove-stage-name-mobile.png`, a 375 px. Se mantuvieron colores, fuentes y estructura existentes; el foco inicia en Cancelar, Escape cancela, Tab permanece en el diálogo y los botones miden al menos 44 px. Sin gradientes, iconos ni animaciones nuevos; energía/ritmo/movimiento: 1. La revisión de interfaz y accesibilidad pasó para el control añadido; no sustituye validación nativa.

## Pendiente de aclaración: eliminar una etapa del progreso

Todavía no se permite retirar etapas de la secuencia. El usuario solicitó que nunca se eliminen etapas completadas. Se encontró además que una etapa personalizada se crea junto con un avance inmutable que la referencia: borrar físicamente esa etapa rompería su historial y la FK.

Se consultó al usuario si se debe retirar la etapa pendiente del progreso conservando sus avances con el nombre original, o permitir eliminar únicamente etapas sin avances. Esa respuesta está pendiente. No se cambió la validación actual que conserva las etapas existentes ni se eliminaron avances. Las etapas base y el mínimo actual de dos etapas permanecen como estaban; cualquier cambio de esa regla necesita concretarse al implementar esta parte.

No se modificaron los dos archivos originales del backlog, los tres informes de auditoría ni los cambios de documentación que ya estaban en el árbol de trabajo.
