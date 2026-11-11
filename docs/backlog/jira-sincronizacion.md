# Sincronización del backlog con Jira

Fuente: revisión estática del 11 de noviembre de 2026, commit documental `982f62f`. [Resumen completo en Jira: HRL-161](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-161).

La sincronización quedó completada: **107 ítems nuevos** (7 epics, 56 historias, 7 tareas técnicas, 36 subtareas y una tarea documental de resumen sin puntos). Se marcaron **43 ítems anteriores como reemplazados**, conservando descripciones e historial, retirándolos de sprints y cerrándolos por sustitución. Cada uno tiene un vínculo con el resumen. Ese cierre no acredita implementación.

Jira asignó claves nuevas. Las referencias HRL-1 a HRL-106 de los documentos no se renumeraron. El [mapa JSON](jira-correspondencia.json) distingue referencias documentales, claves nuevas y claves del backlog anterior.

## Verificación

| Comprobación | Resultado |
| --- | --- |
| Referencias documentales mapeadas | 106, sin faltantes |
| Jerarquía | 7 epics, 63 historias/tareas y 36 subtareas, sin diferencias de padre |
| Estimaciones | 347 puntos en historias/tareas; epics, subtareas y resumen sin estimación adicional |
| Prioridades | Campos High/Medium/Low y etiquetas equivalentes, sin diferencias |
| Estados | Coinciden con el mapeo de la revisión estática |
| Sprints | Asignaciones coinciden; Backlog futuro sin sprint |
| Anteriores | 43 reemplazados, cerrados por sustitución y fuera de sprints |
| Builds/pruebas | No se ejecutaron |

El resumen está bajo el epic técnico y no agrega puntos al alcance. Incluye discrepancias, no verificados, porcentajes y propuesta de rebalanceo.

## Estados y consultas

| Documento | Jira | Alcance |
| --- | --- | --- |
| Hecho | Terminado + `hecho-inspeccion` | Implementación localizada por lectura; sin certificar ejecución o validación nativa |
| En progreso | En curso | Brecha concreta pendiente |
| Pendiente | Por hacer | Capacidad o subtarea pendiente |
| No verificado | En prueba + `no-verificado` | Falta evidencia; no afirma ejecución de pruebas |

Jira no ofrece un estado nativo No verificado. El significado se conserva en descripción y etiqueta. Los anteriores llevan `backlog-reemplazado` y no deben incluirse en métricas del nuevo alcance.

JQL del backlog vigente: `project = HRL AND labels = "backlog-detallado-2026-11"`. Para puntos y avance de las 63 historias/tareas, excluir epics, subtareas y la etiqueta `resumen-backlog`.

## Epics

| Referencia documental | Epic Jira | Módulo |
| --- | --- | --- |
| `HRL-1` | [HRL-55](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-55) | Módulo de Usuarios, registro e inicio de sesión |
| `HRL-11` | [HRL-65](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-65) | Módulo de Registro de clientes y recepción de órdenes |
| `HRL-24` | [HRL-78](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-78) | Módulo de Reparaciones |
| `HRL-39` | [HRL-93](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-93) | Módulo de Bitácora / Auditoría |
| `HRL-48` | [HRL-106](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-106) | Módulo de Empresa |
| `HRL-57` | [HRL-117](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-117) | Base técnica transversal |
| `HRL-65` | [HRL-142](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-142) | Pendientes futuros (Backlog) |

## Sprints

| Sprint | ID Jira | Fechas de 2026 | Ítems base | Puntos | Hechos por inspección |
| --- | ---: | --- | ---: | ---: | ---: |
| 1 | 104 | 26-30 oct | 10 | 55 | 7 |
| 2 | 105 | 2-6 nov | 10 | 58 | 10 |
| 3 | 106 | 9-13 nov | 11 | 43 | 11 |
| 4 | 107 | 16-20 nov | 27 | 146 | 22 |
| Backlog | Sin sprint | Sin fecha | 5 | 45 | 1 |

Se actualizaron las metas a Cimientos y acceso; Usuarios completos y registro de clientes; Evidencias, papelera y reparaciones; Dinero, bitácora, empresa y cierre. Se conservaron nombres, fechas y estados de ciclo: Sprint 1 activo y 2–4 futuros, tal como estaban en Jira. El estado de ciclo no se interpreta como fecha de entrega. No se inició ni cerró ningún sprint ni se aplicó el rebalanceo propuesto.

