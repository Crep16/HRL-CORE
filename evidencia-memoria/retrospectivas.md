# Retrospectivas por sprint (reconstrucción documental)

## Alcance y fechas

**[EVIDENCIA — CSV local y Jira, board 34, `listJiraBoardSprints(state=all)`]** El CSV asigna HU-01/02 a Sprint 1, HU-03/04 a Sprint 2, HU-05/06/07 a Sprint 3, HU-08/09 a Sprint 4 y HU-10/11 a “Cierre”. Jira lista cuatro sprints programados: Sprint 1, 26–30 oct; Sprint 2, 2–6 nov; Sprint 3, 9–13 nov; Sprint 4, 16–20 nov de 2026. **NO ENCONTRADO EN EL REPOSITORIO/Jira**: calendario para “Cierre”. `docs/evidencia-jira/cronologia.md` reúne 93 commits visibles del 19-oct al 19-nov. La carga tardía a GitHub está indicada por el usuario; por eso las fechas de commit no se tratan como fechas reales del trabajo.

**Discrepancia de estado Jira:** la consulta del 23-nov mostró Sprint 1 como `active` y Sprint 2–4 como `future`, aunque sus fechas programadas ya habían pasado. Eso impide tomar el estado de sprint como evidencia de ejecución/cierre real.

## Sprint 1 — HU-01, HU-02 (fechas programadas: 26–30 oct 2026)

| Elemento | Evidencia y lectura |
|---|---|
| Entregable | Acceso local, usuarios/permisos y acceso temporal. Commits: `5cabc7f` (migración SQLite de usuarios/permisos), `8d33a1f` (configuración/autenticación local), `0d56737` (permisos temporales con autoría/auditoría), `6f4884f` (trabajo limitado transaccional). Ramas observadas: `feat/HRL-11-sqlite-permission-foundation`, `feat/HRL-12-local-authentication`, `feat/HRL-13-operation-permissions`, `feat/HRL-13-limited-repair-access`. Jira HRL-11–13 y HRL-54 aparecen en mensajes/ramas; correspondencia funcional con HU-01/02: [INFERIDO] por títulos y código. | [EVIDENCIA] |
| Qué salió bien | Las pruebas nombradas en trazabilidad cubren configuración, inicio de sesión y fallos de auditoría sin estado parcial. | [INFERIDO] a partir de pruebas listadas; la traza no acredita retrospectiva del equipo. |
| Qué salió mal o se atrasó | Cambios de corrección en ventanas y permisos: `6b8a96a`, `2123f41`, `ee75a75`; es señal de iteración, no prueba de atraso respecto a calendario. | [INFERIDO] |
| Ajuste para el siguiente sprint | El sprint siguiente concentra persistencia y flujo de órdenes en SQLite, según historias y commits posteriores. | [INFERIDO] |
| Evidencia disponible | `5cabc7f`, `8d33a1f`, `0d56737`, `6f4884f`; ramas citadas; `docs/evidencia-jira/trazabilidad.md`; Jira HRL-64 para rango de Sprint 1 y el tema del ticket. | [EVIDENCIA] |

**Preguntas para el autor:** ¿Qué impedimentos ocurrieron? ¿Qué historias se terminaron dentro del sprint? ¿Hubo revisión del usuario/taller? ¿Qué significan las ramas alternativas de HRL-12/13 y cuándo se integraron?

## Sprint 2 — HU-03, HU-04 (fechas programadas: 2–6 nov 2026)

| Elemento | Evidencia y lectura |
|---|---|
| Entregable | Recepción/órdenes y evidencias locales. Commits clave: `c216d51` (persistencia transaccional de órdenes), `76f1e8b` (fotografías locales), `06cc012` (nombres SQL en español e integración), `37137e3` (admite PDF y MP4). Ramas: `feat/HRL-14-sqlite-orders`, `fix/HRL-14-order-storage-validation`, `fix/HRL-21-preserve-repair-storage`. Jira relacionado HRL-14, HRL-19/20 y HRL-21: [INFERIDO] por clave/mensaje y trazabilidad; el histórico no certifica asignación al mismo sprint. | [EVIDENCIA] |
| Qué salió bien | Se agregaron pruebas de persistencia, recuperación y evidencia en trazabilidad; reporte 23-nov incluye recorridos de evidencia aprobados con IPC simulado. | [INFERIDO] |
| Qué salió mal o se atrasó | `fix/HRL-14-order-storage-validation` y `fix/HRL-21-preserve-repair-storage` apuntan a correcciones de validación y preservación; el árbol Git no prueba por sí solo atraso. | [INFERIDO] |
| Ajuste para el siguiente sprint | Iteraciones posteriores agregan seguimiento técnico, pagos, ajustes y devoluciones sobre registros persistidos. | [INFERIDO] |
| Evidencia disponible | Commits/ramas anteriores; `docs/evidencia-jira/trazabilidad.md`; logs 23-nov para interfaz y pruebas. | [EVIDENCIA] |

**Preguntas para el autor:** ¿Qué evidencia fotográfica se probó en el taller? ¿Cuál fue el criterio de aceptación real de HU-03/04? ¿Las fechas de publicación de commits posteriores representan carga tardía de este sprint?

## Sprint 3 — HU-05, HU-06, HU-07 (fechas programadas: 9–13 nov 2026)

