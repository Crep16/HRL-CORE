# HRL-71 [Epic] Sincronización con el portal web

Fuente única de alcance: `docs/backlog-base (1).md`, leída el 12 de noviembre de 2026. El CSV `docs/jira-import (1).csv` corrobora claves, padres y sprints; no sustituye los títulos completos del Markdown. Este epic reemplaza la historia eliminada HRL-66, que no se cuenta ni se recrea como historia vigente.

Estado: Pendiente (evidencia: `src-tauri/src/services/repairs.rs::RepairService::save` y `src-tauri/src/repositories/repairs.rs::RepairRepository::save` conservan modo y visibilidad en tablas locales; `src/services/api/workshop.ts::workshopApi` contiene IPC local; `src-tauri/src/commands/mod.rs` y `src-tauri/src/lib.rs::run` no registran publicación remota). La búsqueda en servicios, repositorios y API no localizó cliente del portal, cola de intercambio ni pruebas contra ese contrato. No se concluye que un servicio externo fuera del repositorio no exista.
Prioridad: Alta, porque publica datos del cliente y exige autorización, privacidad y trazabilidad.
Puntos: 29 (suma de tareas; no sumar de nuevo el epic).
Sprint: 3, 4 y Backlog, según tarea.
Subtareas: agrupador de HRL-72 a HRL-76; trabajo técnico nuevo HRL-115 a HRL-126.

Criterios de aceptación del epic (fuentes: `docs/contexto-completo-proyecto.md` §9, `docs/reparaciones-permisos.md`, `docs/bitacora.md`):

- Dadas preferencias guardadas localmente, cuando no hay intercambio confirmado con el portal, entonces no se presentan como una publicación realizada.
- Dados datos públicos e internos de reparación, cuando se prepara información para el cliente, entonces se excluyen notas internas y contraseñas del equipo.
- Dado permiso insuficiente o fallo de publicación, cuando se procesa la acción, entonces no se concede acceso por la UI ni se informa éxito remoto sin evidencia.

No se eligen proveedor, endpoints, autenticación del cliente, esquema de mensaje, reintentos o reglas de conflictos. Faltan el acuerdo con el responsable del portal y su contrato verificable. Los criterios siguientes delimitan lo documentado y las comprobaciones que deberá permitir ese acuerdo; las condiciones concretas del transporte siguen No verificadas. No se ejecutaron pruebas.

## HRL-72 [Tarea] [P03] Acordar la sincronización con el responsable del portal

Estado: Pendiente (evidencia: `src/services/api/workshop.ts::workshopApi` no expone intercambio remoto; `src-tauri/src/services/repairs.rs::RepairService::summary` solo devuelve técnico, nota pública y garantías para consumo local). `docs/contexto-completo-proyecto.md` §9 confirma publicación sin conexión; no se localizó un contrato aprobado del portal en docs. La existencia de un acuerdo externo es No verificada.
Prioridad: Alta, porque bloquea implementación, pruebas e integración del intercambio.
Puntos: 3
Sprint: 3 (9-13 nov de 2026). Asignación de P03 indicada por el usuario; no acredita que el acuerdo haya ocurrido en esas fechas.
Subtareas:

- HRL-115 [Servicio Rust] Preparar con el responsable del portal el contrato de datos, autorización y respuesta, partiendo del resumen público y de la separación entre guardado local y publicación; registrar los acuerdos antes de implementar.
- HRL-116 [Pruebas] Acordar ejemplos y casos de rechazo verificables contra el contrato, dejando explícitos los aspectos todavía sin definir.

Criterios de aceptación (fuentes: tarea HRL-72 de la fuente vigente, `docs/reparaciones-permisos.md`, contexto completo §9):

- Dado el responsable del portal y el alcance solicitado, cuando se acuerda el intercambio, entonces existe un contrato identificable que permite implementar y probar la comunicación sin elegir políticas por suposición.
- Dado el resumen público de reparación, cuando se delimita el contrato, entonces distingue datos públicos, notas internas y secretos que no deben publicarse.
- Dada ausencia de contrato o de definición de autorizaciones, cuando se evalúa la integración, entonces continúa pendiente y no se anuncia conectada.

## HRL-73 [Tarea] [S305] Publicar cambios autorizados

Estado: Pendiente (evidencia: `src-tauri/src/domain/repair_contracts.rs::operations` exige `publication.edit` para preferencias; `src-tauri/src/repositories/repairs.rs::RepairRepository::save` solo escribe `seguimiento_publicacion` y `modo_publicacion`). Esos controles locales corresponden a HRL-38 y no acreditan envío ni confirmación remota.
Prioridad: Alta, porque podría exponer información y requiere permisos auditables.
Puntos: 8
Sprint: 3 (por código S3xx).
Subtareas:

- HRL-117 [Composable/store] Incorporar estados de publicación real separados del guardado local, consumiendo el contrato que se acuerde en HRL-72 sin reutilizar el indicador de preferencia como éxito remoto.
- HRL-118 [Servicio Rust] Implementar la operación de publicación autorizada y su trazabilidad de acuerdo con el contrato; mantener secretos y notas internas fuera de los datos del cliente.
- HRL-119 [Pruebas] Cubrir publicación autorizada, rechazo por permiso y resultado no confirmado con el contrato aprobado, sin certificar una conexión mediante mocks solamente.

Criterios de aceptación (fuentes: `docs/reparaciones-permisos.md`, `docs/bitacora.md`, contexto completo §9):

