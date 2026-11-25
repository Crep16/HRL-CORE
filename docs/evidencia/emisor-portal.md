# Emisor de actualizaciones del portal

Revisión estática de `src-tauri/src/domain/portal.rs`, `src-tauri/src/services/portal_publisher.rs`, `portal_transport.rs`, `portal_sender_tests.rs`, comandos, repositorio y migración `006_publicacion_portal.sql`, más `docs/CONTRATO-FIRESTORE.md`, reglas locales y `docs/contrato/INTEGRACION-HRL-CORE.md`. No contiene credenciales ni tokens.

## 1. Destino y transporte

El backend envía directamente a la API REST de Google Cloud Firestore. No usa un endpoint HTTP propio para publicar. El `project_id` forma parte de la ruta y la configuración almacena también una URL HTTPS del portal, pero esa URL sirve para enlace/QR; no sustituye el destino fijo `firestore.googleapis.com`.

- Inicio de sesión Firebase Auth: `POST https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={api_key}`, JSON `{email, password, returnSecureToken:true}`.
- Renovación: `POST https://securetoken.googleapis.com/v1/token?key={api_key}`, formulario `grant_type=refresh_token` y `refresh_token`.
- Publicación: `POST https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents:commit`, con `Authorization: Bearer {idToken}` y JSON REST de Firestore.
- Reconciliación: `GET https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/seguimiento/{idPublico}`, con el mismo bearer.

El cuerpo de publicación tiene `writes: [{ update: { name: "projects/{project}/databases/(default)/documents/seguimiento/{idPublico}", fields: ... }, updateTransforms: [{ fieldPath: "actualizadoEn", setToServerValue: "REQUEST_TIME" }] }]`. Convierte números a `integerValue` decimal como texto, fechas a `timestampValue`, y mapas/listas a sus tipos REST. Es un reemplazo completo del documento: no envía `updateMask`. La respuesta de commit debe aportar `writeResults[0].updateTime` para confirmar el envío.

El `project_id` es configurable; las URLs base de Google y la colección `seguimiento` están fijas en el código. La `api_key` se configura y se almacena en `portal_configuracion.clave_api`; no reproduzco su valor. No encontré claves de producción ni tokens literales hardcodeados en los archivos revisados; los valores de prueba son marcadores. La clave se inserta en query string de Auth, por lo que no debe copiarse a logs o reportes.

## 2. Autenticación y custodia

Un usuario de Firebase Authentication inicia sesión con correo y contraseña del publicador. HRL CORE comprueba que el UID devuelto coincide con el `publisher_uid` configurado. La contraseña se envuelve en `Zeroizing` durante el flujo y no se almacena. El ID token queda en memoria y se presenta como bearer. El refresh token se protege con DPAPI en `keys/portal-publicador.dpapi`; se comprueba además una sola instancia publicadora con `portal-publicador.lock`. La migración no copia estas credenciales a los respaldos según `docs/contrato/INTEGRACION-HRL-CORE.md`.

La configuración/conexión exige acceso Completo y permiso de Empresa; las acciones de publicación usan autorización de Reparaciones. `src-tauri/src/commands/portal.rs` expone `portal_settings`, `configure_portal`, `connect_portal`, `pause_portal`, `resume_portal`, `disconnect_portal`, `portal_order_status` y `portal_order_action`.

## 3. Contenido que sale

El servicio serializa una proyección explícita; no serializa la orden completa. Campos que el código puede emitir:

- Control: `versionContrato`, `idPublico`, `revision`, `publicado` y `huella`.
- Equipo: `equipo.tipo`, `equipo.marca`, `equipo.modelo`; no se ve el número de serie en la proyección.
- Reparación: `estadoTecnico` y `etapas` (`nombre`, `completada`, `completadaEn` opcional).
- Opcionales según configuración de sección: `eventos` (solo avances marcados públicos, con `fecha`, `titulo`, `detalle`), `notaPublica`, `garantias` (`cobertura`, `dias`) y `pago` (`moneda`, `totalCentavos`, `cobradoCentavos`, `pendienteCentavos`).
- Fechas adicionales en el código: `recibidoEn`, `entregaEstimada`, `entregadoEn` y `accesoHasta`. `actualizadoEn` lo establece Firestore con `REQUEST_TIME`, no lo calcula el cliente.

