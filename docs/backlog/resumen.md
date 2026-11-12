# Revisión actualizada del backlog de HRL CORE

Corte documental: 12 de noviembre de 2026, America/Cancun. **Única fuente de alcance vigente: `docs/backlog-base (1).md`**. `docs/jira-import (1).csv` corrobora las 80 claves y relaciones; sus títulos abreviados no sustituyen al Markdown. Se ignoran para determinar alcance las copias anteriores, incluido `docs/backlog-base.md`.

Hay **80 ítems base: 9 epics y 71 historias/tareas**, con claves HRL-1 a HRL-81 excepto HRL-66. Hay **53 subtareas técnicas HRL-82 a HRL-134**. Los cinco ítems todavía desconocidos de HRL-77 no se crean, estiman ni incluyen en denominadores. No se ejecutaron builds, suites, instalación, cámara, impresión ni sesiones con personal.

## Alcance, estados y estimaciones

- Hecho acredita implementación localizada por lectura estática, sin certificar pruebas o entrega. Los ítems sin cambio de alcance conservan la evidencia de la revisión anterior; las tareas añadidas se contrastaron con código y procedimientos reales en esta actualización.
- En progreso identifica implementación o pruebas presentes con trabajo faltante concreto.
- Pendiente indica ausencia del flujo funcional localizado. La configuración de publicación local de HRL-38 no acredita sincronización con el portal.
- No verificado indica evidencia que no puede acreditarse leyendo código. Incluye obligatoriamente validación nativa y participación de personal del taller.
- Los puntos Fibonacci representan alcance completo, no trabajo restante ni velocidad. Se suman historias/tareas una sola vez; epics y subtareas no añaden puntos independientes. Las historias de 13 puntos mantienen su propuesta de división.
- Los sprints son asignaciones del plan, no fechas de entrega demostradas. S2xx corresponde al 2, S3xx al 3 y S4xx al 4. P03 en Sprint 3 y A05 en Backlog son supuestos indicados por el usuario, no decisiones inferidas del código.

## Epics

| Epic | Documento | Estado agregado | Historias/tareas | Puntos |
| --- | --- | --- | ---: | ---: |
| HRL-1 | [Módulo de Usuarios, registro e inicio de sesión](epic-01-usuarios.md) | Hecho | 9 | 42 |
| HRL-11 | [Módulo de Registro de clientes y recepción de órdenes](epic-02-clientes.md) | Hecho | 12 | 58 |
| HRL-24 | [Módulo de Reparaciones](epic-03-reparaciones.md) | Hecho | 14 | 63 |
| HRL-39 | [Módulo de Bitácora / Auditoría](epic-04-bitacora.md) | En progreso | 8 | 39 |
| HRL-48 | [Módulo de Empresa](epic-05-empresa.md) | En progreso | 8 | 34 |
| HRL-57 | [Base técnica transversal](epic-06-base-tecnica.md) | En progreso | 6 | 53 |
| HRL-65 | [Pendientes futuros (Backlog)](epic-07-futuros.md) | En progreso | 4 | 32 |
| HRL-71 | [Sincronización con el portal web](epic-08-sincronizacion-portal.md) | Pendiente | 5 | 29 |
| HRL-77 | [Pruebas, instalación y entrega](epic-09-pruebas-instalacion-entrega.md) | En progreso | 5 | 36 |
| Total conocido | | | 71 | 386 |

HRL-64 se encuentra solo en [Pruebas, instalación y entrega](epic-09-pruebas-instalacion-entrega.md), bajo HRL-77, con sus mismos 13 puntos y Sprint 4. HRL-57 queda con seis tareas y 53 puntos. HRL-66 se eliminó del alcance; HRL-71 detalla su sustitución sin conservar otra historia de portal duplicada.

## Tabla por sprint

% Hecho = historias/tareas Hechas dividido entre historias/tareas del sprint. Las dos últimas columnas muestran explícitamente el aporte de los epics nuevos como **ítems/puntos**.