## Correspondencia completa

La primera columna identifica documentos. Los enlaces de la segunda identifican ítems Jira nuevos.

| Referencia documental | Jira real | Tipo | Padre documental | Estado documental | Título |
| --- | --- | --- | --- | --- | --- |
| `HRL-1` | [HRL-55](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-55) | Epic | Sin padre | Hecho | Módulo de Usuarios, registro e inicio de sesión |
| `HRL-2` | [HRL-56](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-56) | Historia | `HRL-1` | Hecho | Configuración inicial: definir clave maestra y crear primer administrador (envío atómico) |
| `HRL-3` | [HRL-57](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-57) | Historia | `HRL-1` | Hecho | Inicio de sesión con usuario y contraseña (Argon2, solo cuentas activas) |
| `HRL-4` | [HRL-58](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-58) | Historia | `HRL-1` | Hecho | Cierre de sesión e invalidación de datos de la sesión anterior |
| `HRL-5` | [HRL-59](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-59) | Historia | `HRL-1` | Hecho | Recuperación de clave maestra con soporte (desafío de 15 min, firma Ed25519) |
| `HRL-6` | [HRL-60](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-60) | Historia | `HRL-1` | Hecho | Gestión de usuarios: crear, editar, activar/desactivar y cambiar contraseña |
| `HRL-7` | [HRL-61](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-61) | Historia | `HRL-1` | Hecho | Niveles de acceso y permisos por apartado y acción (completo, limitado, personalizado) |
| `HRL-8` | [HRL-62](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-62) | Historia | `HRL-1` | Hecho | Protección del último administrador activo (servicio + triggers) |
| `HRL-9` | [HRL-63](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-63) | Historia | `HRL-1` | Hecho | Acceso temporal: autorización de otra persona con motivo, limitada a apartado u orden |
| `HRL-10` | [HRL-64](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-64) | Historia | `HRL-1` | Hecho | Perfil de ventana según estado (acceso, configuración inicial, taller) |
| `HRL-11` | [HRL-65](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-65) | Epic | Sin padre | Hecho | Módulo de Registro de clientes y recepción de órdenes |
| `HRL-12` | [HRL-66](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-66) | Historia | `HRL-11` | Hecho | Crear orden con cliente, equipo, falla y trabajo solicitado (folio asignado por backend) |
| `HRL-13` | [HRL-67](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-67) | Historia | `HRL-11` | Hecho | Ficha de recepción: cliente, artículo, antecedentes, precio y observaciones |
| `HRL-14` | [HRL-68](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-68) | Historia | `HRL-11` | Hecho | Listado lateral con búsqueda (Todo/Orden/Cliente/Serie), paginación y navegación entre órdenes |
| `HRL-15` | [HRL-69](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-69) | Historia | `HRL-11` | Hecho | Editar orden con control de revisión |
| `HRL-16` | [HRL-70](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-70) | Historia | `HRL-11` | Hecho | Consulta por folio o ID desde Herramientas |
| `HRL-17` | [HRL-71](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-71) | Historia | `HRL-11` | Hecho | Contraseña del equipo cifrada (AES-256-GCM), visible solo bajo solicitud expresa |
| `HRL-18` | [HRL-72](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-72) | Historia | `HRL-11` | Hecho | Abono inicial al crear la orden (con permiso de cobro) |
| `HRL-19` | [HRL-73](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-73) | Historia | `HRL-11` | Hecho | Evidencias fotográficas por archivo o cámara (JPEG/PNG/WebP, máx. 20 MiB) |
| `HRL-20` | [HRL-74](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-74) | Historia | `HRL-11` | Hecho | Descripción, retiro y restauración de evidencias |
| `HRL-21` | [HRL-75](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-75) | Historia | `HRL-11` | Hecho | Papelera: eliminar y restaurar órdenes conservando historial |
| `HRL-22` | [HRL-76](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-76) | Historia | `HRL-11` | Hecho | Nota de entrada imprimible |
| `HRL-23` | [HRL-77](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-77) | Historia | `HRL-11` | Hecho | Guardas de borrador al cambiar de apartado o cerrar sesión |
| `HRL-24` | [HRL-78](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-78) | Epic | Sin padre | Hecho | Módulo de Reparaciones |
| `HRL-25` | [HRL-79](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-79) | Historia | `HRL-24` | Hecho | Abrir reparación sobre la orden seleccionada con contexto de cliente y equipo |
| `HRL-26` | [HRL-80](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-80) | Historia | `HRL-24` | Hecho | Asignar técnico y registrar notas internas y públicas |
| `HRL-27` | [HRL-81](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-81) | Historia | `HRL-24` | Hecho | Etapas y avances con fecha (completar etapa no marca entrega) |
| `HRL-28` | [HRL-82](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-82) | Historia | `HRL-24` | Hecho | Registro de refacciones utilizadas con costo |
| `HRL-29` | [HRL-83](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-83) | Historia | `HRL-24` | Hecho | Registro de garantías (cobertura y días) |
| `HRL-30` | [HRL-84](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-84) | Historia | `HRL-24` | Hecho | Control de revisión y permisos por operación en el guardado |
| `HRL-31` | [HRL-85](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-85) | Historia | `HRL-24` | Hecho | Panel de pagos: total, abonado, pendiente, saldo a favor y movimientos |
| `HRL-32` | [HRL-86](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-86) | Historia | `HRL-24` | Hecho | Registrar abonos (importe positivo dentro del pendiente) |
| `HRL-33` | [HRL-87](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-87) | Historia | `HRL-24` | Hecho | Correcciones de abonos (con motivo, permiso de acceso completo) |
| `HRL-34` | [HRL-88](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-88) | Historia | `HRL-24` | Hecho | Devolución del saldo a favor (método, motivo y permiso propio) |
| `HRL-35` | [HRL-89](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-89) | Historia | `HRL-24` | Hecho | Reintento seguro de operaciones financieras (ID de solicitud y huella) |
| `HRL-36` | [HRL-90](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-90) | Historia | `HRL-24` | Hecho | Registro de entrega independiente de pago y avance técnico |
| `HRL-37` | [HRL-91](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-91) | Historia | `HRL-24` | Hecho | Nota de salida con resumen técnico público y garantías |
| `HRL-38` | [HRL-92](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-92) | Historia | `HRL-24` | Hecho | Preferencias de publicación (solo guardado local, sin portal web) |
| `HRL-39` | [HRL-93](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-93) | Epic | Sin padre | En progreso | Módulo de Bitácora / Auditoría |
| `HRL-40` | [HRL-94](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-94) | Historia | `HRL-39` | Hecho | Servicio de auditoría dentro de la misma transacción del cambio |
| `HRL-41` | [HRL-95](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-95) | Historia | `HRL-39` | Hecho | Bitácora global con filtros por texto, autor, operación, orden y fechas |
| `HRL-42` | [HRL-96](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-96) | Historia | `HRL-39` | En progreso | Detalle de cambios (valores anterior y nuevo, sin contraseñas) |
| `HRL-43` | [HRL-101](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-101) | Historia | `HRL-39` | Hecho | Historial de una orden |
| `HRL-44` | [HRL-102](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-102) | Historia | `HRL-39` | Hecho | Finanzas: filtros por folio, cliente, clasificación, periodo y alcance |
| `HRL-45` | [HRL-103](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-103) | Historia | `HRL-39` | Hecho | Clasificación de órdenes (completadas, pagadas sin entregar, entregadas con pendiente, sin liquidar) |
| `HRL-46` | [HRL-104](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-104) | Historia | `HRL-39` | Hecho | Seis indicadores financieros agrupados por moneda |
| `HRL-47` | [HRL-105](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-105) | Historia | `HRL-39` | Hecho | Registro de gastos directos (descripción, importe, motivo y autorización) |
| `HRL-48` | [HRL-106](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-106) | Epic | Sin padre | En progreso | Módulo de Empresa |
| `HRL-49` | [HRL-107](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-107) | Historia | `HRL-48` | En progreso | Datos generales: nombre, identificación fiscal, contacto, dirección y horario |
| `HRL-50` | [HRL-110](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-110) | Historia | `HRL-48` | Hecho | Logo y su uso en documentos |
| `HRL-51` | [HRL-111](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-111) | Historia | `HRL-48` | Hecho | Notas impresas: mensajes de entrada/salida, firma y referencia |
| `HRL-52` | [HRL-112](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-112) | Historia | `HRL-48` | Hecho | Cláusulas: título, texto, orden y aplicación a entrada/salida |
| `HRL-53` | [HRL-113](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-113) | Historia | `HRL-48` | Hecho | Hoja de impresión: carta, ticket 80/60 mm, orientación y copias |
| `HRL-54` | [HRL-114](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-114) | Historia | `HRL-48` | Hecho | Folios y garantías: prefijo, siguiente número y días sugeridos |
| `HRL-55` | [HRL-115](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-115) | Historia | `HRL-48` | Hecho | Región e impuestos: país, zona horaria, moneda e impuesto incluido |
| `HRL-56` | [HRL-116](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-116) | Historia | `HRL-48` | Hecho | Edición como borrador con control de revisión, vista previa y auditoría |
| `HRL-57` | [HRL-117](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-117) | Epic | Sin padre | En progreso | Base técnica transversal |
| `HRL-58` | [HRL-118](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-118) | Tarea | `HRL-57` | Hecho | Esquema SQLite 3FN y migraciones (001 y 002) |
| `HRL-59` | [HRL-119](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-119) | Tarea | `HRL-57` | En progreso | Arquitectura backend en capas: commands, services, repositories, db |
| `HRL-60` | [HRL-122](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-122) | Tarea | `HRL-57` | En progreso | Frontend MVVM con stores Pinia y capa services/api |
| `HRL-61` | [HRL-126](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-126) | Tarea | `HRL-57` | En progreso | Mover vistas restantes de components/ a views/ y adelgazar WorkshopApp |
| `HRL-62` | [HRL-130](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-130) | Tarea | `HRL-57` | En progreso | Custodia de claves con DPAPI y estrategia de respaldo (base + assets + claves) |
| `HRL-63` | [HRL-133](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-133) | Tarea | `HRL-57` | En progreso | Retirar código legacy (comandos anteriores, plugin SQL 001-013, archivos Legacy) |
| `HRL-64` | [HRL-138](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-138) | Tarea | `HRL-57` | No verificado | Pruebas frontend y Rust, y validación nativa (cámara, impresora, DPAPI) |
| `HRL-65` | [HRL-142](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-142) | Epic | Sin padre | En progreso | Pendientes futuros (Backlog) |
| `HRL-66` | [HRL-143](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-143) | Historia | `HRL-65` | Pendiente | Portal web del cliente y publicación de avances |
| `HRL-67` | [HRL-147](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-147) | Historia | `HRL-65` | No verificado | Importación de datos desde la base legacy |
| `HRL-68` | [HRL-151](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-151) | Historia | `HRL-65` | Pendiente | Carga de video y documentos como evidencia |
| `HRL-69` | [HRL-156](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-156) | Historia | `HRL-65` | Hecho | Herramientas (acceso temporal, consulta por referencia, actualización e historial) |
| `HRL-70` | [HRL-157](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-157) | Historia | `HRL-65` | Pendiente | Respaldo y migración automática entre instalaciones |
| `HRL-71` | [HRL-97](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-97) | Subtask | `HRL-42` | Pendiente | Completar en `OrderService::edit` las diferencias de email y entrega estimada, y revisar cobertura de cada campo editable sin registrar el secreto del equipo. |
| `HRL-72` | [HRL-98](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-98) | Subtask | `HRL-42` | Pendiente | Transportar entidad e identificador afectado desde la consulta de auditoría en el contrato de detalle; no alterar el libro inmutable existente. |
| `HRL-73` | [HRL-99](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-99) | Subtask | `HRL-42` | Pendiente | Mostrar la identidad del registro y los nuevos campos de diferencia en el detalle de Bitácora. |
| `HRL-74` | [HRL-100](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-100) | Subtask | `HRL-42` | Pendiente | Cubrir edición de email/entrega estimada, comparación anterior/nuevo y ausencia de contraseñas en todos los eventos y DTO públicos de auditoría. |
| `HRL-75` | [HRL-108](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-108) | Subtask | `HRL-49` | Pendiente | Aplicar en `CompanyService::save` las reglas documentadas de correo y sitio web, admitiendo campos opcionales vacíos como el formulario actual. |
| `HRL-76` | [HRL-109](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-109) | Subtask | `HRL-49` | Pendiente | Cubrir llamadas directas con correo/web inválidos, valores opcionales vacíos y rechazo sin modificar configuración ni auditoría. |
| `HRL-77` | [HRL-120](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-120) | Subtask | `HRL-59` | Pendiente | Recibir por constructor los repositorios colaboradores que hoy se instancian en RepairService, EvidenceService y ReportService, conservando la transacción única y la composición. |
| `HRL-78` | [HRL-121](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-121) | Subtask | `HRL-59` | Pendiente | Probar las colaboraciones mediante contratos de repositorio sustituibles donde permitan simular fallos, sin convertir los comandos en acceso directo a SQL. El retiro de rutas anteriores se concentra en HRL-63. |
| `HRL-79` | [HRL-123](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-123) | Subtask | `HRL-60` | Pendiente | Cerrar la coexistencia de acceso a Tauri directo en composables con HRL-63 y mantener `services/api` como único adaptador IPC de aplicación. |
| `HRL-80` | [HRL-124](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-124) | Subtask | `HRL-60` | Pendiente | Trasladar las reglas financieras de exclusión, periodos y agregación de `utils/finances.ts` a servicio/modelo de dominio según arquitectura, dejando formato y fechas puras en utils. |
| `HRL-81` | [HRL-125](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-125) | Subtask | `HRL-60` | Pendiente | Verificar equivalencia de los filtros e indicadores, denegaciones en backend y descarte de respuestas de una sesión anterior tras completar la separación. |
| `HRL-82` | [HRL-127](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-127) | Subtask | `HRL-61` | Pendiente | Trasladar las pantallas completas restantes a views y actualizar la composición de WorkshopApp conservándolo como contenedor. |
| `HRL-83` | [HRL-128](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-128) | Subtask | `HRL-61` | Pendiente | Extraer estado y operaciones de cámara y editor de cláusulas; llevar al viewmodel la selección de días sugeridos que hoy calcula la plantilla de RepairWorkspace. |
| `HRL-84` | [HRL-129](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-129) | Subtask | `HRL-61` | Pendiente | Verificar guardas, eventos, foco y errores de las pantallas trasladadas sin cambiar permisos ni comportamiento del taller. |
| `HRL-85` | [HRL-131](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-131) | Subtask | `HRL-62` | Pendiente | Precisar el contrato de custodia/restauración conjunta de DB, assets y clave protegida, distinguiendo recuperación con el mismo usuario Windows de portabilidad. No elegir exportación o cambio de protección sin definición de HRL-70. |
| `HRL-86` | [HRL-132](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-132) | Subtask | `HRL-62` | Pendiente | Verificar reapertura DPAPI con la identidad real de destino y fallos por archivo ausente/corrupto o identidad no válida; comprobar que no aparece material de clave en SQLite ni auditoría. |
| `HRL-87` | [HRL-134](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-134) | Subtask | `HRL-63` | Pendiente | Archivar migraciones anteriores en legacy y retirar su registro del cargador anterior sin ejecutar ni migrar datos reales (5 puntos de los 13 del padre). |
| `HRL-88` | [HRL-135](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-135) | Subtask | `HRL-63` | Pendiente | Retirar comandos/módulos anteriores de lib.rs y el plugin ligado a esa ruta, conservando solo servicios normalizados y el mecanismo de seguridad vigente (5 puntos). |
| `HRL-89` | [HRL-136](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-136) | Subtask | `HRL-63` | Pendiente | Retirar composables/componentes Legacy y datos de demostración sin uso como fuente de verdad, tras comprobar referencias y conservar utilidades puras aún necesarias (3 puntos). |
| `HRL-90` | [HRL-137](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-137) | Subtask | `HRL-63` | Pendiente | Comprobar ausencia de invocaciones antiguas y regresión de permisos, recepción, reparación y dinero sobre la única ruta vigente (incluida en las entregas anteriores). |
| `HRL-91` | [HRL-139](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-139) | Subtask | `HRL-64` | Pendiente | Ejecutar y completar regresión frontend de sesión, borradores y reintentos; reparar en una tarea posterior la referencia `test:company` a `tests/company.test.mjs`, ausente del árbol revisado (5 puntos). |
| `HRL-92` | [HRL-140](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-140) | Subtask | `HRL-64` | Pendiente | Ejecutar y completar cobertura Rust de migraciones/FK, permisos, concurrencia, libros inmutables y rollback de auditoría, reportando resultados del árbol exacto (5 puntos). |
| `HRL-93` | [HRL-141](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-141) | Subtask | `HRL-64` | Pendiente | Validar en la aplicación Windows cámara concedida/denegada y liberación, impresora carta/ticket, geometría de ventana y reapertura DPAPI; registrar entorno y evidencia reproducible (3 puntos). |
| `HRL-94` | [HRL-144](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-144) | Subtask | `HRL-66` | Pendiente | Definir con producto el contrato de consulta del cliente y construir la vista del portal después de aprobarlo, usando solo datos públicos previstos (5 puntos). |
| `HRL-95` | [HRL-145](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-145) | Subtask | `HRL-66` | Pendiente | Implementar publicación real según contrato y autorización definidos, separada del guardado local y con trazabilidad de la operación efectiva (5 puntos). |
| `HRL-96` | [HRL-146](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-146) | Subtask | `HRL-66` | Pendiente | Verificar distinción entre guardado local/publicación, aislamiento de notas internas y manejo del fallo de conexión conforme al contrato que se apruebe (3 puntos). |
| `HRL-97` | [HRL-148](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-148) | Subtask | `HRL-67` | Pendiente | Inventariar una copia autorizada de la base origen y definir correspondencias a 3FN, conservando libros, identificadores de referencia y fechas desconocidas; no escribir migración de datos sin confirmar el alcance. |
| `HRL-98` | [HRL-149](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-149) | Subtask | `HRL-67` | Pendiente | Diseñar la importación transaccional al destino normalizado después de confirmar datos y reglas de conflictos, sin reutilizar la sesión ni el pool del importador anterior. |
| `HRL-99` | [HRL-150](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-150) | Subtask | `HRL-67` | Pendiente | Con fixtures autorizadas, comprobar conservación de originales, dinero y auditoría, rechazo de datos ambiguos y ausencia de duplicación al repetir la importación según el contrato aprobado. |
| `HRL-100` | [HRL-152](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-152) | Subtask | `HRL-68` | Pendiente | Diseñar selección y consulta de archivos adicionales después de definir formatos, tamaños y comportamiento de visualización; no reutilizar silenciosamente el límite fotográfico de 20 MiB. |
| `HRL-101` | [HRL-153](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-153) | Subtask | `HRL-68` | Pendiente | Extender contratos de evidencia y estados de reintento manteniendo identificadores, alcance y errores de sesión. |
| `HRL-102` | [HRL-154](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-154) | Subtask | `HRL-68` | Pendiente | Ampliar validación y almacenamiento controlado por tipo junto al repositorio normalizado, una vez aprobada la política de archivo. |
| `HRL-103` | [HRL-155](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-155) | Subtask | `HRL-68` | Pendiente | Cubrir tipos autorizados/no autorizados, tamaños definidos, rollback de metadatos/auditoría y conservación del original retirado. |
| `HRL-104` | [HRL-158](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-158) | Subtask | `HRL-70` | Pendiente | Definir e implementar, tras aprobar la estrategia de HRL-62, el procedimiento autorizado para recuperar claves junto con la instalación, distinguiéndolo del restablecimiento Ed25519 de contraseñas de acceso (5 puntos). |
| `HRL-105` | [HRL-159](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-159) | Subtask | `HRL-70` | Pendiente | Implementar captura y restauración consistentes de DB normalizada y assets, conservando referencias, libros y secuencias; el respaldo legacy existente no sustituye esta entrega (5 puntos). |
| `HRL-106` | [HRL-160](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-160) | Subtask | `HRL-70` | Pendiente | Probar restauración conjunta en origen/destino y rechazos por paquete incompleto, clave no recuperable o esquema incompatible, sin alterar el original hasta validar el resultado (3 puntos). |

