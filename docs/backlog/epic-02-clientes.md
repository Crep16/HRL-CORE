# HRL-11 [Epic] Módulo de Registro de clientes y recepción de órdenes

Fuente de alcance vigente: `docs/backlog-base (1).md` (80 ítems, sin HRL-66). Actualización documental del 12 de noviembre de 2026. Los estados de ítems sin cambio de alcance conservan la evidencia de la revisión estática previa; no se ejecutaron pruebas.

Estado: Hecho por inspección estática de las historias. Cámara, impresora y persistencia nativa no se ejecutaron; su validación transversal está en HRL-64.
Prioridad: Alta, porque crea el expediente del que dependen seguimiento, dinero y auditoría.
Puntos: 58 (suma de historias, sin sumar otra vez el epic).
Sprint: 2, 3 y 4, conservando el reparto original.
Subtareas: agrupador de HRL-12 a HRL-23. No se añaden subtareas para funciones ya localizadas completas.

Criterios de aceptación del epic (fuentes: contexto funcional §5.2, `docs/evidencias.md`, `docs/papelera-ordenes.md`):

- Dada una recepción válida y autorizada, cuando guarda, entonces obtiene folio único, cliente/equipo relacionados y auditoría sin exponer la contraseña del equipo.
- Dada una orden con fotografías y movimientos, cuando se envía a papelera y se restaura, entonces conserva expediente, archivos, dinero e historial.
- Dado permiso insuficiente, revisión antigua o borrador sin resolver, cuando intenta cambiar el expediente, entonces se rechaza o bloquea la salida sin perder datos confirmados.

Convenciones de estado y estimación: véase [usuarios](epic-01-usuarios.md). Fuentes funcionales de recepción: `docs/contexto-funcional-vistas-y-estructura.md` §§2, 3 y 5.2; para revisión y MVVM, `docs/arquitectura-objetivo.md`. Se conservan reglas funcionales aunque las rutas técnicas citadas por esos documentos sean anteriores.

## HRL-12 [Historia] Crear orden con cliente, equipo, falla y trabajo solicitado (folio asignado por backend)

Estado: Hecho (evidencia: `src-tauri/src/services/orders.rs::OrderService::create` y `validate`, `src-tauri/src/repositories/orders.rs::OrderRepository::reserve_folio`, `src/composables/useReceptionForm.ts::submit`). Reserva el folio dentro de la transacción y valida pertenencia del equipo al cliente.
Prioridad: Alta, porque habilita todos los registros vinculados a una orden.
Puntos: 8
Sprint: 2
Subtareas: ninguna faltante detectada. Pruebas presentes, no ejecutadas: `src-tauri/src/services/tests.rs::concurrent_receipts_use_distinct_folios_and_audit_authorship` y `failed_order_or_payment_audit_rolls_back_all_rows_and_the_folio_reservation`.
Criterios de aceptación (fuentes: contexto funcional §5.2 y arquitectura objetivo):

- Dado permiso de crear y cliente, teléfono, artículo y falla completos, cuando guarda, entonces crea la orden con cliente y equipo relacionados y folio asignado por Rust.
- Dadas dos recepciones sobre la misma secuencia, cuando se confirman, entonces reciben folios distintos sin reutilizar los ocupados por papelera.
- Dado un equipo de otro cliente o falta de permiso, cuando intenta crear, entonces rechaza sin dejar filas parciales ni consumir folio.
- Dado fallo de auditoría, cuando se crea la orden, entonces se revierte también la reserva de folio.

## HRL-13 [Historia] Ficha de recepción: cliente, artículo, antecedentes, precio y observaciones

Estado: Hecho (evidencia: `src/components/orders/OrderForm.vue` formulario, `src/composables/useReceptionForm.ts::display` y `submit`, `src-tauri/src/repositories/orders.rs::OrderRepository::insert_order`). El contrato usa campos relacionales y precio en centavos.
Prioridad: Media, porque reúne los datos necesarios para atender el equipo.
Puntos: 5
Sprint: 2
Subtareas: ninguna faltante detectada. La prueba de creación/cifrado está en `src-tauri/src/services/tests.rs::receipt_creation_encrypts_the_password_and_reuses_exact_customer_and_equipment`, sin ejecutar.
Criterios de aceptación (fuentes: contexto funcional §5.2, `docs/reglas-pagos.md`):

- Dada una recepción guardada, cuando consulta la ficha, entonces presenta cliente, equipo, antecedentes, precio y observaciones persistidos.
- Dado un borrador modificado, cuando cancela y confirma descarte, entonces conserva la ficha guardada previamente.
- Dado un precio fuera de rango o un campo obligatorio vacío, cuando guarda, entonces rechaza y conserva el borrador para corregirlo.

## HRL-14 [Historia] Listado lateral con búsqueda (Todo/Orden/Cliente/Serie), paginación y navegación entre órdenes