| Sprint | Fechas de 2026 | Ítems | Puntos | Hechos/ítems | % Hecho | Puntos Hechos/total | HRL-71 sincronización | HRL-77 pruebas/entrega |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| 1 | 26-30 oct | 10 | 55 | 7/10 | 70,0 % | 31/55 (56,4 %) | 0/0 | 0/0 |
| 2 | 2-6 nov | 11 | 63 | 10/11 | 90,9 % | 58/63 (92,1 %) | 0/0 | 1/5 |
| 3 | 9-13 nov | 15 | 64 | 11/15 | 73,3 % | 43/64 (67,2 %) | 3/16 | 1/5 |
| 4 | 16-20 nov | 30 | 167 | 22/30 | 73,3 % | 102/167 (61,1 %) | 1/8 | 3/26 |
| Backlog | Sin fecha | 5 | 37 | 1/5 | 20,0 % | 3/37 (8,1 %) | 1/5 | 0/0 |
| Total sprints 1–4 | | 66 | 349 | 50/66 | 75,8 % | 234/349 (67,0 %) | 4/24 | 5/36 |
| Total con Backlog | | 71 | 386 | 51/71 | 71,8 % | 237/386 (61,4 %) | 5/29 | 5/36 |

Las cinco tareas de sincronización están Pendientes. En HRL-77, HRL-78, HRL-79 y HRL-80 están En progreso por pruebas/configuración/procedimientos parciales localizados; HRL-64 y HRL-81 están No verificados. Ninguna tarea nueva se declara Hecha a partir de un PASS histórico o de fixtures.

Asignaciones preservadas:

- Sprint 1: HRL-2, HRL-3, HRL-4, HRL-7, HRL-10, HRL-40, HRL-58, HRL-59, HRL-60, HRL-62.
- Sprint 2: HRL-5, HRL-6, HRL-8, HRL-9, HRL-12, HRL-13, HRL-14, HRL-15, HRL-17, HRL-23, HRL-78.
- Sprint 3: HRL-16, HRL-19, HRL-20, HRL-21, HRL-25, HRL-26, HRL-27, HRL-28, HRL-29, HRL-30, HRL-36, HRL-72, HRL-73, HRL-74, HRL-79.
- Sprint 4: HRL-18, HRL-22, HRL-31, HRL-32, HRL-33, HRL-34, HRL-35, HRL-37, HRL-38, HRL-41, HRL-42, HRL-43, HRL-44, HRL-45, HRL-46, HRL-47, HRL-49, HRL-50, HRL-51, HRL-52, HRL-53, HRL-54, HRL-55, HRL-56, HRL-61, HRL-63, HRL-75, HRL-64, HRL-80, HRL-81.
- Backlog: HRL-67, HRL-68, HRL-69, HRL-70, HRL-76.

## Ítems con trabajo o verificación pendientes

| Ítem | Estado | Sprint | Puntos | Subtareas pendientes |
| --- | --- | --- | ---: | --- |
| HRL-42 | En progreso | 4 | 5 | HRL-82, HRL-83, HRL-84, HRL-85 |
| HRL-49 | En progreso | 4 | 5 | HRL-86, HRL-87 |
| HRL-59 | En progreso | 1 | 8 | HRL-88, HRL-89 |
| HRL-60 | En progreso | 1 | 8 | HRL-90, HRL-91, HRL-92 |
| HRL-61 | En progreso | 4 | 8 | HRL-93, HRL-94, HRL-95 |
| HRL-62 | En progreso | 1 | 8 | HRL-96, HRL-97 |
| HRL-63 | En progreso | 4 | 13 | HRL-98, HRL-99, HRL-100, HRL-101 |
| HRL-67 | No verificado | Backlog | 8 | HRL-105, HRL-106, HRL-107 |
| HRL-68 | Pendiente | Backlog | 8 | HRL-108, HRL-109, HRL-110, HRL-111 |
| HRL-70 | Pendiente | Backlog | 13 | HRL-112, HRL-113, HRL-114 |
| HRL-72 | Pendiente | 3 | 3 | HRL-115, HRL-116 |
| HRL-73 | Pendiente | 3 | 8 | HRL-117, HRL-118, HRL-119 |
| HRL-74 | Pendiente | 3 | 5 | HRL-120, HRL-121 |
| HRL-75 | Pendiente | 4 | 8 | HRL-122, HRL-123, HRL-124 |
| HRL-76 | Pendiente | Backlog | 5 | HRL-125, HRL-126 |
| HRL-64 | No verificado | 4 | 13 | HRL-102, HRL-103, HRL-104 |
| HRL-78 | En progreso | 2 | 5 | HRL-127, HRL-128 |
| HRL-79 | En progreso | 3 | 5 | HRL-129, HRL-130 |
| HRL-80 | En progreso | 4 | 8 | HRL-131, HRL-132 |
| HRL-81 | No verificado | 4 | 5 | HRL-133, HRL-134 |

