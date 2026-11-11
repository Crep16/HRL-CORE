# Revisión del backlog de HRL CORE

Corte: 11 de noviembre de 2026, zona America/Cancun. Fuente de claves, títulos y sprints: `docs/backlog-base.md`. Fuente de estados: inspección del árbol de trabajo real, incluidos archivos nuevos/modificados sin commit, en `src/`, `src-tauri/src/` y `src-tauri/migrations/`. Fuente de aceptación: reglas funcionales citadas en cada historia.

Se crearon siete documentos de epic y este resumen. Se conservaron HRL-1 a HRL-70: siete epics y 63 historias/tareas. Se añadieron 36 subtareas, HRL-71 a HRL-106, solo donde se identificó trabajo faltante. No se cambiaron código fuente, migraciones, pruebas, configuración ni documentos fuera de `docs/backlog/`. No se ejecutaron builds, suites, la aplicación ni operaciones sobre bases del taller.

## Cómo leer los estados y puntos

- Hecho: se localizó implementación del alcance por lectura estática, con cadena de llamadas y reglas pertinentes. No significa aceptación ejecutada, despliegue ni validación nativa.
- En progreso: hay implementación, pero la inspección identifica una brecha concreta respecto del alcance o arquitectura.
- Pendiente: no se localizó el flujo funcional y la ruta actual confirma que no lo ofrece. No implica ausencia de otro proyecto externo desconocido.
- No verificado: falta evidencia que no puede obtenerse leyendo el código, o falta un origen/contrato confirmado para comprobar el alcance.

Las estimaciones Fibonacci de las historias/tareas expresan **alcance completo**, incluidas las hechas. No son puntos pendientes ni velocidad observada. Los epics muestran sumas y no agregan puntos adicionales. Las subtareas no se vuelven a sumar: las divisiones de historias de 13 puntos reparten esos mismos 13. Un epic supera 8 por agregación, sin ser una historia que deba dividirse otra vez.

El porcentaje Hecho de la tabla es `historias/tareas Hecho ÷ historias/tareas del sprint × 100`, sin epics ni subtareas. Se añade porcentaje por puntos para distinguir cobertura de cantidad y tamaño. No se infieren fechas de entrega a partir de las fechas asignadas al sprint.

## Archivos por epic

| Epic | Documento | Estado agregado | Hijas | Puntos |
| --- | --- | --- | ---: | ---: |
| HRL-1 | [Usuarios](epic-01-usuarios.md) | Hecho por inspección | 9 | 42 |
| HRL-11 | [Clientes y recepción](epic-02-clientes.md) | Hecho por inspección | 12 | 58 |
| HRL-24 | [Reparaciones](epic-03-reparaciones.md) | Hecho por inspección | 14 | 63 |
| HRL-39 | [Bitácora](epic-04-bitacora.md) | En progreso | 8 | 39 |
| HRL-48 | [Empresa](epic-05-empresa.md) | En progreso | 8 | 34 |
| HRL-57 | [Base técnica](epic-06-base-tecnica.md) | En progreso | 7 | 66 |
| HRL-65 | [Pendientes futuros](epic-07-futuros.md) | En progreso como agrupador | 5 | 45 |
| Total | | | 63 | 347 |

Los estados agregados heredan la evidencia de sus hijas; no se trata la palabra «Pendientes» del título de HRL-65 como prueba de estado. HRL-69 ya está implementado aunque figure en ese agrupador.

## Tabla por sprint

| Sprint | Fechas de 2026 | Ítems (sin epics/subtareas) | Puntos | Hechos / ítems | % Hecho | Puntos Hechos / puntos | % Hecho por puntos | % de los 302 puntos de sprints 1–4 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 26-30 oct | 10 | 55 | 7 / 10 | 70,0 % | 31 / 55 | 56,4 % | 18,2 % |
| 2 | 2-6 nov | 10 | 58 | 10 / 10 | 100,0 % | 58 / 58 | 100,0 % | 19,2 % |
| 3 | 9-13 nov | 11 | 43 | 11 / 11 | 100,0 % | 43 / 43 | 100,0 % | 14,2 % |
| 4 | 16-20 nov | 27 | 146 | 22 / 27 | 81,5 % | 102 / 146 | 69,9 % | 48,3 % |
| Total sprints 1–4 | | 58 | 302 | 50 / 58 | 86,2 % | 234 / 302 | 77,5 % | 100,0 % |
| Backlog | Sin fecha | 5 | 45 | 1 / 5 | 20,0 % | 3 / 45 | 6,7 % | No aplica |
| Total general | | 63 | 347 | 51 / 63 | 81,0 % | 237 / 347 | 68,3 % | No aplica |

