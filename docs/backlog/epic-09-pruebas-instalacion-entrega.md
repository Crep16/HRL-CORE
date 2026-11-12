# HRL-77 [Epic] Pruebas, instalación y entrega

Fuente única de alcance: `docs/backlog-base (1).md`, leída el 12 de noviembre de 2026. Los códigos entre corchetes son parte del título y se conservan. HRL-64 se mueve desde el epic HRL-57; no se duplica ni cambia de sprint.

Estado: En progreso (evidencia: `tests/ipc-stores.test.ts`, `tests/phase4.test.ts`, `src-tauri/src/db/tests.rs` y `src-tauri/src/services/tests.rs` contienen pruebas reales; `src-tauri/tauri.conf.json::bundle` configura empaquetado). La existencia de suites, configuración o procedimientos no acredita instalación, ejecución actual, cámara, impresora ni pruebas con personal. La aceptación nativa permanece No verificada.
Prioridad: Alta, porque comprueba permisos, dinero, auditoría, recuperación y uso del taller antes de entregar.
Puntos: 36 (solo suma de los cinco ítems disponibles; los cinco faltantes no se estiman).
Sprint: 2, 3 y 4, según tarea.
Subtareas: agrupador de HRL-64 y HRL-78 a HRL-81. HRL-102 a HRL-104 pertenecen a HRL-64; HRL-127 a HRL-134 cubren los ítems añadidos.

Criterios de aceptación del epic (fuentes: `docs/evidencias.md`, `docs/auditoria-empresa.md`, `docs/ventana-por-sesion.md`, `docs/contexto-completo-proyecto.md` §6.4):

- Dado un conjunto de pruebas automatizadas, cuando se informa su validación, entonces se distingue código presente, ejecución real, mocks y procedimientos documentados.
- Dada la aplicación instalada con hardware y custodia válidos, cuando se realiza el recorrido nativo, entonces se registran resultados reales de cámara, impresión y recuperación sin atribuirlos a fixtures.
- Dado un error de permiso, auditoría, instalación o recuperación, cuando ocurre durante la validación, entonces se documenta el rechazo sin borrar pruebas ni declarar el flujo aprobado.

Los procedimientos y PASS históricos de documentos son antecedentes, no resultados ejecutados por esta revisión. No se realizaron builds, suites, instalación ni sesiones con personas. La fuente está incompleta; la nota final identifica los cinco ítems faltantes sin crear claves o tareas ficticias.

## HRL-64 [Tarea] Pruebas frontend y Rust, y validación nativa (cámara, impresora, DPAPI)

Estado: No verificado (evidencia de pruebas presentes: `tests/ipc-stores.test.ts`, `tests/phase4.test.ts`, `src-tauri/src/db/tests.rs`, `src-tauri/src/services/tests.rs::windows_key_custody_reopens_dpapi_key_without_storing_it_in_sqlite`, `src-tauri/src/windowing.rs::tests`). Se comprueba existencia de casos, no su resultado. Falta ejecutar suites y recorridos con cámara, impresora y DPAPI en Windows; está expresamente fuera de esta solicitud.
Prioridad: Alta, porque valida seguridad, dinero, auditoría y dependencias nativas antes del cierre.
Puntos: 13
Sprint: 4
División propuesta: HRL-102 (5), HRL-103 (5) y HRL-104 (3), sin sumar estos puntos de nuevo a los 13 del padre.
Subtareas:

- HRL-102 [Pruebas] Ejecutar y completar regresión frontend de sesión, borradores y reintentos; reparar en una tarea posterior la referencia `test:company` a `tests/company.test.mjs`, ausente del árbol revisado (5 puntos).
- HRL-103 [Pruebas] Ejecutar y completar cobertura Rust de migraciones/FK, permisos, concurrencia, libros inmutables y rollback de auditoría, reportando resultados del árbol exacto (5 puntos).
- HRL-104 [Pruebas] Validar en la aplicación Windows cámara concedida/denegada y liberación, impresora carta/ticket, geometría de ventana y reapertura DPAPI; registrar entorno y evidencia reproducible (3 puntos).

Criterios de aceptación (fuentes: límites de `docs/evidencias.md`, `docs/auditoria-empresa.md`, `docs/ventana-por-sesion.md`):

- Dado el árbol final, cuando se ejecutan sus suites, entonces se registra el resultado real y las pruebas de permiso denegado y rollback sin retirar pruebas para forzar éxito.
- Dado hardware Windows, cuando se captura, imprime y reabre custodia, entonces se documenta lo observado separándolo de mocks y comprobaciones de compilación.
- Dada cámara denegada, impresora no disponible o fallo DPAPI, cuando se produce, entonces el flujo presenta error y no afirma éxito ni pierde silenciosamente datos.
- Dada una comprobación que no pudo realizarse, cuando se entrega el resultado, entonces figura No verificado con entorno y condición faltantes.

