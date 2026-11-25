# Cronología de commits e historias

Referencia: HEAD `d807455` (`Evita consultas concurrentes que impiden preparar el QR del comprobante`, 2026-11-19). Fechas y asuntos tomados de `git log`; se contrastaron rutas con el historial de archivos. Para los sprints se consultaron `trazabilidad.md` y `historias-jira.csv`.

## Commits por semana

Las semanas inician en lunes. Se ejecutó el comando solicitado `git log --since=2026-10-19 --date=short --pretty=format:"%ad"`: produjo 90 fechas, de 2026-10-26 a 2026-11-19. El historial sin filtro también muestra tres commits fechados 2026-10-19 (`60f9aba`, `5b0c420`, `85fc1c4`), que se incluyen en la primera semana para cubrir el intervalo calendario desde esa fecha. Total del intervalo por fechas visibles en el log: 93 commits.

| Semana (lunes a domingo) | Commits | Temas principales observados en mensajes y rutas |
|---|---:|---|
| 2026-10-19 a 2026-10-25 | 3 | Inicialización del proyecto, primer commit titulado “victor” y pantalla de órdenes con reorganización de la UI. |
| 2026-10-26 a 2026-11-01 | 25 | Acceso y cuentas, permisos, órdenes persistentes, reparaciones, evidencias, abonos y devoluciones, notas de empresa y resumen financiero. |
| 2026-11-02 a 2026-11-08 | 2 | Documentación de arquitectura y esquema objetivo; cambios previos al refactor. |
| 2026-11-09 a 2026-11-15 | 34 | Auditoría del backlog, servicios normalizados y MVVM, pruebas de finanzas, reintentos de abonos y respaldos portables. |
| 2026-11-16 a 2026-11-22 | 29 | Cambios de reparaciones y evidencias, protección de borradores, desglose financiero y contrato, cola y conexión del portal. |
| **Total** | **93** | |

La fecha de corte del comando `--since` dejó fuera las tres entradas que el historial sin filtro fecha el 2026-10-19. Se muestran aparte para que el conteo por semana corresponda al rango calendario solicitado.

## Primer y último commit relacionado por historia

La columna Sprint se tomó de `historias-jira.csv`: `trazabilidad.md` no incluye el sprint en sus filas. Un commit se considera relacionado cuando el mensaje y las rutas modificadas aportan a la historia; no se cuentan cambios generales de documentación que solo mencionan el mismo tema.

| Historia | Sprint asignado | Primer commit relacionado | Último commit relacionado |
|---|---|---|---|
| HU-01 | 1 | `5cabc7f` (2026-10-26) — SQLite para usuarios y migración de permisos | `89085ec` (2026-11-14) — selector de usuarios con búsqueda al acceso |
| HU-02 | 1 | `0d56737` (2026-10-29) — permisos temporales con autoría y auditoría | `770d63a` (2026-11-14) — termina vistas y composables; incluye el flujo frontend de acceso temporal |
| HU-03 | 2 | `85fc1c4` (2026-10-19) — pantalla de órdenes y componentes de recepción | `60feef2` (2026-11-17) — protege borradores de recepción al navegar y cerrar |
| HU-04 | 2 | `76f1e8b` (2026-10-29) — fotografías locales vinculadas a una orden | `37137e3` (2026-11-17) — admite evidencia PDF y MP4 |
| HU-05 | 3 | `3ba7496` (2026-10-26) — conserva avances de reparaciones y componentes de taller | `37451f0` (2026-11-19) — abre refacciones y garantías en diálogos |
| HU-06 | 3 | `c216d51` (2026-10-27) — persistencia transaccional de órdenes | `5fe8fff` (2026-11-19) — desglose financiero por pago y entrega |
| HU-07 | 3 | `7d651f1` (2026-10-29) — conserva abonos y registra ajustes y devoluciones | `c70df9b` (2026-11-14) — confirmación de abonos y reintentos exactos |
| HU-08 | 4 | `a42fd55` (2026-10-29) — bitácora y resumen financiero local | `5fe8fff` (2026-11-19) — desglose de ganancias por pago y entrega |
| HU-09 | 4 | `e6ca818` (2026-10-29) — configuración de empresa y notas de impresión | `d807455` (2026-11-19) — coordina consultas de QR del comprobante |
| HU-10 | Cierre | `c84b99e` (2026-11-14) — define respaldos portables manuales y automáticos | `8354650` (2026-11-14) — agrega respaldos y copia automática al cerrar |
| HU-11 | Cierre | `9226f73` (2026-11-17) — propuesta de contrato para publicación del portal | `8945bb1` (2026-11-19) — corrige la fecha de avances públicos del portal |

## Comparación con el sprint asignado

**Sin dato** para las once historias. El CSV asigna sprint, pero `trazabilidad.md` no lo registra y los archivos revisados no establecen fechas de inicio y fin para Sprint 1, 2, 3, 4 o Cierre. La documentación encontrada menciona una fecha de cierre de Sprint 5, que no basta para determinar los límites de los sprints asignados. Por lo tanto, no se puede afirmar con evidencia si la fecha del primer commit cayó dentro o fuera de su sprint.

## Pruebas Rust listadas

Se ejecutó `cargo test -- --list` desde `src-tauri`. Resultado: código de salida 0; listó 131 pruebas (además de targets con 0 pruebas). **Sí aparecen pruebas definidas en `src-tauri/legacy/`**: se listan bajo sus módulos, por ejemplo `access::tests::setup_login_logout_and_revocation`, `orders::tests::order_and_audit_survive_reopen_and_reject_stale_updates`, `payments::tests::corrections_are_linked_compensating_entries_with_admin_only_access_and_atomic_audit` y `repairs::tests::technical_tracking_survives_database_reopen`. No llevan el prefijo `legacy::`; `src-tauri/src/lib.rs` las enlaza mediante `#[path = "../legacy/..."]` bajo `#[cfg(test)]`.