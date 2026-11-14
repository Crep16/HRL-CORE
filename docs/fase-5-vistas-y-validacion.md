# Fase 5 de HRL CORE

Entrega del 14 de noviembre de 2026. Antislop se aplicó durante los cambios por elección del usuario. La referencia visual fue la aplicación existente, con ENERGY 1, RHYTHM 1 y MOTION 1. Las decisiones de presentación se limitaron a la identidad del registro de auditoría, el foco de teclado y la legibilidad de importes.

## Estructura resultante

| Responsabilidad | Archivo |
| --- | --- |
| Acceso y configuración inicial | `src/views/AccessView.vue` |
| Recepción | `src/views/ReceptionView.vue` |
| Reparaciones | `src/views/RepairsView.vue` |
| Bitácora | `src/views/LogbookView.vue` |
| Finanzas | `src/views/FinancesView.vue` |
| Empresa | `src/views/CompanyView.vue` |
| Usuarios | `src/views/UsersView.vue` |
| Contenedor y navegación | `src/WorkshopApp.vue`, `src/composables/useWorkshop.ts` |
| Cámara y ciclo de vida | `src/composables/useEvidenceCamera.ts` |
| Editor de cláusulas | `src/composables/useCompanyClauses.ts` |
| Seguimiento, garantías y pagos | `src/composables/useRepairScreen.ts` |

Los componentes de cámara y cláusulas contienen contratos de props/eventos, plantilla y estilos. Los composables encapsulan el estado y sus transiciones, y reciben callbacks para notificar eventos a sus vistas. Esta composición conserva la separación entre presentación y operación de pantalla.

Bitácora muestra la entidad y el identificador afectados junto con el folio, y etiqueta correo y entrega estimada en sus diferencias. Los encabezados de las vistas admiten el foco de navegación. El editor devuelve el foco al aplicar, cancelar, mover, retirar o restaurar cláusulas. El importe de saldo pendiente conserva sus dígitos en una línea, con reflujo de su etiqueta dentro del panel lateral.

El árbol activo conserva una sola llamada `invoke`, en `services/api/transport.ts`. Las vistas acceden mediante composables y stores. La búsqueda estática del frontend activo quedó libre de `datos_json`, `operaciones_json`, `solicitud_json` y órdenes de demostración.

## Validación

La suite `test:phase5` contiene nueve casos de permiso de cámara rechazado, dispositivo ausente/ocupado, solicitud tardía, captura pendiente, limpieza al desmontar, cláusulas y foco, identidad de Bitácora, días de garantía y guardas de navegación.

La ejecución conjunta aprobó 38/38 pruebas frontend. `npm.cmd run build`, `cargo check --offline` y las 93 pruebas de `cargo test --offline` aprobaron. `npm.cmd run test:browser` también aprobó. Los servidores SSR de las pruebas desactivan WebSocket para permitir ejecución conjunta con el servidor de revisión. La prueba de rechazo de nota produce una advertencia Vue esperada, y las pruebas Rust históricas conservan sus 43 advertencias de funciones archivadas sin llamadas.

Las pruebas de cámara utilizan dispositivos y canvas simulados. La suite Rust conserva sus 93 casos con SQLite y DPAPI. La validación física de cámara, impresora e identidad Windows de destino corresponde a HRL-132 y HRL-141.

El navegador integrado y el runtime de Windows fallaron al inicializarse con `failed to write kernel assets`. La revisión de renderizado se completó con Chrome headless, un perfil temporal del proyecto y el fixture `tests/ui-restored.html`. Se observaron las pantallas de Clientes, Reparaciones, Empresa, Usuarios y Bitácora a 1280 × 900, con sus anchos dentro del viewport y capturas en `artifacts/phase5/`.

La prueba `tests/phase5-browser.mjs` recorre esas vistas, verifica el importe del panel, edita una cláusula, comprueba la guarda de borrador y abre/cierra cámara y galería con restitución del foco. El fixture utiliza IPC simulado. Para reproducirla, iniciar Vite en `127.0.0.1:1421`, iniciar Chrome headless con un perfil temporal y `--remote-debugging-port=9224`, y ejecutar `npm.cmd run test:browser`.

Durante la revisión se detectó el salto de línea dentro del importe del saldo y se corrigió su estilo. La comprobación de foco del navegador se ajustó para desplegar Evidencias antes de activar sus controles, siguiendo el recorrido visible.

## Recorrido funcional y límites

| Paso | Evidencia de esta entrega |
| --- | --- |
| Acceso | Pruebas existentes de sesión, configuración inicial, credenciales y renderizado de la vista trasladada |
| Clientes | Renderizado de recepción, consultas y borradores con IPC simulado, junto con regresión Rust de órdenes |
| Evidencias | Apertura/cierre de galería y cámara en navegador, ciclo de vida simulado y regresión Rust de archivos/metadatos |
| Reparaciones | Renderizado de la vista y selección de garantía sugerida en su viewmodel |
| Pagos | Regresión Rust y modelo financiero, más importe completo en el panel del navegador |
| Entrega | Regresión Rust existente de edición y combinación independiente de entrega y dinero |
| Nota de salida | Renderizado SSR del componente real con resumen público simulado, garantías y exclusión de secretos |

El recorrido completo con la aplicación instalada, dispositivos y personal del taller sigue pendiente. La integración del portal necesita el contrato acordado y las políticas de importación, formatos adicionales y portabilidad de respaldos conservan sus decisiones pendientes. Esta entrega conserva las reglas funcionales documentadas.