La retirada publica control más las fechas de entrega/vencimiento disponibles y omite el contenido de seguimiento. El código no incluye contraseña del equipo, nombre/teléfono/domicilio del cliente, notas internas, costos, gastos, utilidad, datos de usuarios, permisos, auditoría, rutas locales ni archivos/fotos/documentos en la proyección revisada. La nota pública se habilita aparte de notas internas. La proyección limita la estructura/campos y longitudes, pero el código revisado no filtra semánticamente nombres, contraseñas u otros datos sensibles que alguien pudiera escribir dentro de texto público.

**Diferencia que requiere acuerdo:** el código también transmite `huella` y puede transmitir `pago`; el borrador `docs/CONTRATO-FIRESTORE.md` no incluye `huella` y deja `pago` fuera hasta decisión. La regla local `docs/firestore.rules` tampoco permite esos campos; además tiene UID marcador. El borrador está explícitamente sin aprobar y la integración no se ha probado en Firebase real, así que no concluyo que la regla desplegada sea esa ni que el código esté rechazado en producción. También hay fechas que el código emite y no aparecen en la tabla de campos del borrador: `recibidoEn`, `entregaEstimada` y `entregadoEn`.

## 4. Cola local, estados e idempotencia

La migración `006_publicacion_portal.sql` crea configuración, política/revisión por orden, códigos activos, publicaciones normalizadas, tablas separadas para etapas/eventos/garantías y `portal_envios`. No guarda la publicación como JSON: guarda sus valores y una huella. Triggers sellan publicaciones y sus elementos como inmutables.

Estados de `portal_envios`: `pending`, `sending`, `confirmed`, `superseded`, `error`, `conflict`. La captura local de revisión, proyección y auditoría se realiza con la operación que la originó. El worker envía después; por orden selecciona solo la revisión pendiente más reciente. Una revisión nueva vuelve obsoletas las antiguas no enviadas. El servicio serializa trabajo y usa un lock para evitar dos publicadores locales simultáneos.

Cada snapshot conserva la misma revisión y huella en sus reintentos. Si se pierde la respuesta, al reanudar o resolver un `PERMISSION_DENIED` lee el documento remoto: misma revisión y huella significa confirmado; misma revisión y huella distinta, conflicto; revisión remota superior, superado. La clave única local de orden/revisión y esa comparación evitan aplicar dos veces una actualización lógica.

Errores de conexión, timeout, HTTP 408, 429 y 5xx vuelven a `pending`. El retraso crece exponencialmente desde unos 2 segundos, añade variación aleatoria y queda limitado a 300 segundos; el código no fija un número máximo de intentos. Sin internet la operación local y su auditoría permanecen guardadas; publicar se reintenta después. Si la autenticación ya no sirve, intenta renovar y luego deja la cola pausada para reconectar. Contenido inválido termina en error visible, no en reintento infinito. Un 403 desencadena lectura/reconciliación antes de decidir confirmado, superado, conflicto o error.

## 5. Cobertura y límites de las pruebas

Pruebas Rust relevantes por nombre:

