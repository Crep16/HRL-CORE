# Contrato de intercambio con el portal

## Fuente de implementación

**[EVIDENCIA]** La proyección se arma explícitamente en `src-tauri/src/domain/portal.rs:207-220,249-329`; el transporte REST Firestore está en `src-tauri/src/services/portal_transport.rs:204-215`; la descripción local del emisor está en `docs/evidencia/emisor-portal.md:29-36`.

## Campos publicados

**[EVIDENCIA]** La proyección envía una lista construida de campos, sin serializar la orden entera (`src-tauri/src/domain/portal.rs:207-220,318-329`):

- Campos base: `versionContrato`, `idPublico`, `revision`, `publicado`, `huella`.
- Equipo: `equipo` con `tipo`, `marca`, `modelo`.
- Seguimiento: `estadoTecnico`, `etapas` (`nombre`, `completada`, opcional `completadaEn`).
- Fechas disponibles: `recibidoEn`, `entregaEstimada`, `entregadoEn`, `accesoHasta`.
- Secciones opcionales sujetas a preferencias: `eventos` (fecha/título/detalle de avances visibles al cliente), `notaPublica`, `garantias` (cobertura/días), `pago`.
- El emisor agrega `actualizadoEn` como hora del servidor `REQUEST_TIME` al commit REST.

**[EVIDENCIA]** El resumen de pago contiene exactamente `moneda`, `totalCentavos`, `cobradoCentavos` y `pendienteCentavos` (`src-tauri/src/domain/portal.rs:150-181`). Los importes son enteros limitados a `MAX_SAFE_INTEGER`; `pendiente = max(total - cobrado, 0)`. No es historial de abonos. La regla sobre valor neto (ajustes/devoluciones) está descrita en `docs/contrato/CONTRATO-FIRESTORE.md`, sección 4, y en `docs/evidencia/emisor-portal.md:31-34`.

## Datos que permanecen privados

**[EVIDENCIA]** La revisión del emisor registra como ausentes en la proyección: folio, contraseña del equipo, serie, notas internas, datos personales de cliente, usuarios/permisos, auditoría interna, costos/gastos/utilidad, historial detallado de cobros, rutas locales y adjuntos (`docs/evidencia/emisor-portal.md:34`). Los avances privados se filtran en `src-tauri/src/domain/portal.rs:262-280`; la nota se toma de `public_note` y se serializa como `nota_publica` (`src-tauri/src/domain/portal.rs:193-201,282-286`).

**[LÍMITE]** El repositorio también contiene documentos de contrato y reglas con diferencias de versión/aprobación. `docs/contrato/CONTRATO-FIRESTORE.md` señala que es documento de trabajo y pendientes de validación; `docs/evidencia/emisor-portal.md` registra campos emitidos que no coinciden con el borrador `docs/CONTRATO-FIRESTORE.md` (incluida huella/pago). No se concluye que Firebase real acepte o rechace el contrato: no se encontró una prueba contra proyecto real.

## Huella de contenido

**[EVIDENCIA]** Algoritmo SHA-256 hexadecimal minúsculo sobre JSON canónico del documento proyectado, excluyendo `actualizadoEn` y `huella` (`src-tauri/src/domain/portal.rs:391-400`; referencia JS `docs/contrato/huella.js:16-20`). La canonicalización se implementa en `src-tauri/src/domain/portal.rs:412-440`; la referencia JS ordena claves recursivamente y codifica UTF-8. Arreglos preservan su orden. Fechas se normalizan a UTC con milisegundos en `docs/contrato/huella.js:5-10`. La prueba del vector compartido está en `src-tauri/src/domain/portal/tests.rs::fingerprint_matches_the_portal_javascript_contract_vector`.

### Referencias de línea

Los números pueden variar si el archivo cambia. Consulta reproducible: `rg -n "pub fn fingerprint_document|Sha256|PublicProjection|notaPublica|totalCentavos|REQUEST_TIME|updateTransforms|private|visible_cliente" src-tauri/src/domain/portal.rs src-tauri/src/services/portal_transport.rs src-tauri/src/services/portal_publisher.rs`.

**[EVIDENCIA]** `src-tauri/src/services/portal_transport.rs:204-215` construye el cuerpo REST y `actualizadoEn = REQUEST_TIME`; `src-tauri/src/services/portal_publisher.rs:442,573` verifica huella y resuelve reconciliación. El commit REST contiene `update` completo sin `updateMask` (`portal_transport.rs:214`).