Las subtareas de HRL-64 coordinan la regresión y la validación nativa; HRL-78 y HRL-79 completan casos específicos sin volver a estimar las mismas suites. HRL-62 define custodia; HRL-70 contempla automatización/portabilidad futura; HRL-80 prepara procedimiento y comprobación de instalación/recuperación local. No se implementan tres respaldos distintos. HRL-69 conserva su clave y estado de agrupación ya implementada, sin generar nuevas tareas de las funciones que solapan HRL-9/14/16/43.

## Discrepancias entre documentos y código

Las discrepancias D-01 a D-17 siguen siendo hallazgos de la revisión estática previa; no se presentan como nuevas ejecuciones. Los documentos recientes ya recogen parte de la transición. Las fuentes históricas pueden explicar evolución, pero no sustituyen la fuente actual de alcance ni prueban el resultado del árbol actual.

| ID | Documentación | Evidencia del código actual | Impacto / ítems |
| --- | --- | --- | --- |
| D-01 | `docs/accesos.md` y `docs/acceso-validacion.md`: recuperación todavía por definir | `src-tauri/src/services/access.rs::AccessService::generate_recovery_challenge` y `verify_recovery_signature`; `services/recovery.rs::RecoveryState::consume_at` | Recuperación con soporte ya implementada; HRL-5. El contexto completo la describe. |
| D-02 | `docs/acceso-temporal.md`: triggers y tabla de la migración 011 registran inicio/fin | `src-tauri/src/services/access.rs::AccessService::temporary` y `close` llaman a `AuditService` en transacción normalizada | No usar la descripción anterior para localizar auditoría vigente; HRL-9/40. |
| D-03 | Contexto funcional §4: Herramientas deshabilitado | `src/components/layout/AppHeader.vue` menú y `src/composables/useWorkshop.ts::receptionTool` | Consulta, actualización e historial ya conectados; HRL-16/69. |
| D-04 | Contexto funcional §5.2: tratamiento de contraseña de equipo pendiente | `src-tauri/src/services/crypto.rs::CryptoService::encrypt`/`decrypt`, `services/orders.rs::OrderService::get` | Ya existe cifrado recuperable; arquitectura §7 lo documenta; HRL-17/62. |
| D-05 | `docs/evidencias.md`: raíz local y carpeta por folio hexadecimal | `src-tauri/src/services/evidence.rs::EvidenceService::relative`; `db/connection.rs::Database::open_file` | Ruta nueva `assets-3fn/evidencias/<id interno>/<id foto>.<ext>`; no confundir los conjuntos al respaldar; HRL-19/62/70. |
| D-06 | `docs/papelera-ordenes.md`: visibilidad por último evento de `Historial_papelera` | `src-tauri/src/repositories/orders.rs::OrderRepository::trash` escribe `ordenes.eliminada_en` | La auditoría conserva eventos, pero ya no es el mecanismo de visibilidad; HRL-21. |
| D-07 | `docs/reparaciones-permisos.md`: ficha de la migración 006 y pagos dentro del guardado | `src-tauri/src/repositories/repairs.rs::RepairRepository::save` escribe tablas relacionadas; `domain/repair_contracts.rs::operations` rechaza pagos en seguimiento | Cobros por `LedgerService::record`; HRL-25 a HRL-38. |
| D-08 | `docs/bitacora.md`: identidad del registro y detalle de cambios nuevos | `src-tauri/src/services/orders.rs::OrderService::edit` omite email/entrega estimada en `changes`; `repositories/reports.rs::ReportRepository::logbook` omite entidad/ID del DTO global | Brecha concreta de trazabilidad, HRL-42. |
| D-09 | `docs/bitacora.md`: nombre de autor conservado en el evento | `src-tauri/src/repositories/audit.rs::AuditRepository::insert` guarda ID; `repositories/reports.rs::ReportRepository::logbook` hace JOIN con nombre actual | No se garantiza nombre histórico; falta decisión explícita, HRL-40/42/43. |
| D-10 | `docs/auditoria-empresa.md`: validación de correo/web | `src/composables/useCompany.ts::companyErrors` la aplica; `src-tauri/src/services/company.rs::CompanyService::save` no la repite | Puede eludirse mediante IPC directo; HRL-49. |
| D-11 | Auditoría de Empresa/accesos: configuración en `hrl.company.v1`, integración SQLite pendiente | `src/stores/useCompanyStore.ts::run` y `src-tauri/src/repositories/company.rs::CompanyRepository::save` | SQLite normalizado y logo por archivo ya implementados; HRL-49 a HRL-56. |
| D-12 | `docs/evidencias.md`: respaldo de DB y carpeta de fotos | `src-tauri/src/services/key_store.rs::OsKeyStore::open`, `services/company.rs::CompanyService::save_logo`, `db/connection.rs::Database::open_file` | También se necesitan logo y custodia de claves; DPAPI no acredita portabilidad; HRL-62/70. |
| D-13 | `AGENTS.md`: archivar 001-013; arquitectura §7 admite conservación transitoria | `src-tauri/src/lib.rs::run` registra las antiguas y siguen en la raíz de migrations | Retiro pendiente, no una migración de datos autorizada; HRL-59/63. |
| D-14 | `AGENTS.md`: data solo para catálogos de presentación | `src/data/orders.ts::initialOrders` contiene órdenes y datos de demostración, sin referencias de uso localizadas en src fuera de su definición | Reportado y excluido como fuente de verdad; retiro pendiente HRL-63. |
| D-15 | Documentos citan suites `.mjs` y resultados PASS anteriores | Archivos citados ausentes/eliminados en el árbol; `package.json::scripts.test:company` apunta a `tests/company.test.mjs`, ausente | No acreditar esas ejecuciones sobre este árbol; HRL-64. |
| D-16 | HRL-69 en Backlog futuro y contexto funcional: Herramientas pendiente | `AppHeader.vue` y `useWorkshop.ts::requestAccess`/`receptionTool` conectan las cuatro funciones | Cerrar agrupador; evitar duplicar HRL-9/14/16/43. |
| D-17 | Contexto funcional §7: Pinia no instalado y faltan stores/API | `package.json::dependencies.pinia`, `src/main.ts` registra `createPinia`, existen `src/stores/` y `src/services/api/` | Esas capas ya existen, aunque quedan infracciones en código anterior y utils; HRL-60/61/63. |
| D-18 | La fuente anterior concentraba el portal en HRL-66 y empleaba HRL-71 en adelante para subtareas | `RepairService::save` y `RepairRepository::save` siguen siendo locales; no hay cliente remoto localizado en `BackendServices::new` ni `workshopApi` | HRL-66 sale del alcance; HRL-71 a HRL-76 lo desglosan, todos Pendientes. Subtareas desde HRL-82. |
| D-19 | Documentos reportan PASS históricos y recorridos con IPC simulado | `tests/ui-restored.ts` contiene fixtures; `tests/phase4.test.ts` usa renderizador de prueba; `src-tauri/src/windowing.rs::tests` simula ventana | No acreditan instalación nativa ni aceptación del personal. HRL-64/81 No verificados; HRL-78/79/80 parciales. |