- `only_public_event_fields_cross_the_boundary`
- `hidden_sections_are_absent_and_visible_empty_lists_are_preserved`
- `payments_preserve_credit_and_validate_the_view_balance`
- `withdrawal_has_no_tracking_content_and_keeps_expiration`
- `delivery_alone_starts_exactly_seven_days_of_access`
- `custom_stages_keep_their_order_and_dates_require_actual_instants`
- `invalid_public_data_is_rejected_without_truncating_the_local_source`
- `portal_sender_confirms_a_lost_response_by_revision_and_fingerprint`
- `portal_sender_keeps_conflicts_paused_until_explicit_order_reconciliation`
- `portal_sender_pauses_on_revoked_authentication_without_losing_local_changes`
- `portal_sender_recovers_when_remote_commit_succeeds_and_local_audit_fails`
- `portal_sender_delay_is_bounded_and_a_different_uid_is_rejected`
- `portal_status_does_not_confirm_invalid_content_or_reuse_an_old_code_confirmation`
- `portal_status_only_exposes_the_confirmation_of_the_latest_publication`
- `portal_publishes_local_repair_events_as_utc_without_rewriting_the_history`
- `rest_body_replaces_everything_and_uses_server_time_and_integer_strings`
- `dpapi_vault_keeps_only_the_encrypted_refresh_token_and_supports_rotation`
- `publisher_lease_serializes_instances_and_is_released_when_the_handle_closes`

`src-tauri/src/services/tests.rs` también contiene `portal_queue_preserves_the_exact_snapshot_across_later_money_and_price_changes`, `portal_manual_withdrawal_rotation_and_delivery_follow_the_public_contract` y `portal_storage_failure_rolls_back_the_order_and_invalid_public_text_does_not`. Hay pruebas frontend en `tests/portal.test.ts` con IPC simulado.

Las pruebas usan un gateway sustituible/falso y verifican proyección, cola, serialización REST, errores simulados, renovación/custodia de token y reconciliación. No prueban inicio de sesión real, reglas desplegadas, permisos de Firebase del proyecto del compañero, despliegue del portal web ni lectura real del documento por el cliente. `docs/contrato/INTEGRACION-HRL-CORE.md` confirma que todavía no se publicó una orden real en Firebase.

## 6. Estado del contrato

`docs/CONTRATO-FIRESTORE.md` se titula propuesta/borrador y dice que no está aprobado ni implementado. Define como propuesta la colección `seguimiento/{idPublico}`, una lista permitida de campos, solo lectura por enlace, revisión creciente, documento de retirada en vez de borrado y actualización del campo de servidor. Prohíbe compartir la orden completa y enumera datos privados que no deben publicarse.

Deja abiertas, entre otras, la aprobación de versión, si mostrar `folio` o `pago`, modo y responsable de publicación, catálogo de estados/etapas, longitud/forma del identificador, uso y plazo de `accesoHasta`, reconciliación al restaurar/cambiar equipo, región y conservación de documentos retirados. `docs/contrato/INTEGRACION-HRL-CORE.md` agrega que no hay validación remota real; las reglas recibidas y el cálculo de saldo a favor tampoco coinciden aún plenamente con lo requerido localmente.

## 7. Preguntas para el desarrollador del portal

1. ¿Se confirma `seguimiento` como nombre de colección y `idPublico` como ID directo del documento? ¿El ID canónico son 12 caracteres agrupados con guiones, y cómo se resuelven las rotaciones?
2. ¿La regla desplegada acepta `huella` y el objeto `pago`? Si no, ¿qué contrato y versión deben implementar ambos lados? ¿También acepta `recibidoEn`, `entregaEstimada` y `entregadoEn`?
3. ¿Qué campos son obligatorios/opcionales en creación, actualización y retirada? ¿Cómo se valida el `REQUEST_TIME` y la revisión estrictamente creciente?
4. ¿La lectura del portal consulta exactamente `seguimiento/{idPublico}` por enlace/QR? ¿Qué tratamiento aplica a documento inexistente, vencido, retirado y caché offline?
5. ¿Cuál es el UID final autorizado, cómo se limitará el permiso de escritura y se prohíben `list`/`delete`? No copiar credenciales al intercambio.
6. ¿Qué valores acepta `estadoTecnico` y cuál es el catálogo/orden de `etapas` publicado?
7. ¿Se aprueba incluir el resumen de pago? Si sí, confirmar nombres, tipos, enteros en centavos y regla del saldo a favor.
8. ¿Qué región, expiración, retención de retirados y procedimiento de reconciliación tras restaurar respaldo se usarán?