| Elemento | Evidencia y lectura |
|---|---|
| Entregable | Reparaciones, libro financiero, pagos y clasificación de órdenes. Commits: `3ba7496` (preservar órdenes/reparaciones/empresa), `7d651f1` (abonos, ajustes, devoluciones), `42b5aff` (retirar etapas pendientes conservando historial), `c70df9b` (confirmación de abonos y reintentos exactos), `5fe8fff` (desglose financiero). Jira relacionado HRL-21/22/23/27: [INFERIDO] desde mensajes y `docs/evidencia-jira/trazabilidad.md`; no acredita sprint real. | [EVIDENCIA] |
| Qué salió bien | Pruebas nombradas cubren dinero entero, reversión de auditoría, persistencia y cálculos; `cargo-test-2026-11-23.txt` registra suite completa aprobada en noviembre. | [INFERIDO] |
| Qué salió mal o se atrasó | Correcciones/repetición en pagos y estados aparecen en `c70df9b` y `42b5aff`; indican ajustes del comportamiento, pero no muestran cuándo se detectaron ni si atrasaron sprint. | [INFERIDO] |
| Ajuste para el siguiente sprint | Los cambios posteriores separan auditoría/finanzas y amplían reportes y filtros. | [INFERIDO] |
| Evidencia disponible | Commits citados; `docs/reglas-pagos.md`; `docs/evidencia-jira/trazabilidad.md`; reporte 23-nov. | [EVIDENCIA] |

**Preguntas para el autor:** ¿Qué errores de operación motivaron las correcciones? ¿Se hicieron pruebas manuales de cobros/devoluciones? ¿Hubo migración o datos reales usados?

## Sprint 4 — HU-08, HU-09 (fechas programadas: 16–20 nov 2026)

| Elemento | Evidencia y lectura |
|---|---|
| Entregable | Bitácora/finanzas y configuración de empresa/notas imprimibles. Commits: `a42fd55` (bitácora y resumen financiero local), `e6ca818` (empresa y notas de impresión), `d77be15` (MVVM y validación de notas), `24d7416` (confirmar guardado/descarte), `5fe8fff` (clasificación financiera). Jira relacionado HRL-27/28/29/48: [INFERIDO] desde clave de commits y trazabilidad; no acredita sprint real. | [EVIDENCIA] |
| Qué salió bien | Hay pruebas de notas de entrada/salida y protección contra imprimir secretos; logs 23-nov informan pruebas de UI aprobadas. | [INFERIDO] |
| Qué salió mal o se atrasó | `fix(empresa)` del 17-nov refleja ajuste del flujo de salida de pantalla; no hay reporte de fallo fechable del sprint. | [INFERIDO] |
| Ajuste para el siguiente sprint | Cierre amplía respaldos y prepara publicación del portal; ambos cambios aparecen como nuevas áreas en commits. | [INFERIDO] |
| Evidencia disponible | Commits citados; `tests/company-notes.test.ts`; `docs/evidencia/pruebas-2026-11-23.md`. | [EVIDENCIA] |

**Preguntas para el autor:** ¿En qué impresora/papel se validó físicamente? ¿Quién aprobó textos de notas? ¿Qué se entendió por cierre de este sprint?

## Cierre — HU-10, HU-11 (sin fechas de sprint localizadas)

| Elemento | Evidencia y lectura |
|---|---|
| Entregable | Respaldos portables y publicación de seguimiento. Commits: `c84b99e`, `9a4957e`, `8354650` (respaldos); `9226f73`, `1f472f1`, `566b8fb`, `26f5558`, `8945bb1`, `d807455` (contrato, cola, integración, fechas/QR). Ramas candidatas observables: `refactor/arquitectura` para respaldos; no se encontró rama portal en el listado actual. HRL-157/160 corresponde a respaldo/restauración en Jira, pero figura “En prueba”; relación de HU-10/11 con esa asignación: [INFERIDO] por alcance. | [EVIDENCIA] |
| Qué salió bien | El reporte del 23-nov registra prueba portable automatizada y tests de portal/aplicación; el código contiene cola normalizada y reconciliación por revisión/huella. | [INFERIDO] |
| Qué salió mal o se atrasó | Jira HRL-157/160 sigue En prueba; Jira y contrato dejan aceptación nativa y pruebas Firebase reales pendientes. Hay diferencias entre borrador del contrato y la proyección implementada. | [EVIDENCIA] del estado; [INFERIDO] como riesgo de cierre. |
| Ajuste siguiente | No hay sprint siguiente fechado en la evidencia consultada. Ajuste propuesto: cerrar pruebas de extremo a extremo, aceptación de respaldo y aprobación de contrato antes de afirmar entrega final. | [INFERIDO] |
| Evidencia disponible | Commits citados; `docs/evidencia/emisor-portal.md`; `docs/contrato/CONTRATO-FIRESTORE.md`; Jira HRL-157/160; `docs/evidencia/pruebas-2026-11-23.md`. | [EVIDENCIA] |

**Preguntas para el autor:** ¿Cuándo fue el periodo real de cada sprint? ¿Qué pruebas se ejecutaron el 19 y 22-nov? ¿Se probó Firebase real y se aprobó el contrato con el responsable del portal? ¿Qué criterio permitió cerrar estadía/entrega?

## Discrepancia de calendario

**[EVIDENCIA]** Hay commits con fecha 19-oct (`60f9aba`, `85fc1c4`, `5b0c420`), previos a las fechas programadas del Sprint 1 (26–30 oct). La cronología muestra otros bloques con commits del 27–29-oct y 13–19-nov, algunos posteriores a las ventanas programadas. El usuario advierte que se subieron cambios por bloques y tarde; por ello las fechas de Git no establecen la fecha real de trabajo ni permiten afirmar retraso del sprint.
