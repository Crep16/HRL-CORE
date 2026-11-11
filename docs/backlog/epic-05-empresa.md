# HRL-48 [Epic] Módulo de Empresa

Estado: En progreso por validaciones de contacto pendientes en Rust (HRL-49). Los demás flujos están implementados por lectura estática; impresión física No verificada, en HRL-64.
Prioridad: Alta, porque configura folios, documentos y datos usados por otros módulos.
Puntos: 34 (suma de historias, sin duplicar el epic).
Sprint: 4 para todas sus historias.
Subtareas: agrupador de HRL-49 a HRL-56.

Fuente funcional: `docs/auditoria-empresa.md`, tabla de siete vistas y límites comprobables. Persistencia/revisión: arquitectura objetivo y `docs/bitacora.md`. Puntos de alcance completo; no se ejecutó ninguna prueba.

## HRL-49 [Historia] Datos generales: nombre, identificación fiscal, contacto, dirección y horario

Estado: En progreso (evidencia: `src/composables/useCompany.ts::companyErrors`, `src/composables/useCompanyWorkspace.ts::save`, `src-tauri/src/services/company.rs::CompanyService::save`, `src-tauri/src/repositories/company.rs::CompanyRepository::save`). Los campos persisten en SQLite, pero el formato de correo y web se valida solo en frontend; una llamada directa al servicio no repite esas validaciones.
Prioridad: Media, porque define la identificación y contacto que aparecen en las notas.
Puntos: 5
Sprint: 4
Subtareas:

- HRL-75 [Servicio Rust] Aplicar en `CompanyService::save` las reglas documentadas de correo y sitio web, admitiendo campos opcionales vacíos como el formulario actual.
- HRL-76 [Pruebas] Cubrir llamadas directas con correo/web inválidos, valores opcionales vacíos y rechazo sin modificar configuración ni auditoría.

Criterios de aceptación (fuente: `docs/auditoria-empresa.md`, Datos generales y validación):

- Dados datos generales válidos y permiso de editar Empresa, cuando guarda, entonces conserva nombre, identificación fiscal, contacto, dirección y horario y los usa en la vista previa.
- Dado un correo o sitio web inválido, cuando intenta guardar incluso por IPC directo, entonces rechaza y conserva la configuración anterior.
- Dado un usuario sin `company.edit`, cuando intenta guardar, entonces Rust deniega sin alterar Empresa.

Discrepancia D-10: la validación de correo/web documentada existe en `companyErrors`, pero no en `CompanyService::save`; el frontend no sustituye la validación de negocio del servicio.

## HRL-50 [Historia] Logo y su uso en documentos

Estado: Hecho (evidencia: `src/composables/useCompanyWorkspace.ts::readLogo`, `upload` y `pasteLogo`, `src-tauri/src/services/company.rs::CompanyService::save_logo` y `read_logo`, `src/components/company/CompanyNote.vue` imagen condicionada por `logoPrint`). El archivo queda fuera de SQLite, con referencia y huella en el nombre.
Prioridad: Media, porque identifica al negocio en sus documentos.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante de implementación detectada; portapapeles e impresión nativos en HRL-64. Prueba presente `src-tauri/src/services/tests.rs::restored_evidence_and_company_reopen_without_business_json`, sin ejecutar.
Criterios de aceptación (fuente: `docs/auditoria-empresa.md`, Logo):

- Dado un PNG/JPG de hasta 1 MiB, cuando lo carga y guarda, entonces conserva el logo y lo muestra en notas si se habilitó su uso en impresión.
- Dado formato, cabecera o tamaño inválido, cuando intenta guardar, entonces rechaza sin sustituir el logo confirmado.
- Dado un cambio de logo, cuando consulta auditoría, entonces identifica el cambio sin copiar el contenido binario de la imagen.
- Dado portapapeles no disponible o sin permiso, cuando intenta pegar, entonces informa el fallo y permite subir archivo.

## HRL-51 [Historia] Notas impresas: mensajes de entrada/salida, firma y referencia

