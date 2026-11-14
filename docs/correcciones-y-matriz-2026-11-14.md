# Correcciones y matriz de validación del 14 de noviembre de 2026

## Abonos

Se reprodujo en el navegador el formulario abierto que no confirmaba el abono.
`v-model` sobre un input `type="number"` entregaba un número; `submit()` intentaba
ejecutar `.trim()` sobre él y lanzaba una excepción antes del IPC.

El importe ahora se captura como texto con teclado decimal y se convierte a
centavos mediante `parseMinorUnits`, sin multiplicar números flotantes. Admite
punto y coma decimal, informa importes inválidos y conserva las reglas del
servicio Rust. Las pruebas cubren el valor numérico anterior, punto, coma,
rechazos y reintentos con identificador y contenido idénticos.

## Antecedentes

La decisión del usuario sustituye las preguntas de otro taller y caída/humedad
por un solo campo libre. La migración nueva 003 añade texto nullable conservando
los datos anteriores. La ficha formatea esos datos cuando aún no hay nota nueva.
El usuario puede escribir o borrar la nota; una cadena vacía no vuelve a mostrar
los antecedentes antiguos. Véase [regla vigente](antecedentes-equipo.md).

## Matriz para HRL-176, HRL-177 y HRL-179

| Recorrido | Evidencia ejecutable | Resultado |
| --- | --- | --- |
| Edición con permiso, revisión antigua y auditoría | `services/tests.rs`: `restored_edit_trash_finances_and_refund_preserve_original_books`, `order_optional_changes_keep_old_and_new_values_and_audit_identity_without_secrets` | PASS, SQLite real aislado |
| Antecedentes existentes, nota libre, borrado y rollback | `equipment_history_text_preserves_old_fields_audits_changes_and_rolls_back`; `tests/phase4.test.ts`; navegador | PASS |
| Evidencia: consulta, descripción, revisión antigua, retiro, restauración y original | `integrated_evidence_revision_restore_and_revoked_authorization_preserve_original`; navegador | PASS |
| Evidencia: edición denegada, préstamo de otro alcance y autorizador revocado | Mismo caso Rust integrado | PASS, sin modificar el original |
| Respuestas de sesión antigua y datos denegados | `tests/ipc-stores.test.ts` | PASS |
| Confirmar abono con teclado, punto/coma y mensaje de rechazo | `tests/payments-form.test.ts`; `tests/phase5-browser.mjs` | PASS |
| Resultado incierto y rechazo definitivo del abono | `tests/payments-form.test.ts`; `restored_ledger_retries_exact_request_and_rolls_back_audit_failure` | PASS |
| Notas públicas/internas, garantías y refacciones junto con dinero | `integrated_notes_warranties_parts_receipts_adjustments_refunds_and_currencies` | PASS |
| Cobros, ajustes, reducción de precio, devolución total y separación MXN/USD | Mismo caso integrado; `tests/finances-model.test.ts` | PASS |
| Nota de salida: resumen público, garantías, exclusión de secretos y error de preparación | `tests/company-notes.test.ts` | PASS, renderizado real y contrato IPC simulado |

El navegador monta los componentes reales con un transporte IPC simulado. Los
casos Rust ejecutan servicios, repositorios y SQLite en bases aisladas; el caso
de evidencias usa archivos temporales reales. Son verificaciones complementarias,
no una sesión de aceptación del taller ni una prueba física de cámara/impresión.

## Ejecución

- `cargo check --offline`: PASS.
- `cargo test --offline`: 96/96 PASS con el perfil Windows real, incluidas las pruebas DPAPI.
- `npm.cmd run build`: PASS, TypeScript y Vite.
- `node --experimental-strip-types --test tests/*.test.ts`: 43/43 PASS.
- `npm.cmd run test:browser`: PASS, sin excepciones; flujo de abonos, antecedentes y galería.
- Navegador: cinco vistas a 1280 px y recepción a 760/375 px sin desbordamiento horizontal.
- `git diff --check`: PASS.

Registros locales: `artifacts/validacion-rust-2026-11-14.log`,
`artifacts/validacion-frontend-2026-11-14.log`. Capturas revisadas:
`artifacts/phase5/history-unified.png`, `payment-confirmed.png` y
`evidence-restored.png`. Persisten 43 advertencias de funciones históricas sin
uso en la compilación de pruebas; el código antiguo sigue excluido de producción.

## Limpieza

Se retiraron los comandos de exportación Figma que apuntaban a scripts ausentes,
la configuración del plugin SQL retirado, el estilo del grupo de preguntas
eliminado y `extract-reference.mjs`, que leía una ruta personal de referencia.
El README describe ahora el sistema 3FN y los comandos existentes.

Se eliminaron cinco resultados temporales de `artifacts`: los antiguos
`hrl-core-espanol.exe`, `.pdb`, `.d`, y los scripts provisionales
`phase5-browser.mjs` y `phase5-focus-debug.mjs`. Se recuperaron 182,62 MiB.
Las pruebas vigentes están en `tests/`. Se conservan los archivos históricos que
sirven para regresión, las migraciones aplicadas, referencias de diseño y datos.

## Antislop durante los cambios

Lectura de diseño: formularios operativos de un taller, conservando la identidad
HRL, energía 1, ritmo 2 y movimiento 1. El foco sigue siendo capturar la orden y
confirmar el dinero. Un campo de antecedentes evita selecciones redundantes;
su etiqueta y ayuda expresan qué escribir. Conserva tipografía, colores y
espaciado del formulario existente; no añade decoración ni recursos visuales.

- Hard Gate PASS: sin texto promocional, cifras inventadas, recursos nuevos o destinos añadidos. Los campos tienen etiqueta, lectura y edición; el abono informa rechazo y confirmación.
- R-03 PASS: comprobación de ancho a 1280/760/375 px y captura del campo unificado.
- R-25 PASS: contraste calculado de texto `#12222d`/blanco 16,25:1, ayuda `#55656f`/blanco 6,04:1, acento `#1b72ad`/blanco 5,18:1 y foco `#0f4c78`/blanco 9,03:1.
- R-26/R-27 PASS: se ejecutaron confirmar, rechazo, guardar antecedentes, guardar descripción, retirar y restaurar; existen estados de carga, vacío y error.
- R-32/R-35 PASS: confirmación del abono con Enter, foco restaurado en cámara/galería y recorridos reales de componentes en navegador sin excepciones. IPC simulado identificado expresamente.
- R-33/R-34 PASS: cambios escritos en fuente; se conserva el tema existente y no se añade selector.
- Purpose-Gate PASS: sin nuevos iconos, fondos, sombras, fuentes, tarjetas o animaciones; la simplificación responde a la captura solicitada.
- Liveliness PASS: valores 1/2/1 acordes con formularios de operación; jerarquía de secciones, espacio estructural, acento de acción y motivos HRL conservados en capturas.
- Craftsmanship y Quality Locks PASS: etiqueta y ayuda específicas, controles operativos, estados comprobados, paleta y composición existentes; no se fabrica contenido de negocio ni se rediseña la aplicación.

## Límites y continuación

No se generó ni actualizó instalador. HRL-141 y HRL-181 conservan pendientes las
validaciones nativas e instalación; HRL-38 necesita participantes reales. La
publicación requiere contrato del portal; importar datos históricos requiere
origen y conflictos confirmados; video/documentos y respaldo entre instalaciones
requieren políticas acordadas. Estas tareas no se declaran terminadas por usar
mocks ni por las suites de esta entrega.