Ítems de cada sprint, conservados de la base:

- Sprint 1: HRL-2, HRL-3, HRL-4, HRL-7, HRL-10, HRL-40, HRL-58, HRL-59, HRL-60 y HRL-62.
- Sprint 2: HRL-5, HRL-6, HRL-8, HRL-9, HRL-12, HRL-13, HRL-14, HRL-15, HRL-17 y HRL-23.
- Sprint 3: HRL-16, HRL-19, HRL-20, HRL-21, HRL-25, HRL-26, HRL-27, HRL-28, HRL-29, HRL-30 y HRL-36.
- Sprint 4: HRL-18, HRL-22, HRL-31, HRL-32, HRL-33, HRL-34, HRL-35, HRL-37, HRL-38, HRL-41, HRL-42, HRL-43, HRL-44, HRL-45, HRL-46, HRL-47, HRL-49, HRL-50, HRL-51, HRL-52, HRL-53, HRL-54, HRL-55, HRL-56, HRL-61, HRL-63 y HRL-64.
- Backlog: HRL-66, HRL-67, HRL-68, HRL-69 y HRL-70.

## Brechas de implementación localizadas

| Ítem | Estado | Qué falta | Subtareas |
| --- | --- | --- | --- |
| HRL-42 | En progreso | Diferencias de email/entrega estimada e identidad del registro en detalle global de auditoría | HRL-71 a HRL-74 |
| HRL-49 | En progreso | Repetir validación de correo/web en servicio Rust; hoy solo existe en frontend | HRL-75 y HRL-76 |
| HRL-59 | En progreso | Inyectar todos los repositorios colaboradores y cerrar la coexistencia del backend anterior con HRL-63 | HRL-77 y HRL-78 |
| HRL-60 | En progreso | Eliminar IPC directo anterior y trasladar reglas financieras fuera de utils | HRL-79 a HRL-81 |
| HRL-61 | En progreso | Pantallas restantes en views y lógica de cámara/cláusulas en viewmodels | HRL-82 a HRL-84 |
| HRL-62 | En progreso | Estrategia de recuperación conjunta y verificación de custodia DPAPI | HRL-85 y HRL-86 |
| HRL-63 | En progreso | Retirar comandos, plugin, migraciones registradas y archivos anteriores sin migrar datos por suposición | HRL-87 a HRL-90 |
| HRL-64 | No verificado | Resultados de ejecución de suites y hardware nativo | HRL-91 a HRL-93 |
| HRL-66 | Pendiente | Portal y publicación efectiva con contrato aprobado | HRL-94 a HRL-96 |
| HRL-67 | No verificado | Datos origen y contrato de importación a 3FN confirmados | HRL-97 a HRL-99 |
| HRL-68 | Pendiente | Flujo de video/documentos y política de tipos/tamaños | HRL-100 a HRL-103 |
| HRL-70 | Pendiente | Respaldo/restauración integral y traslado de claves entre instalaciones | HRL-104 a HRL-106 |

## Discrepancias entre documentación y código

Las referencias a documentos antiguos se señalan como desactualización o coexistencia, sin cambiar sus reglas ni reinterpretar fechas de entrega. El contexto completo reciente sí recoge varias de estas transiciones.

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

Precisión adicional comprobada: `src-tauri/src/schema_backup.rs::before_spanish_migration`, registrado por `lib.rs::run`, puede abrir una base legacy y hacer `VACUUM INTO` antes de la 013. La ausencia de precarga en `tauri.conf.json` no prueba que el ejecutable nunca inspeccione la base anterior. No se ejecutó ese mecanismo en esta revisión.

## No verificado y evidencia necesaria