D-15 incluye una precisión: `docs/ventana-por-sesion.md` reporta un fallo DPAPI anterior y `docs/restauracion-diseno.md` reporta después 87 pruebas Rust aprobadas con DPAPI. Se registran ambos antecedentes sin deducir el resultado actual; no se ejecutó esa prueba aquí. `package.json::scripts.test:company` sigue apuntando a `tests/company.test.mjs`, ausente de tests.

La ausencia de precarga legacy en configuración tampoco demuestra que el ejecutable nunca inspeccione una base anterior: `src-tauri/src/schema_backup.rs::before_spanish_migration`, registrado por `lib.rs::run`, puede abrirla y realizar un respaldo previo a la 013. No se ejecutó.

## No verificado y qué falta

| Ítem o aspecto | Evidencia localizada | Necesario para verificar |
| --- | --- | --- |
| HRL-64 | Suites Rust/TS y prueba DPAPI `services/tests.rs::windows_key_custody_reopens_dpapi_key_without_storing_it_in_sqlite` | Ejecutar sobre versión identificada y registrar cámara, impresora, geometría y DPAPI en Windows. Fuera de esta revisión. |
| HRL-81 | Solo fixtures, mocks y procedimientos generales; no acta de personas del taller localizada | Recorrido acordado, versión instalada, entorno, participantes/roles autorizados y resultados reales. No inventar aceptación. |
| HRL-67 | Importador anterior usa pool/sesión legacy; `db/migrations.rs::apply` rechaza el esquema viejo | Confirmar origen y alcance de migración, copia autorizada y reglas de correspondencia/conflictos antes de escribir migración. |
| HRL-72 a HRL-76 | Persistencia local de modo/visibilidad; sin cliente, endpoint o pruebas remotas localizados | Contrato acordado con responsable, entorno y resultados de intercambio real. La tarea queda Pendiente; infraestructura externa y políticas no disponibles están No verificadas. |
| HRL-78/79 | Pruebas reales de revisión, permisos, evidencias, garantías y saldos; fixtures de UI | Completar huecos y registrar ejecución integrada del flujo normalizado. No usar pruebas legacy para acreditar 3FN. |
| HRL-80 | Configuración bundle, binario de desarrollo y rutas de DB/assets/claves; respaldo preventivo legacy | Instalador y procedimiento integral, instalación/reapertura y recuperación en Windows. Un ejecutable debug o configuración no acreditan entrega instalada. |
| HRL-62/70 | Código DPAPI y recursos externos a DB | Estrategia aprobada y restauración conjunta con identidad Windows conocida; no suponer portabilidad por copiar el archivo. |
| HRL-68 | Validadores operativos de fotos rechazan video/documentos | Especificar formatos, tamaños y consulta de archivos; después implementar y verificar el flujo. |
| Historias Hechas por inspección | Implementación localizada y evidencias citadas | Ejecución/aceptación posterior si se exige cierre QA. Cámara e impresión físicas no se infieren del código. |

