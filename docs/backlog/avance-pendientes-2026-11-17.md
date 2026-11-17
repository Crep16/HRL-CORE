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

## HRLP-30 · Aclaración de evidencias retiradas

Referencia original: HRL-30, epic HRL-3. El usuario confirmó el 17 de noviembre de 2026 que el criterio de visibilidad sin Borrar se refiere a **evidencias retiradas**. No se modifica la regla de consultar fotos activas con Visualizar ni los archivos originales del backlog. La ambigüedad de la auditoría queda resuelta por esta confirmación.

`src-tauri/src/services/evidence.rs::EvidenceService::list/read` exige `evidence.delete` para listar o leer retiradas; `update` exige esa facultad para retirar/restaurar, verifica revisión y rechaza descripciones mayores de 500 caracteres. No borra el archivo al retirar. La interfaz `useOrderEvidence.ts` conserva el control por permisos.

Se amplió `src-tauri/src/services/tests.rs::integrated_evidence_revision_restore_and_revoked_authorization_preserve_original`: un usuario con solo Visualizar no puede listar retiradas ni leer directamente sus bytes; tampoco aparecen en su lista activa. Tras restaurar con el administrador se conservan bytes y metadatos, y la consulta de una activa con Visualizar sigue permitida. También se rechaza una descripción de 501 caracteres Unicode. No cambió código de producción.

Resultado: Hecho bajo la aclaración autorizada. `cargo check --offline --quiet` pasó y la suite completa `cargo test --offline --quiet` pasó 100/100 con el perfil real de Windows (incluidas DPAPI). La regresión de navegador del paso anterior ya cubre descripción, retiro y restauración con IPC simulado. Build vigente sin nuevos cambios de frontend. Persisten advertencias históricas de código no utilizado.

## HRLP-55 · Propuesta del contrato del portal

El usuario confirmó que no existe contrato y solicitó una propuesta para que los datos enviados sean visibles en la web. Se creó `docs/backlog/contrato-portal-propuesta-v1.md`, con flujo app/cola/API/Firebase/web, campos permitidos y prohibidos, enlace privado, autenticación separada de la local, revisiones, idempotencia, despublicación, trabajo sin conexión y plan por HRLP-55–59. Incluye fuentes oficiales de Firebase y distingue capacidades propuestas de código existente.

Resultado: En progreso; aprobación del responsable No verificada. No se implementó ni desplegó sincronización y no se crearon recursos remotos. HRLP-56–59 continúan pendientes de contrato aprobado e integración. La decisión de un único emisor, política de acceso, vencimiento y alcance público se ofrece para revisión; no se impone como regla vigente. Los dos archivos originales del backlog y los tres informes originales de auditoría permanecen sin modificaciones.

Commits de implementación previos: HRLP-62 `760266c`, HRLP-48 `24d7416`, HRLP-28 `60feef2` y HRLP-30 `2156f61`.