Estado: Hecho (evidencia: `src-tauri/src/services/company.rs::CompanyService::save`, `src-tauri/src/repositories/company.rs::CompanyRepository::save`, `src/components/company/CompanyNote.vue` pie, firma y referencia). El documento distingue entrada/salida y presenta referencia interna sin inventar un enlace web.
Prioridad: Media, porque define contenido que recibe el cliente.
Puntos: 3
Sprint: 4
Subtareas: ninguna faltante detectada para configuración y documento; validación física en HRL-64.
Criterios de aceptación (fuente: `docs/auditoria-empresa.md`, Notas impresas):

- Dados mensajes de entrada y salida distintos, cuando prepara cada nota, entonces incluye el pie que corresponde.
- Dadas opciones de firma, dirección y referencia, cuando prepara el documento, entonces respeta las opciones guardadas y usa folio interno como referencia.
- Dado un mensaje mayor a 200 caracteres o permiso insuficiente, cuando guarda, entonces rechaza sin modificar configuración.

## HRL-52 [Historia] Cláusulas: título, texto, orden y aplicación a entrada/salida

Estado: Hecho funcionalmente (evidencia: `src/components/company/CompanyClauses.vue::apply`, `move`, `remove` y `undo`, `src-tauri/src/services/company.rs::CompanyService::save`, `src-tauri/src/repositories/company.rs::CompanyRepository::save`). El repositorio conserva orden en `posicion`; el componente aún contiene lógica de formulario (deuda MVVM, HRL-61).
Prioridad: Media, porque incorpora condiciones del servicio a las notas.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante funcional detectada; separación arquitectónica registrada en HRL-61.
Criterios de aceptación (fuente: `docs/auditoria-empresa.md`, Cláusulas):

- Dadas cláusulas con título/texto válidos, cuando reordena y guarda, entonces conserva orden y selección de entrada/salida en las notas correspondientes.
- Dada una cláusula quitada del borrador, cuando pulsa Deshacer antes de guardar o descartar, entonces la recupera una sola vez en su posición.
- Dado título vacío o mayor a 80 caracteres, o texto vacío o mayor a 400, cuando guarda, entonces rechaza y conserva el borrador.
- Dada ubicación al pie o reverso y numeración elegida, cuando prepara la nota, entonces respeta esas opciones.

## HRL-53 [Historia] Hoja de impresión: carta, ticket 80/60 mm, orientación y copias

Estado: Hecho por lectura del motor (evidencia: `src/utils/printLayout.ts::printLayout`, `src/utils/paginatePrintCopies.ts::paginatePrintCopies`, `src/utils/printCompanyNote.ts::printCompanyNote`, `src-tauri/src/services/company.rs::CompanyService::save`). Impresora y diálogo nativos No verificados, HRL-64.
Prioridad: Media, porque hace utilizables los documentos en los formatos del taller.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante de implementación detectada; validación física transversal en HRL-64.
Criterios de aceptación (fuente: `docs/auditoria-empresa.md`, Hoja de impresión y ahorro de papel):

- Dada carta vertical u horizontal, cuando prepara dos copias, entonces las agrupa en una hoja con línea de corte; cinco copias forman grupos 2, 2 y 1.
- Dado ticket 80 o 60 mm, cuando imprime varias copias, entonces prepara el ancho elegido con márgenes de 3 mm y copias separadas.
- Dado texto extenso, cuando pagina carta, entonces continúa en otras hojas sin perder texto ni reducir tipografía para forzar el ajuste.
- Dadas copias fuera de 1 a 5, formato inválido o fallo de preparación, cuando guarda/imprime, entonces informa error y no presenta una impresión exitosa.

## HRL-54 [Historia] Folios y garantías: prefijo, siguiente número y días sugeridos

Estado: Hecho (evidencia: `src-tauri/src/services/company.rs::CompanyService::save`, `src-tauri/src/repositories/company.rs::CompanyRepository::save`, `src-tauri/src/repositories/orders.rs::OrderRepository::reserve_folio`, `src/composables/useReceptionForm.ts::begin`, `src/composables/useRepairScreen.ts` inicialización de días). Los cambios de secuencia no retroceden la reserva almacenada.
Prioridad: Alta, porque la numeración bloquea la creación de órdenes sin colisión.
Puntos: 3
Sprint: 4
Subtareas: ninguna faltante detectada; prueba de folios concurrentes presente en `src-tauri/src/services/tests.rs::concurrent_receipts_use_distinct_folios_and_audit_authorship`, sin ejecutar.
Criterios de aceptación (fuente: `docs/auditoria-empresa.md`, Folios y garantías):

