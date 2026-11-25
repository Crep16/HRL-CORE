# Conteo y reportes de pruebas

## Estado actual consultado

**[EVIDENCIA — `cargo test -- --list`, ejecutado desde `src-tauri`]** La salida listó **131 pruebas Rust**, cero benchmarks; `src-tauri/src/main.rs` y los doctests reportaron cero pruebas adicionales. Este comando solo lista; no mide cuántas pasan en esta ejecución.

**[EVIDENCIA — `docs/evidencia/cargo-test-2026-11-23.txt`, salida de `cargo test`]** El reporte del 23-11-2026 registra `131 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`, código de salida 0.

## Reportes del 19, 22 y 23 de noviembre

| Fecha | Evidencia localizada | Resultado comprobable |
|---|---|---|
| 2026-11-19 | **NO ENCONTRADO EN EL REPOSITORIO** un log de ejecución con esa fecha. Hay commits del día, pero no equivalen a reportes de pruebas. | NO ENCONTRADO EN EL REPOSITORIO |
| 2026-11-22 | **NO ENCONTRADO EN EL REPOSITORIO** un log o informe de pruebas con esa fecha. | NO ENCONTRADO EN EL REPOSITORIO |
| 2026-11-23 | `docs/evidencia/pruebas-2026-11-23.md`, `cargo-test-2026-11-23.txt`, `ui-tests-2026-11-23.txt`, `vite-server-2026-11-23.txt` | Rust 131/131; interfaz Node 71/71; suite de navegador pasó tras preparar Vite y Chrome CDP, sin total numérico de casos. Primer intento de navegador falló por `ECONNREFUSED 127.0.0.1:9224`, luego reintento aprobado. |

## Discrepancia 126 + 4 = 130 frente a 131

**[EVIDENCIA]** La suma aritmética de 126 aprobadas y 4 fallidas es 130. En el árbol consultado, la salida completa guardada del 23 de noviembre registra 131 pruebas ejecutadas: 131 aprobadas y cero fallidas. La consulta actual `cargo test -- --list` también enumera 131.

**[EVIDENCIA]** Los archivos examinados no contienen el reporte que sustente 126 + 4 ni una prueba duplicada identificable como explicación.

**Conclusión limitada:** para el estado y la ejecución documentados el 23-11-2026, la cifra respaldada es **131 aprobadas, 0 fallidas**. No se puede explicar de dónde salieron las cifras 126 y 4: **NO ENCONTRADO EN EL REPOSITORIO**. No afirmar que la diferencia sea una prueba repetida o agregada después sin el log de origen.

## HRL-64 y restauración

**[EVIDENCIA — consulta Jira `getJiraIssue(HRL-64)`]** HRL-64 figura como “[009] Perfil de ventana según estado (acceso, configuración inicial, taller)”, estado “Terminado”. El texto de Jira dice que la validación nativa y pruebas con personal no están verificadas.

**[EVIDENCIA — Jira `HRL-157`]** El ticket “[062] Respaldo y migración automática entre instalaciones” figura “En prueba”; su subtarea HRL-160, de pruebas conjuntas de restauración, también “En prueba”. La descripción de HRL-157 registra implementación y prueba automatizada portable el 14 de noviembre, pero deja pendiente aceptación nativa y restauración bajo otra identidad de Windows u otra PC.

**[EVIDENCIA — Jira `HRL-75`]** Es “[019] Papelera: eliminar y restaurar órdenes conservando historial”, no restauración de respaldos.

Por tanto, “HRL-64 falló el 22 y se aprobó el 23” **NO ENCONTRADO EN EL REPOSITORIO ni confirmado en Jira**. Parece haber una confusión de ticket/tema; hace falta el enlace o clave correcta y el reporte del día 22.