## Backlog anterior sustituido

Las siguientes son claves Jira reales anteriores, no referencias documentales. Se conservaron descripción y etiquetas previas y se añadieron `backlog-reemplazado`, `reemplazado-2026-11`, el prefijo `[REEMPLAZADO]` y un vínculo con HRL-161. El estado Terminado significa cierre por sustitución de planificación.

| Jira anterior | Estado antes | Título anterior |
| --- | --- | --- |
| [HRL-8](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-8) | Terminado | [P01] Comprobar el entorno y el proyecto inicial |
| [HRL-9](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-9) | En curso | [P02] Cerrar las reglas mínimas del registro y la bitácora |
| [HRL-10](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-10) | Por hacer | [P03] Acordar la sincronización con el responsable web |
| [HRL-11](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-11) | En prueba | [S101] Crear el esquema local y las migraciones |
| [HRL-12](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-12) | Terminado | [S102] Configurar el primer administrador e iniciar sesión |
| [HRL-13](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-13) | En prueba | [S103] Implementar accesos completo, limitado y personalizado |
| [HRL-14](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-14) | En prueba | [S104] Registrar y recuperar una orden completa |
| [HRL-15](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-15) | Terminado | [S105] Registrar el primer evento de auditoría |
| [HRL-19](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-19) | En prueba | [S201] Buscar y modificar órdenes |
| [HRL-20](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-20) | En curso | [S202] Capturar y agregar fotografías de evidencia |
| [HRL-21](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-21) | En curso | [S203] Registrar avances técnicos y notas |
| [HRL-22](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-22) | En prueba | [S204] Registrar abonos y calcular el saldo |
| [HRL-23](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-23) | En prueba | [S205] Consultar la bitácora de cambios |
| [HRL-24](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-24) | En curso | [S206] Probar edición, evidencias, permisos y pagos |
| [HRL-26](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-26) | En curso | [S208] Revisar la operación local |
| [HRL-27](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-27) | En prueba | [S301] Mostrar cobros y pendientes en la bitácora |
| [HRL-28](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-28) | En prueba | [S302] Guardar la configuración esencial de la empresa |
| [HRL-29](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-29) | En curso | [S303] Generar notas de entrada y salida |
| [HRL-30](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-30) | En curso | [S304] Registrar la garantía básica por orden |
| [HRL-31](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-31) | Por hacer | [S305] Publicar cambios autorizados con una cola local |
| [HRL-32](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-32) | Por hacer | [S306] Probar el contrato de intercambio |
| [HRL-33](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-33) | Por hacer | [S307] Probar notas, garantías, totales y desconexión |
| [HRL-34](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-34) | Por hacer | [S308] Documentar la bitácora y la sincronización |
| [HRL-35](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-35) | Por hacer | [S309] Revisar el flujo previo a la entrega |
| [HRL-36](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-36) | Por hacer | [S401] Integrar la publicación con el servicio web real |
| [HRL-37](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-37) | Por hacer | [S402] Preparar instalación y recuperación local |
| [HRL-38](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-38) | Por hacer | [S403] Realizar pruebas con personal del taller |
| [HRL-39](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-39) | Por hacer | [S404] Corregir fallos que impidan el uso |
| [HRL-40](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-40) | Por hacer | [S405] Completar resultados y cierre de la memoria |
| [HRL-41](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-41) | Por hacer | [S406] Revisar la versión candidata |
| [HRL-42](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-42) | Por hacer | [E01] Revisar formato y referencias del reporte |
| [HRL-43](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-43) | Por hacer | [E02] Preparar el paquete para revisión |
| [HRL-44](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-44) | Por hacer | [E03] Entregar para revisión |
| [HRL-45](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-45) | Por hacer | [R01A] Aplicar observaciones recibidas |
| [HRL-46](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-46) | Por hacer | [A01] Capturar y reproducir video |
| [HRL-47](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-47) | Por hacer | [A02] Ampliar la galería de evidencias |
| [HRL-48](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-48) | En curso | [A03] Completar opciones avanzadas de Empresa |
| [HRL-49](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-49) | Por hacer | [A04] Consultar garantías de órdenes anteriores |
| [HRL-50](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-50) | Por hacer | [A05] Ampliar controles de publicación |
| [HRL-51](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-51) | En curso | [A06] Ampliar filtros y exportación de reportes |
| [HRL-52](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-52) | En curso | [A07] Permitir estados escritos por el usuario |
| [HRL-53](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-53) | En curso | [A08] Agregar logo desde el portapapeles |
| [HRL-54](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-54) | En prueba | Autorizar acceso temporal por ventana con auditoría de ambas identidades |

La búsqueda indexada tardó en reflejar el último reemplazo, HRL-54. Se verificó directamente: título reemplazado, etiqueta correcta, Terminado y sin sprint. No se duplicó la operación por ese retraso del índice.

Solo se añadieron o actualizaron los dos archivos de sincronización en `docs/backlog/`. El código, configuración, migraciones, pruebas y documentos fuente del backlog se conservaron.