Discrepancia D-15: documentos antiguos citan suites `.mjs` que aparecen eliminadas o ausentes en el árbol actual; `package.json::scripts.test:company` todavía referencia `tests/company.test.mjs`. Los PASS históricos no prueban el estado actual. `docs/ventana-por-sesion.md` además reporta un fallo DPAPI previo; no se afirma que persista ni que esté resuelto sin ejecutarlo.

## HRL-78 [Tarea] [S206] Probar edición, evidencias y permisos

Estado: En progreso (evidencia: `src-tauri/src/services/tests.rs::restored_edit_trash_finances_and_refund_preserve_original_books`, `restored_evidence_and_company_reopen_without_business_json` y `reception_history_requires_logbook_permission_and_list_rechecks_active_users`; `tests/ipc-stores.test.ts` incluye el caso `denied persistent queries clear cached data and changed orders reject stale history`). Cubren servicios reales con SQLite y pruebas de IPC simulado; no se localizó en tests una suite integral que reúna todo el recorrido de edición, evidencia y denegaciones del frontend. No se ejecutaron esos casos.
Prioridad: Alta, porque verifica permisos y conservación del expediente sin escrituras indebidas.
Puntos: 5
Sprint: 2 (por código S2xx).
Subtareas:

- HRL-127 [Pruebas] Completar una matriz de edición, revisión antigua, consulta de evidencia, descripción, retiro/restauración y denegaciones sobre los contratos normalizados, identificando casos ya existentes y huecos sin duplicarlos.
- HRL-128 [Pruebas] Preparar y registrar la ejecución integrada de la matriz frontend/Rust sobre datos aislados. La cámara física pertenece a HRL-64; no acreditar ese recorrido con mocks.

Criterios de aceptación (fuentes: `docs/papelera-ordenes.md`, `docs/evidencias.md`, `docs/acceso-temporal.md`):

- Dada una orden y revisión vigente con permiso de editar, cuando se prueba guardar, entonces persiste el cambio y su auditoría sin perder el expediente.
- Dada una revisión antigua o un usuario sin permiso, cuando se prueba modificar ficha o evidencia, entonces se rechaza y se conservan los datos confirmados.
- Dada una fotografía retirada y permiso de borrar, cuando se prueba restaurarla, entonces reaparece conservando archivo, autor y fecha originales.
- Dado préstamo de alcance distinto o autorizador revocado, cuando se prueba la acción, entonces Rust la deniega aunque el control hubiera estado habilitado.

## HRL-79 [Tarea] [S307] Probar notas, garantías y totales

Estado: En progreso (evidencia: `src-tauri/src/services/tests.rs::restored_repair_uses_relational_records_and_rejects_stale_changes` registra garantías y refacciones; `src-tauri/src/db/tests.rs::balances_keep_receipts_price_reductions_adjustments_and_total_refunds_separate` comprueba saldos; `src-tauri/src/finances_tests.rs::all_four_payment_delivery_combinations_use_the_ledger_and_direct_costs` cubre la ruta anterior). `tests/ui-restored.ts` configura un resumen público de prueba, pero no es una suite de aserciones de impresión; el script `test:company` apunta a un archivo ausente. Falta acreditar cobertura integrada de las notas actuales y su ejecución. No se usan pruebas legacy para dar por validada la ruta 3FN.
Prioridad: Alta, porque verifica importes y evita exponer notas internas en documentos del cliente.
Puntos: 5
Sprint: 3 (por código S3xx).
Subtareas:

- HRL-129 [Pruebas] Completar aserciones del resumen público y documento de entrada/salida normalizados: nota pública, garantías, exclusión de notas internas/contraseñas y errores de preparación; distinguir fixtures de pruebas ejecutables.
- HRL-130 [Pruebas] Integrar los casos de totales, refacciones, cobros, ajustes, devoluciones y separación de monedas con garantías y notas del flujo actual, registrando después la ejecución real. La impresión física queda en HRL-64.

Criterios de aceptación (fuentes: `docs/reparaciones-permisos.md`, `docs/reglas-pagos.md`, `docs/bitacora.md`, `docs/auditoria-empresa.md`):

- Dado seguimiento con nota pública, nota interna y garantías, cuando se prueba la nota de salida, entonces incluye únicamente el resumen público, técnico y cobertura/días autorizados y excluye secretos.
- Dados cobros originales, ajustes y devoluciones, cuando se comprueban totales, entonces neto, pendiente y saldo a favor siguen el libro y se separan por moneda.
- Dado fallo de consulta del resumen o permiso insuficiente, cuando se prepara la nota, entonces no se emite un documento con información inventada o incompleta.
- Dada garantía registrada, cuando se presenta en la nota, entonces su inicio se relaciona con la entrega y no se inventa una fecha independiente.

## HRL-80 [Tarea] [S402] Preparar instalación y recuperación local

