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

## HRLP-67 · Video y documentos como evidencia

Referencia original: HRL-67, epic HRL-9; sin cambio de título, epic ni sprint. Primera versión implementada: documentos PDF y videos MP4, además de JPG/PNG/WebP, con máximo de 20 MB por archivo. No se amplía a formatos de Office, AVI o MOV ni se añade captura de video de cámara. No hay conexión del portal.

La validación pura vive en `domain/evidence_contracts.rs::validate/evidence_format/valid_mp4`; comprueba MIME, tamaño, origen y cabeceras/estructura básica. El servicio `EvidenceService::relative/add/read/update` conserva permisos, autorización por orden, reintentos, pertenencia de la referencia y ruta controlada. `EvidenceRepository::insert` persiste `document` o `video` en la tabla existente. Los binarios siguen fuera de SQLite; no cambia el esquema.

`useOrderEvidence` conserva el flujo de carga y consulta mediante el store/API. `OrderEvidence.vue` presenta PDF como copia descargable y video con controles, sin reproducción automática; si un códec no se reproduce, explica cómo guardar una copia. Las miniaturas PDF/MP4 muestran su tipo sin intentar decodificarlas como imagen. Cambiar o cerrar evidencia pausa el video y revoca el Blob URL. Las teclas del reproductor mantienen su función y las flechas de navegación solo actúan cuando el contenedor tiene el foco. Se mantienen nombres de contratos internos `Photo` para compatibilidad; ahora transportan los cinco MIME admitidos.

Pruebas nuevas: `attachments_validate_content_size_and_camera_origin` cubre contenido falso, PDF incompleto, MP4 truncado, cajas fuera del archivo, tamaño y origen cámara. `video_and_pdf_evidence_share_permissions_retry_restore_and_file_persistence` usa servicios y SQLite reales para ambos formatos: duplicado idempotente, contenido, tipo persistido, lectura/agregado de Limitado, retiro denegado, descripción, retiro/restauración de Completo, lectura/listado de retiradas denegados a Limitado, identidad/metadatos conservados y reapertura del archivo de base con recuperación de bytes.

Validación: `cargo check --offline --quiet`, `cargo test --offline --quiet` (105/105 con perfil real de Windows para DPAPI), `npm run test:phase5` (10/10), `npm run test:ui` (14/14), `npm run test:ipc` (14/14), `npm run build` y `npm run test:browser`. El navegador carga archivos mediante el input real, guarda una copia PDF, reproduce y pausa un MP4 generado en la prueba, retira/restaura ambos tipos y revisa 375/760/1280 px. Capturas `artifacts/phase5/video-evidence-375.png`, `video-evidence-760.png` y `video-evidence-1280.png`, revisadas visualmente. Se conservan colores y tipografía; los controles de video son nativos y los botones tienen al menos 44 px. No hay nuevos activos decorativos ni animación (energía/ritmo/movimiento: 1). La revisión de interfaz y accesibilidad pasó para el flujo añadido.

Limitaciones: la validación de archivos es básica, no un decodificador completo ni antivirus; la reproducción depende del códec disponible. El recorrido de navegador usa IPC simulado y no acredita WebView instalado, impresora/cámara física o cambio de perfil de Windows (HRLP-63). La persistencia real y los permisos sí están cubiertos por la prueba Rust. Los backlogs originales y la auditoría estática permanecen intactos.

Referencias técnicas: el registro de [marcas MP4](https://mp4ra.org/registered-types/brands) describe los identificadores de formato; el [elemento video](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/video) ofrece controles de reproducción, y el [atributo download](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/a#download) permite guardar una copia de un recurso Blob. Estas capacidades web no sustituyen la validación nativa pendiente.

## HRLP-10 · Criterio de migraciones actualizado

Referencia original: HRL-10, epic HRL-1. El usuario aprobó expresamente reemplazar el criterio de «solo 001 y 002» por «crea hrl_core_3fn.db y aplica en orden las migraciones registradas del esquema actual, sin ejecutar legacy». Se actualizan Jira y `docs/arquitectura-objetivo.md`; los dos archivos originales del backlog y los tres informes de auditoría permanecen intactos.

Resultado: Hecho conforme al criterio actualizado. El cargador registra actualmente 001–005, compila el DDL desde migraciones y rechaza bases anteriores. Las pruebas `src-tauri/src/db/tests.rs` cubren aplicación reentrante, cinco versiones registradas, FK por conexión, integridad/reapertura, rechazo de esquema anterior y checksum alterado, restricciones de los libros/auditoría y actualización 004→005. La suite completa de 105 pruebas Rust y `cargo check` acababan de pasar, al igual que build y regresión frontend/navegador. Este paso solo actualiza documentación; no modifica migraciones ni código.