Estado: Hecho (evidencia: `src/composables/useReceptionForm.ts::refreshList`, `changeListPage` y `goToOrder`, `src/stores/useOrderStore.ts::listOrders`, `src-tauri/src/services/orders.rs::OrderService::list`, `src-tauri/src/repositories/orders.rs::OrderRepository::list`). La página se consulta en backend; tamaño actual de presentación: cinco órdenes.
Prioridad: Media, porque permite localizar y recorrer expedientes existentes.
Puntos: 5
Sprint: 2
Subtareas: ninguna faltante detectada. Prueba presente, no ejecutada: `src-tauri/src/services/tests.rs::reception_list_search_reference_history_and_balances_survive_reopen`.
Criterios de aceptación (fuentes: contexto funcional §5.2, `docs/papelera-ordenes.md`):

- Dadas órdenes guardadas, cuando busca por Todo, Orden, Cliente o Serie, entonces recibe los resultados del campo elegido con total y páginas coherentes.
- Dada una selección en una página, cuando navega a otra orden, entonces consulta la ficha correspondiente sin alterar sus datos.
- Dado un borrador sin resolver, cuando cambia de orden, entonces impide abandonar los cambios hasta guardarlos o descartarlos.
- Dado un usuario sin Visualizar Clientes, cuando solicita el listado, entonces Rust deniega la consulta.

## HRL-15 [Historia] Editar orden con control de revisión

Estado: Hecho (evidencia: `src-tauri/src/services/orders.rs::OrderService::edit`, `src-tauri/src/repositories/orders.rs::OrderRepository::update_order`, `src/composables/useReceptionForm.ts::edit` y `submit`). Compara revisión, conserva moneda y excluye abonos del guardado de ficha.
Prioridad: Alta, porque afecta precio y trazabilidad y protege cambios concurrentes.
Puntos: 5
Sprint: 2
Subtareas: ninguna faltante detectada. Prueba presente, no ejecutada: `src-tauri/src/services/tests.rs::restored_edit_trash_finances_and_refund_preserve_original_books`. La cobertura de todos los campos auditados tiene trabajo pendiente en HRL-42.
Criterios de aceptación (fuentes: contexto funcional §5.2, `docs/reglas-pagos.md`, `docs/papelera-ordenes.md`):

- Dada una revisión vigente y permiso de editar, cuando guarda la ficha, entonces persisten los cambios y aumenta la revisión.
- Dada una revisión antigua, cuando guarda, entonces informa conflicto y conserva los cambios de pantalla para resolverlos.
- Dada una orden con cobros, cuando reduce el precio, entonces conserva movimientos y calcula el excedente como saldo a favor sin devolver automáticamente.
- Dado un cambio de moneda o un abono enviado dentro de la edición, cuando guarda, entonces rechaza esa alteración.

## HRL-16 [Historia] Consulta por folio o ID desde Herramientas

Estado: Hecho (evidencia: `src/components/layout/AppHeader.vue` menú Herramientas, `src/composables/useWorkshop.ts::receptionTool`, `src/composables/useReceptionForm.ts::lookup`, `src-tauri/src/services/orders.rs::OrderService::find`, `src-tauri/src/repositories/orders.rs::OrderRepository::find_id`).
Prioridad: Media, porque acelera la consulta de expedientes existentes.
Puntos: 3
Sprint: 3
Subtareas: ninguna faltante detectada. Prueba de referencia presente en `src-tauri/src/services/tests.rs::reception_list_search_reference_history_and_balances_survive_reopen`, sin ejecutar.
Criterios de aceptación (fuentes: `docs/backlog-base (1).md` HRL-16, contexto funcional §5.2 y `docs/accesos.md`):

- Dada una orden activa, cuando introduce su folio o ID desde Herramientas, entonces muestra esa ficha sin crear otra orden.
- Dada una referencia vacía o inexistente, cuando consulta, entonces comunica el error y no reemplaza la selección con datos inventados.
- Dado un usuario sin Visualizar Clientes, cuando consulta por referencia, entonces el backend rechaza la solicitud.

Discrepancia D-03: el contexto funcional §4 declara Herramientas deshabilitado; `AppHeader.vue` y `receptionTool` ya conectan consulta, actualización e historial.

## HRL-17 [Historia] Contraseña del equipo cifrada (AES-256-GCM), visible solo bajo solicitud expresa