- Dado prefijo y siguiente número guardados, cuando crea una recepción, entonces Rust reserva un folio disponible sin sobrescribir órdenes existentes ni folios de papelera.
- Dado un borrador nuevo cancelado, cuando descarta, entonces no consume un folio.
- Dados días sugeridos de mano de obra y refacciones, cuando abre su captura de garantía, entonces usa la sugerencia correspondiente sin cambiar garantías ya registradas.
- Dado prefijo inválido o días fuera de 0 a 3650, cuando guarda Empresa, entonces rechaza sin cambiar configuración.

## HRL-55 [Historia] Región e impuestos: país, zona horaria, moneda e impuesto incluido

Estado: Hecho (evidencia: `src-tauri/src/services/company.rs::CompanyService::save`, `src/services/api/company.ts` adaptación de tasa, `src/composables/useCompanyNote.ts::subtotal` y `money`, `src/utils/finances.ts::utcDayBoundary`). Guarda tasa en milésimas y región/moneda/zona entre opciones admitidas.
Prioridad: Alta, porque afecta presentación de dinero y límites temporales de auditoría.
Puntos: 3
Sprint: 4
Subtareas: ninguna faltante funcional detectada; cálculos de presentación y deuda de reglas en utils se revisan en HRL-60.
Criterios de aceptación (fuente: `docs/auditoria-empresa.md`, Región e impuestos):

- Dada región y moneda configuradas, cuando muestra importes y fechas, entonces aplica formato y zona elegidos y mantiene la interfaz en español sin convertir monedas.
- Dado impuesto incluido habilitado, cuando presenta un total, entonces desglosa subtotal como total dividido entre uno más la tasa y no aumenta el total registrado.
- Dada una tasa fuera de 0 a 100 % o una opción de moneda/zona no admitida, cuando guarda, entonces Rust rechaza.
- Dada una orden existente en otra moneda, cuando cambia Empresa, entonces conserva la moneda y los importes históricos de esa orden.

## HRL-56 [Historia] Edición como borrador con control de revisión, vista previa y auditoría

Estado: Hecho (evidencia: `src/composables/useCompanyWorkspace.ts::save`, `discard` y `dirty`, `src/stores/useCompanyStore.ts::run`, `src-tauri/src/services/company.rs::CompanyService::save`). Solo actualiza configuración confirmada después de respuesta válida y rechaza revisión antigua.
Prioridad: Alta, porque evita pérdida de cambios y mantiene configuración e historial juntos.
Puntos: 5
Sprint: 4
Subtareas: ninguna faltante detectada; prueba presente `src-tauri/src/services/tests.rs::restored_company_and_users_persist_with_permissions_and_revisions`, no ejecutada.
Criterios de aceptación (fuentes: `docs/auditoria-empresa.md`, `docs/bitacora.md`):

- Dado un borrador de Empresa, cuando cambia sus campos, entonces actualiza vista previa sin sustituir la configuración confirmada hasta guardar.
- Dada una revisión antigua o fallo de escritura/auditoría, cuando guarda, entonces conserva configuración anterior y borrador con error recuperable.
- Dado un borrador con cambios, cuando intenta salir, entonces exige guardarlos o descartarlos; descartar restaura la configuración confirmada.
- Dado guardado autorizado correcto, cuando confirma, entonces datos, opciones y diferencias de auditoría se guardan en la misma transacción.

Discrepancia D-11: `docs/auditoria-empresa.md` y `docs/accesos.md` describen Empresa en `hrl.company.v1`/localStorage; la ruta actual `useCompanyStore → companyApi → CompanyService → CompanyRepository` usa SQLite y archivos de logo. Las marcas PASS de documentos antiguos no acreditan ejecución sobre el árbol revisado.
