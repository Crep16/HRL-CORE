# Implementación posterior a la auditoría

Los tres informes de auditoría se conservan como fotografía de la revisión estática. Este registro documenta cambios posteriores autorizados; no modifica el backlog original ni sus claves, epics o sprints.

## HRLP-62 · Pruebas de bitácora, finanzas y empresa

Referencia original: HRL-62. Resultado: Hecho.

- `src-tauri/src/services/tests.rs::normalized_finances_classify_payment_and_delivery_independently_in_each_currency` crea ocho órdenes mediante los servicios normalizados, registra cobros y entrega cuando corresponde. Comprueba las cuatro clasificaciones (`completed`, `paid_uncollected`, `delivered_unpaid`, `pending`) en MXN y USD, junto con saldo, total, moneda, pago y entrega independientes del estado técnico.
- `tests/finances-model.test.ts` comprueba el filtro individual de las cuatro clases, las tres clases incompletas y que solo las completadas aportan a la utilidad completada.
- La prueba vigente de nota de salida en `tests/company-notes.test.ts` confirma que no se imprimen contraseña ni nota interna.

Validación ejecutada: `npm run test:finances` (4/4), `npm run test:company` (3/3), `npm run build`, `cargo check --offline` y `cargo test --offline --quiet` (100/100). La suite Rust completa pasó con el perfil real de Windows fuera del aislamiento. Dentro del aislamiento pasaron 97 pruebas y fallaron tres de DPAPI con `Crypto`; se repitió la suite completa fuera del aislamiento sin deshabilitar ni modificar esas pruebas. Persisten advertencias de código histórico no utilizado.

No cambian reglas de negocio ni código de producción en este paso.

## HRLP-48 · Configuración de empresa y guarda de borrador

Referencia original: HRL-48, epic HRL-6. Resultado: Hecho.

Empresa ofrece «Guardar y salir», «Descartar y salir» y «Seguir editando» al abandonar un borrador mediante navegación o cierre de sesión. El guardado debe confirmarse antes de salir: una validación fallida o un rechazo conserva el borrador. Bloquea la salida durante guardado, lectura del logo, impresión o edición de una cláusula sin aplicar/cancelar.

Evidencia: `src/composables/useDraftLeaveGuard.ts`, `useCompanyWorkspace.ts::prepareLeave/save`, `useWorkshop.ts::prepareLeave`, `CompanyView.vue` y `src/components/ui/DraftLeaveDialog.vue`. La revisión optimista y la auditoría transaccional permanecen en los servicios/repositorios normalizados existentes.

La guarda encapsula la elección pendiente y compone callbacks; no introduce herencia ni reglas de negocio en la vista. El diálogo contiene presentación, eventos y manejo del foco. Conserva fuentes y colores del taller, sin animación adicional (energía, ritmo y movimiento: 1). En ventanas estrechas apila acciones; usa botones de al menos 44 px, Escape para seguir editando, recorrido Tab dentro del diálogo y restauración del foco. Se aplicaron antislop-ui, antislop-human, antislop-layoutmobile y la guía estática de UI/UX Pro Max; Python/py no estuvo disponible por denegación de acceso de Windows.

Validación: fase 5 10/10, IPC 14/14, notas de empresa 3/3, build de TypeScript/Vite y `npm run test:browser`. Se verificaron también los tres caminos de salida, guardado rechazado y confirmado, cierre de sesión, Escape, Tab y foco en Chrome aislado con IPC simulado, a 360/540/768/1280 px. Capturas en `artifacts/draft-guards/`. Las comprobaciones Rust de la sección anterior siguen vigentes; este paso no modifica backend. La prueba del diálogo no sustituye las validaciones físicas de los tickets No verificado.

## HRLP-28 · Guardas de borrador

Referencia original: HRL-28, epic HRL-3. Resultado: Hecho.

Recepción integra la guarda compartida del commit `24d7416`. Navegación y cierre de sesión esperan la elección guardar/descartar/seguir editando; los fallos conservan la captura. Una solicitud financiera incierta no se puede descartar, incluso si el error ocurre después de abrir el diálogo. Se conserva el identificador y el payload para reintentar. No cambia el guardado automático de Reparaciones ni su bloqueo ante fallo.

Evidencia: `src/composables/useReceptionForm.ts::prepareLeave`, `src/views/ReceptionView.vue`, `src/composables/useWorkshop.ts::prepareLeave/navigate/logout` y las pruebas nuevas en `tests/phase4.test.ts`, `tests/backups.test.ts` y `tests/draft-guards-browser.mjs`.

Validación: UI 14/14, respaldos 6/6, pagos 4/4, finanzas 4/4, fase 5 10/10, IPC 14/14, notas 3/3 y build. Las pruebas verifican una única solicitud pendiente, acciones bloqueadas mientras guarda, rechazo sin salida, cancelación al desmontar, los tres caminos de Recepción y la prohibición de descartar un cobro incierto. La prueba de cierre confirma que no hay respaldo ni cierre hasta una elección aceptada; un guardado rechazado mantiene la ventana abierta. `node tests/draft-guards-browser.mjs` verifica también guardado y descarte de Recepción, Empresa, navegación, cierre de sesión y teclado en los cuatro anchos. La regresión `npm run test:browser` pasó con la nueva elección en Empresa.

La simulación de IPC y el foco de un modal bajo un contenedor inerte no sustituyen la ejecución nativa del instalador, cámara, impresora o cambio de perfil de Windows.