Estado: Hecho (evidencia: `src-tauri/src/services/crypto.rs::CryptoService::encrypt` y `decrypt`, `src-tauri/src/services/orders.rs::OrderService::get`, `src/composables/useReceptionForm.ts::togglePassword`). Solo el pedido explícito `includeDevicePassword` obtiene el secreto tras autorización.
Prioridad: Alta, porque custodia un secreto recuperable del cliente.
Puntos: 8
Sprint: 2
Subtareas: ninguna faltante detectada en cifrado y revelado. Custodia y respaldo se evalúan aparte en HRL-62 y HRL-70. Prueba presente, no ejecutada: `src-tauri/src/services/tests.rs::crypto_authenticates_ciphertext_nonce_order_and_key_and_uses_fresh_nonces`.
Criterios de aceptación (fuentes: arquitectura objetivo §7, contexto funcional §5.2 y `docs/bitacora.md`):

- Dada una contraseña de equipo, cuando guarda, entonces persiste un sobre AES-256-GCM y la clave de cifrado permanece fuera de SQLite.
- Dada una consulta ordinaria, cuando carga la orden, entonces no recibe el secreto; al solicitarlo expresamente requiere acceso a esa orden.
- Dado un sobre alterado o permiso denegado, cuando solicita revelarlo, entonces falla sin exponer contraseña ni material criptográfico.
- Dada una orden con contraseña, cuando imprime o consulta auditoría, entonces el secreto no aparece en esos documentos ni eventos.

Discrepancia D-04: el contexto funcional mantiene pendiente el tratamiento de la contraseña; arquitectura §7 y `CryptoService` ya concretan cifrado recuperable. No se cambia aquí la decisión del proyecto.

## HRL-18 [Historia] Abono inicial al crear la orden (con permiso de cobro)

Estado: Hecho (evidencia: `src-tauri/src/services/orders.rs::OrderService::create` y `validate`, `src-tauri/src/repositories/orders.rs::OrderRepository::initial_payment`, `src/composables/useReceptionForm.ts::submit` y `canPay`). Incluye permiso independiente, solicitud con huella y auditoría en la transacción de recepción.
Prioridad: Alta, porque registra dinero y no debe duplicar cobros.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante detectada. Prueba presente, no ejecutada: `src-tauri/src/services/tests.rs::same_financial_request_is_idempotent_and_changed_content_cannot_reuse_it`.
Criterios de aceptación (fuentes: `docs/reglas-pagos.md`, `docs/acceso-temporal.md`):

- Dado permiso de crear y de cobrar, cuando crea con abono positivo no mayor al precio y método libre, entonces orden, abono y auditoría se confirman juntos.
- Dado permiso de crear pero no de cobrar, cuando incluye abono, entonces rechaza toda la recepción sin crear el cobro ni la orden.
- Dada la misma solicitud y huella, cuando reintenta tras un resultado incierto, entonces obtiene la recepción original sin duplicar dinero.

## HRL-19 [Historia] Evidencias fotográficas por archivo o cámara (JPEG/PNG/WebP, máx. 20 MiB)

Estado: Hecho por inspección de implementación (evidencia: `src/composables/useOrderEvidence.ts::importFiles`, `capture` y `checkPhoto`, `src/components/orders/EvidenceCamera.vue::enable`, `take` y `stop`, `src-tauri/src/domain/evidence_contracts.rs::validate`, `src-tauri/src/services/evidence.rs::EvidenceService::add`). La cámara física y sus permisos nativos están No verificados: HRL-64.
Prioridad: Media, porque documenta condiciones de recepción y trabajo técnico.
Puntos: 5
Sprint: 3
Subtareas: ninguna faltante de implementación detectada. Prueba relacional presente, no ejecutada: `src-tauri/src/services/tests.rs::restored_evidence_and_company_reopen_without_business_json`.
Criterios de aceptación (fuente: `docs/evidencias.md`):

- Dada una orden guardada activa y permiso de crear, cuando selecciona JPEG, PNG o WebP de hasta 20 MiB, entonces guarda una copia local con sus metadatos y auditoría.
- Dada la cámara autorizada, cuando captura o cierra, entonces no solicita micrófono y detiene las pistas de video.
- Dado un formato, cabecera o tamaño inválido, o cámara denegada, cuando intenta agregar, entonces muestra el error sin registrar una evidencia inválida.
- Dado un resultado incierto, cuando reintenta con el mismo identificador y contenido, entonces no duplica fotografía ni evento.

Discrepancia D-05: `docs/evidencias.md` describe carpeta por folio hexadecimal y raíz `app_local_data_dir`; `EvidenceService::relative` usa `evidencias/<id interno>/<id foto>.<ext>` bajo el directorio de assets del nuevo `Database`. Los respaldos deben identificar la ruta normalizada real.

## HRL-20 [Historia] Descripción, retiro y restauración de evidencias