| Ítem o aspecto | Lo que sí se comprobó | Lo que falta para verificar |
| --- | --- | --- |
| HRL-64 completo | Existen suites Rust/TS y casos de ventana/DPAPI | Ejecutar las suites sobre el árbol exacto y comprobar cámara, impresión, geometría y DPAPI en aplicación Windows; registrar comandos, entorno, resultado y fallos. No autorizado en esta tarea. |
| HRL-67 completo | Importador anterior usa otro pool; cargador 3FN rechaza esquema legacy | Confirmar datos de origen, disponer de copia autorizada y contrato de migración/conflictos; después comprobar conservación y resultado en destino. No escribir migración de datos antes. |
| HRL-10, HRL-19, HRL-22, HRL-37, HRL-50 y HRL-53: componente nativo | Rutas, preparación y manejo de errores están implementados | Recorrido de ventana, cámara/portapapeles concedidos y denegados e impresión real. Se concentra en HRL-93; no se confunde con los estados estáticos de estas historias. |
| HRL-62: DPAPI y recuperación | Código cifra la clave externa y contiene prueba de reapertura bajo Windows | Resultado con la identidad real, restauración del conjunto y estrategia aprobada para pérdida de custodia. El fallo DPAPI histórico de `docs/ventana-por-sesion.md` no se volvió a reproducir ni se declara resuelto. |
| HRL-66: infraestructura externa y reglas del portal | La ruta del repositorio solo guarda preferencias | Especificación de acceso/publicación y repositorio/contrato del portal si existe. No se infiere inexistencia de servicios fuera del repo. |
| HRL-68: nuevos formatos | Validadores operativos rechazan video/documento; SQL contempla tipos | Formatos, límites, tratamiento de archivos y consulta aprobados; después flujo funcional y pruebas. No extender por suposición el máximo de fotografía. |
| HRL-70: portabilidad/automatización | Existe respaldo preventivo legacy, sin flujo integral normalizado | Estrategia de claves, consistencia, paquete, autorización, frecuencia/retención y dos instalaciones conocidas para restaurar. |
| Todas las historias Hechas: resultado ejecutado | Implementación y pruebas citadas se localizaron por lectura | Una ejecución posterior, si se quiere usar «Hecho» como cierre QA o entrega real. Esta revisión no acredita que una prueba pase ni que se haya entregado en el sprint fechado. |

## Rebalanceo recomendado

Sprint 4 concentra **146/302 = 48,3 %** de los puntos de los cuatro sprints y **146/347 = 42,1 %** incluso incluyendo el Backlog. Supera el 40 % con ambos denominadores. Por cantidad concentra 27 de las 58 historias/tareas programadas. Las fechas y asignaciones originales se conservan en todos los archivos.

No se deben mover historias a sprints ya concluidos para aparentar entregas históricas: el código actual no acredita fecha de terminación. Tampoco hay datos de capacidad o velocidad para prometer que un número de puntos cabe en cinco días.

Como propuesta de planificación, separar el cierre técnico HRL-63 (13) y HRL-64 (13) en un sprint adicional de cierre posterior al 20 de noviembre. Sprint 4 quedaría con 120/302 = 39,7 % y el cierre con 26/302 = 8,6 %. Esto es una recomendación documental, no un cambio aplicado al backlog ni una fecha comprometida. No aplazaría las comprobaciones de seguridad y dinero que bloqueen el uso: HRL-91/92 deben ejecutarse para cada entrega afectada; el cierre agrupa el retiro final y la aceptación integral.

Antes de fijar esa propuesta, reestimar trabajo restante. Hay 24 puntos de alcance completo todavía no Hechos en Sprint 1 (HRL-59/60/62) y 44 en Sprint 4 (HRL-42/49/61/63/64): **68 puntos de alcance implicado**, no 68 puntos de esfuerzo restante. Resolver primero HRL-42, HRL-49 y custodia HRL-62 por auditoría, validación de datos y secretos; coordinar HRL-59/60/61 antes de cerrar HRL-63, y verificar con HRL-64. Las divisiones de 13 puntos propuestas en los epics permiten ejecutar entregas revisables de 3 o 5 puntos.

HRL-69 solapa funciones ya estimadas en HRL-9/14/16/43; los 3 puntos se mantienen en la tabla para conservar el alcance original, pero no deben generar nueva capacidad ni implementación al planificar. HRL-62 define custodia; HRL-70 implementa automatización/portabilidad futura. Mantener esa frontera evita contar dos veces el mismo trabajo.

## Registro de entrega documental

Se hicieron commits descriptivos por cada epic. Un commit final agrupa este resumen, criterios agregados de los epics y la corrección del total de Clientes a 58. Git ignora `docs/`; se incluyeron únicamente los archivos solicitados mediante rutas explícitas, sin cambiar `.gitignore` ni añadir cambios ajenos.

La revisión de documentos comprueba cobertura de claves, unicidad y continuidad de HRL-71 a HRL-106, de 3 a 6 criterios por historia/tarea y de 2 a 5 subtareas para ítems no Hechos. Son comprobaciones del entregable Markdown mediante lectura, no suites del proyecto.