## Carga del Sprint 4 y rebalanceo

**Sprint 4 supera el 40 %:** concentra **167/349 = 47,9 %** de los puntos programados y **167/386 = 43,3 %** incluso contando Backlog. Tiene 30 historias/tareas conocidas. Los cinco ítems faltantes de HRL-77 podrían aumentar esta carga; no se les asignaron puntos o sprint por suposición.

Como propuesta, separar HRL-63 (13), HRL-64 (13) y HRL-81 (5) en un cierre posterior: Sprint 4 quedaría con 136/349 = 39,0 % y el cierre con 31/349 = 8,9 %. No es un cambio aplicado, ni un compromiso de capacidad o fechas. Los controles de seguridad, dinero y auditoría que bloqueen una entrega deben verificarse antes de usarla; agrupar aceptación final no autoriza omitirlos.

No mover trabajo a sprints pasados para aparentar entregas. Primero reestimar esfuerzo pendiente de los ítems En progreso y recoger los cinco faltantes; los 386 puntos describen alcance total, no esfuerzo restante. Acordar HRL-72 antes de implementar/probar la integración; HRL-38 no sustituye HRL-73/75. Las pruebas con personal requieren coordinación real y no se programan con participantes inventados.

## Preparación del proyecto Jira nuevo

El usuario pidió un proyecto nuevo llamado **HRL registro**, con clave **HRL** y numeración desde **HRL-1**. No se actualizó el proyecto anterior con esta revisión ni se reinició su contador. El conector no ofrece crear/renombrar proyectos y el intento de acceso administrativo al navegador agotó su autorización; el proyecto previo se mantiene intacto.

Se prepara una carga derivada de estos documentos, no una importación de la copia anterior. Hay un hueco intencional en HRL-66 y un padre posterior (HRL-64 → HRL-77). Por eso, crear simplemente 80 ítems en orden no garantiza que sus números coincidan con las 80 referencias. La carga con claves explícitas requiere validar el importador y sus campos administrativos; no se consume HRL-66 creando una tarea ficticia. Las subtareas se cargan después de todos los ítems base.

La correspondencia y el reporte de la sincronización anterior se conservarán como históricos, no como prueba de que el proyecto solicitado ya exista. Falta disponibilidad de la clave HRL y acceso administrativo para crear/verificar el destino. No se elimina el proyecto anterior ni se liberan claves por una acción no autorizada.

## Alcance incompleto y verificación documental

**HRL-77 tiene cinco ítems faltantes por incorporar.** No se inventaron claves, nombres, subtareas, puntos ni criterios de esos ítems.

La lectura estructural verifica: 80 claves base únicas, nueve epics, 71 historias/tareas, HRL-64 solo bajo HRL-77, ausencia de HRL-66 como ítem vigente, códigos originales conservados, 53 subtareas únicas HRL-82 a HRL-134 y criterios Dado/Cuando/Entonces de 3 a 6 por historia/tarea. Es una comprobación de Markdown/CSV; no son suites ni builds del proyecto.