Estado: Hecho (evidencia: `src/composables/useOrderEvidence.ts::saveNote` y `remove`, `src-tauri/src/services/evidence.rs::EvidenceService::update`, `src-tauri/migrations/001_esquema_objetivo.sql::evidencias_conservar_original` y `evidencias_no_delete`). Exige revisión esperada, permisos separados y orden activa.
Prioridad: Media, porque conserva las fotografías y la historia de sus cambios.
Puntos: 3
Sprint: 3
Subtareas: ninguna faltante detectada en el flujo leído. Pruebas de integración presentes en `src-tauri/src/services/tests.rs::restored_evidence_and_company_reopen_without_business_json`, sin ejecutar.
Criterios de aceptación (fuente: `docs/evidencias.md`):

- Dado permiso de editar y revisión vigente, cuando guarda una descripción de hasta 500 caracteres, entonces persiste con valores anterior y nuevo en auditoría.
- Dado permiso de borrar, cuando retira y después restaura una foto, entonces cambia su visibilidad conservando archivo, autor y fecha originales.
- Dada una revisión antigua, descripción excesiva u orden en papelera, cuando modifica evidencia, entonces rechaza y conserva el original.

## HRL-21 [Historia] Papelera: eliminar y restaurar órdenes conservando historial

Estado: Hecho (evidencia: `src-tauri/src/services/orders.rs::OrderService::trash`, `src-tauri/src/repositories/orders.rs::OrderRepository::trash`, `src/composables/useReceptionForm.ts::setDeleted` y `toggleTrash`). Usa eliminación lógica y revisión; la restauración no crea otra orden.
Prioridad: Alta, porque debe conservar folio, dinero y auditoría sin borrado físico.
Puntos: 5
Sprint: 3
Subtareas: ninguna faltante detectada. Prueba presente, no ejecutada: `src-tauri/src/services/tests.rs::restored_edit_trash_finances_and_refund_preserve_original_books`.
Criterios de aceptación (fuente: `docs/papelera-ordenes.md`):

- Dada una orden activa y permiso Borrar Clientes, cuando confirma eliminar, entonces sale del listado activo y conserva expediente, folio, fotos, libros e historial.
- Dada una orden en papelera, cuando una persona autorizada restaura, entonces recupera la misma orden y registra autoría del evento.
- Dado permiso insuficiente o revisión antigua, cuando elimina o restaura, entonces rechaza sin cambiar visibilidad.
- Dada una entrega, pago o cambio de estado técnico, cuando se registra, entonces no envía automáticamente a papelera ni cancela deuda.

Discrepancia D-06: `docs/papelera-ordenes.md` deriva la visibilidad de `Historial_papelera`; el repositorio normalizado escribe `ordenes.eliminada_en` y el servicio registra auditoría.

## HRL-22 [Historia] Nota de entrada imprimible

Estado: Hecho por inspección del flujo (evidencia: `src/composables/useWorkshop.ts::print`, `src/components/company/CompanyNote.vue` rama `kind === 'entry'`, `src/utils/printCompanyNote.ts::printCompanyNote`). La salida física por impresora queda No verificada en HRL-64.
Prioridad: Media, porque entrega constancia de recepción al cliente.
Puntos: 3
Sprint: 4
Subtareas: ninguna faltante de implementación detectada; validación nativa transversal en HRL-64.
Criterios de aceptación (fuentes: contexto funcional §§3 y 5.2, `docs/auditoria-empresa.md`):

- Dada una orden guardada y configuración de Empresa cargada, cuando solicita nota de entrada, entonces prepara cliente, equipo, folio, notas y cláusulas aplicables.
- Dada una contraseña del equipo, cuando imprime, entonces no aparece en la nota.
- Dado un borrador pendiente o error al preparar los datos, cuando solicita imprimir, entonces impide emitir una nota con cambios sin guardar o datos incompletos.

## HRL-23 [Historia] Guardas de borrador al cambiar de apartado o cerrar sesión

Estado: Hecho (evidencia: `src/composables/useReceptionForm.ts::prepareLeave`, `cancel` y `discard`, `src/composables/useWorkshop.ts::prepareLeave`, `navigate` y `logout`, `src/App.vue::prepareLeave` expuesto). Recepción exige resolver cambios; Empresa bloquea salida y Reparaciones intenta guardar.
Prioridad: Media, porque evita perder capturas al navegar o terminar sesión.
Puntos: 3
Sprint: 2
Subtareas: ninguna faltante detectada en la coordinación leída. Recorrido nativo de cierre en HRL-64.
Criterios de aceptación (fuentes: `docs/acceso-validacion.md`, `docs/papelera-ordenes.md`, contexto funcional §4):

- Dada una ficha o Empresa con cambios, cuando cambia de apartado o cierra sesión, entonces debe guardar o descartar antes de completar la salida.
- Dada una recepción modificada, cuando cancela y elige seguir editando o Escape, entonces conserva el borrador; al confirmar descarte conserva solo lo persistido.
- Dada una reparación con cambios, cuando falla el guardado previo a salir, entonces permanece en Reparaciones y muestra el error.