Estado: En progreso (evidencia: `src-tauri/tauri.conf.json::bundle` habilita empaquetado; `src-tauri/src/db/connection.rs::Database::open_file` define DB/assets; `src-tauri/src/services/key_store.rs::OsKeyStore::open` custodia DPAPI; `src-tauri/src/schema_backup.rs::before_spanish_migration` respalda únicamente una base legacy antes de la 013). Se localizaron binarios de desarrollo, pero no un instalador MSI o setup dentro de target en la búsqueda realizada. Hay instrucciones de respaldo parcial en `docs/evidencias.md` y advertencias de custodia en contexto completo §6.4; falta procedimiento integral y resultado de instalación/restauración. La instalación y recuperación nativas son No verificadas por lectura.
Prioridad: Alta, porque una instalación o recuperación incompleta puede perder fotografías o acceso a credenciales cifradas.
Puntos: 8
Sprint: 4 (por código S4xx).
Subtareas:

- HRL-131 [Servicio Rust] Documentar el procedimiento de instalación, primer inicio y recuperación local con rutas reales de DB, assets y custodia, apoyado en la estrategia de HRL-62. No implementar aquí otra migración automática ni duplicar HRL-70.
- HRL-132 [Pruebas] Preparar y, en una validación posterior autorizada, registrar instalación/reapertura y recuperación conjunta en entorno Windows conocido, incluyendo rechazo de respaldo incompleto y custodia inválida. No inferir portabilidad DPAPI.

Criterios de aceptación (fuentes: contexto completo §6.4, `docs/acceso-validacion.md`, `docs/evidencias.md`):

- Dada una instalación nueva, cuando se realiza el primer inicio validado, entonces configura administrador y clave maestra atómicamente sin cuentas predeterminadas.
- Dado respaldo operativo, cuando se verifica recuperación, entonces contempla base, evidencias, logo y custodia de claves; copiar solo DB no se declara suficiente.
- Dada una clave no recuperable o recursos faltantes, cuando se intenta restaurar, entonces informa el fallo y no afirma recuperación exitosa.
- Dado un restablecimiento Ed25519 de claves de acceso, cuando se documenta recuperación local, entonces no se confunde con restauración o traslado de la clave AES.

## HRL-81 [Tarea] [S403] Realizar pruebas con personal del taller

Estado: No verificado (evidencia de límites: `tests/ui-restored.ts` usa fixtures y `mockIPC`; `tests/phase4.test.ts` utiliza un renderizador de prueba; `src-tauri/src/windowing.rs::tests` usa ventana simulada). Esos archivos no acreditan participación de personas del taller. No se localizó en docs un acta identificable de aceptación por personal para esta versión. Falta recorrido acordado, versión instalada, participantes/roles autorizados y resultados observados; no se inventan personas, respuestas ni aprobación.
Prioridad: Alta, porque valida uso real de flujos de dinero, permisos y custodia antes de entregar.
Puntos: 5
Sprint: 4 (por código S4xx).
Subtareas:

- HRL-133 [Pruebas] Acordar con el responsable del taller un recorrido y criterios de aceptación sobre cuentas y datos de prueba autorizados, cubriendo recepción, reparación, dinero y consulta de auditoría sin seleccionar participantes por suposición.
- HRL-134 [Pruebas] Registrar versión, entorno, roles participantes, resultados, incidencias y decisión de aceptación real; mantener pendientes los recorridos no realizados y derivar fallos a trabajo identificado.

Criterios de aceptación (fuentes: tarea HRL-81, contexto funcional §3, `docs/reglas-pagos.md`, `docs/accesos.md`, `docs/bitacora.md`):

- Dado personal autorizado y un recorrido acordado, cuando realiza las pruebas con la versión identificada, entonces existe registro de acciones observadas y resultados que distingue prueba real de simulación.
- Dado un rol sin permiso para cobrar, editar o consultar Bitácora, cuando intenta esas acciones durante el recorrido, entonces se deniegan conforme a las facultades de su cuenta.
- Dada una orden de prueba con trabajo, dinero y entrega, cuando se recorre el flujo, entonces se verifica que completar una etapa, cobrar e imprimir no registran entrega automáticamente.
- Dado un caso no realizado o una incidencia sin resolver, cuando se presenta la aceptación, entonces queda explícitamente No verificado o rechazado y no se atribuye aprobación al personal.

Discrepancia D-19: existen pruebas automatizadas y documentos con PASS históricos, pero eso no demuestra ejecución sobre el árbol actual, instalación nativa ni aceptación de personas. `docs/restauracion-diseno.md` reporta 87 pruebas Rust aprobadas con DPAPI; `docs/ventana-por-sesion.md` registra un fallo DPAPI anterior. Esta revisión no ejecutó ninguno de los dos escenarios y no deduce un resultado actual por elegir un reporte histórico.

Nota de alcance incompleto: **faltan 5 ítems por incorporar a este epic**. La fuente indica que no se alcanzan a ver en la captura de Jira. No se crean claves, títulos, subtareas ni estimaciones para esos cinco ítems hasta recibirlos.