- Dado contrato acordado y autorización vigente, cuando publica cambios, entonces diferencia el resultado remoto confirmado de las preferencias guardadas localmente y conserva trazabilidad.
- Dado un operador sin la facultad requerida, cuando solicita publicar, entonces el servicio rechaza sin exponer información.
- Dada una operación local sin conexión o sin confirmación remota, cuando guarda, entonces conserva el trabajo local sin presentar publicación exitosa.
- Dadas notas internas y contraseña del equipo, cuando construye datos para el cliente, entonces no las incluye.

## HRL-74 [Tarea] [S306] Probar el contrato de intercambio

Estado: Pendiente (evidencia: `tests/ipc-stores.test.ts` usa `mockIPC` para contratos Vue/Tauri; `tests/ui-restored.ts` devuelve fixtures locales; `src-tauri/src/services/tests.rs` prueba servicios SQLite). No se localizaron pruebas de contrato con un endpoint del portal. Las pruebas IPC existentes no son pruebas de intercambio externo.
Prioridad: Alta, porque valida privacidad y compatibilidad antes de conectar producción.
Puntos: 5
Sprint: 3 (por código S3xx).
Subtareas:

- HRL-120 [Servicio Rust] Preparar el adaptador de contrato comprobable en un entorno acordado con el portal; no definir un endpoint o credencial a partir de fixtures locales.
- HRL-121 [Pruebas] Añadir y registrar casos válidos, rechazo de autorización, contenido privado excluido y resultados fallidos o no confirmados según el contrato de HRL-72.

Criterios de aceptación (fuentes: tarea HRL-74, `docs/reparaciones-permisos.md`, `docs/bitacora.md`, límites del contexto completo §9):

- Dado contrato aprobado y entorno identificable, cuando se realizan las pruebas, entonces se documentan versión, datos de prueba y resultados reales distinguiéndolos del IPC simulado.
- Dada información interna o contraseña del equipo, cuando se comprueba el mensaje público, entonces se verifica que no salga hacia el portal.
- Dado rechazo de autorización o respuesta fallida prevista por el contrato, cuando se prueba, entonces no se registra publicación exitosa ni se altera la identidad del operador.

## HRL-75 [Tarea] [S401] Integrar la publicación con el servicio web

Estado: Pendiente (evidencia: `src-tauri/src/commands/mod.rs` delega operaciones locales a `BackendServices`; `src-tauri/src/services/mod.rs::BackendServices::new` no construye un cliente del portal; `src/services/api/workshop.ts::workshopApi` solo invoca comandos locales). No se encontró integración operativa con un servicio web. Requiere HRL-72, HRL-73 y HRL-74.
Prioridad: Alta, porque conecta la publicación de información privada del taller con un servicio externo.
Puntos: 8
Sprint: 4 (por código S4xx).
Subtareas:

- HRL-122 [Composable/store] Conectar el flujo de publicación al resultado del servicio integrado, con estados de espera, éxito confirmado y error sin afirmar una publicación por guardar preferencias.
- HRL-123 [Servicio Rust] Integrar el cliente del portal aprobado por constructor y respetar el comportamiento offline-first, permisos y trazabilidad; no introducir comunicación desde componentes.
- HRL-124 [Pruebas] Registrar una integración de extremo a extremo en el entorno acordado y sus rechazos, separando la aceptación remota de los mocks locales.

Criterios de aceptación (fuentes: tarea HRL-75, arquitectura objetivo, contexto funcional §1, `docs/bitacora.md`):

- Dado servicio del portal configurado según contrato y cambio autorizado, cuando se integra la publicación, entonces se comprueba el resultado remoto real y su trazabilidad.
- Dada ausencia de conexión o rechazo remoto, cuando opera el taller, entonces no se declara publicación exitosa y se mantiene la distinción con la persistencia local.
- Dado permiso revocado, cuando intenta publicar desde una UI previamente habilitada, entonces el backend deniega el uso de esa facultad.

## HRL-76 [Tarea] [A05] Ampliar controles de publicación

Estado: Pendiente (evidencia: `src/components/repairs/RepairWorkspace.vue` muestra controles locales; `src-tauri/src/domain/repair_contracts.rs::operations` autoriza cambios de modo/visibilidad y `RepairRepository::save` los persiste). HRL-38 cubre esos controles actuales; no se localizó una ampliación conectada al portal ni especificación de nuevos controles.
Prioridad: Media, porque amplía el uso de la publicación después de disponer del contrato y la integración segura.
Puntos: 5 (provisional hasta definir la ampliación).
Sprint: Backlog. Asignación de A05 indicada por el usuario, sin fecha comprometida.
Subtareas:

- HRL-125 [Vista] Precisar y construir los controles adicionales que se acuerden para el portal, distinguiéndolos de las opciones locales de HRL-38 y sin inventar destinos ni capacidades.
- HRL-126 [Pruebas] Verificar los controles definidos, sus permisos y la diferencia entre preferencia local y publicación confirmada; registrar lo que siga sin especificar.

Criterios de aceptación (fuentes: tarea HRL-76, `docs/reparaciones-permisos.md`, contexto completo §9):

- Dado alcance adicional acordado, cuando se implementan los controles, entonces su función y resultado se distinguen de las preferencias locales ya existentes.
- Dado acceso Limitado sin edición de publicación, cuando intenta cambiar un control protegido, entonces se rechaza sin conceder permisos por la interfaz.
- Dado un control que solo guarda preferencias, cuando confirma su cambio, entonces no informa publicación web realizada.

Discrepancia D-18: el backlog previo agrupaba el portal en HRL-66; la fuente vigente lo elimina y distribuye el alcance en HRL-71 a HRL-76. Los identificadores de subtareas antiguos HRL-71 en adelante ya no son referencias vigentes: se renumeran desde HRL-82 para evitar colisiones con los nuevos ítems base.
