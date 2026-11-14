# Revisión de pendientes de Jira — 13 de noviembre de 2026

Consulta directa a Jira: proyecto HRL, incidencias con `statusCategory != Done`; paginación completa. Se encontraron 20 historias/tareas, 53 subtareas y 6 epics abiertos. Las claves de esta revisión son las claves reales de Jira; no se deben confundir con las referencias del backlog base.

## Entrega HRL-134 — archivo de migraciones antiguas

Se trasladaron con `git mv` las 13 migraciones anteriores a `src-tauri/migrations/legacy/`, sin alterar su contenido. Se eliminó su registro del plugin SQL en `src-tauri/src/lib.rs`. El cargador normalizado conserva las migraciones 001 y 002. `test_support`, compilado solo para pruebas, conserva la ejecución explícita de las migraciones antiguas sobre bases de prueba.

El plugin SQL y los comandos anteriores siguen presentes: su retiro pertenece a HRL-135. Esta entrega no importa, recrea ni modifica bases de usuario. No se modificó la UI. La decisión transitoria documentada en arquitectura §7 se actualizó para reflejar este archivo.

HRL-134 se pasó a Terminado en Jira tras completar las comprobaciones. Su padre HRL-133 permanece En curso. Quedan 52 subtareas abiertas de las 53 consultadas; esta entrega no cierra las 20 historias/tareas principales.

## Pendientes principales consultados

| Jira | Trabajo | Estado al consultar |
| --- | --- | --- |
| [HRL-10](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-10) | [063] [P03] Acordar la sincronización con el responsable del portal | Por hacer |
| [HRL-24](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-24) | [068] [S206] Probar edición, evidencias y permisos | En curso |
| [HRL-31](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-31) | [064] [S305] Publicar cambios autorizados | Por hacer |
| [HRL-32](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-32) | [065] [S306] Probar el contrato de intercambio | Por hacer |
| [HRL-33](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-33) | [069] [S307] Probar notas, garantías y totales | En curso |
| [HRL-36](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-36) | [066] [S401] Integrar la publicación con el servicio web | Por hacer |
| [HRL-37](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-37) | [070] [S402] Preparar instalación y recuperación local | En curso |
| [HRL-38](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-38) | [071] [S403] Realizar pruebas con personal del taller | En prueba |
| [HRL-50](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-50) | [067] [A05] Ampliar controles de publicación | Por hacer |
| [HRL-96](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-96) | [038] Detalle de cambios (valores anterior y nuevo, sin contraseñas) | En curso |
| [HRL-107](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-107) | [044] Datos generales: nombre, identificación fiscal, contacto, dirección y horario | En curso |
| [HRL-119](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-119) | [053] Arquitectura backend en capas: commands, services, repositories, db | En curso |
| [HRL-122](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-122) | [054] Frontend MVVM con stores Pinia y capa services/api | En curso |
| [HRL-126](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-126) | [055] Mover vistas restantes de components/ a views/ y adelgazar WorkshopApp | En curso |
| [HRL-130](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-130) | [056] Custodia de claves con DPAPI y estrategia de respaldo (base + assets + claves) | En curso |
| [HRL-133](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-133) | [057] Retirar código legacy (comandos anteriores, plugin SQL 001-013, archivos Legacy) | En curso |
| [HRL-138](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-138) | [058] Pruebas frontend y Rust, y validación nativa (cámara, impresora, DPAPI) | En prueba |
| [HRL-147](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-147) | [059] Importación de datos desde la base legacy | En prueba |
| [HRL-151](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-151) | [060] Carga de video y documentos como evidencia | Por hacer |
| [HRL-157](https://utrivieramaya-team-z3bsbh6g.atlassian.net/browse/HRL-157) | [062] Respaldo y migración automática entre instalaciones | Por hacer |

## Dependencias y orden de continuación

- Fase 1: HRL-134 resuelve el archivo y retiro del registro de las migraciones antiguas. No se adelanta la limpieza completa del frontend.
- Backend: siguen pendientes la inyección de colaboradores (HRL-120/121), los detalles de auditoría (HRL-97/98/100) y la validación de datos de empresa (HRL-108/109), conforme al orden de fases del repositorio.
- MVVM y vistas: HRL-122, HRL-126 y la limpieza HRL-133 requieren las fases posteriores. HRL-133 conserva pendientes HRL-135/136/137.
- Portal: HRL-10 requiere acordar contrato, autorización, entorno y ejemplos antes de implementar HRL-31/32/36/50. No se inventaron endpoints ni acuerdos externos.
- Importación: HRL-147 requiere confirmar datos origen y política de conflictos antes de escribir una migración.
- Evidencias adicionales: HRL-151 requiere definir formatos, tamaños y visualización.
- Respaldo: HRL-130 y HRL-157 requieren distinguir custodia recuperable con la misma identidad Windows de portabilidad entre instalaciones.
- Validación nativa y aceptación: HRL-141 y HRL-38 requieren cámara/impresora y participantes reales; las suites automatizadas no acreditan esos recorridos.
- El backlog sigue sin identificar cinco ítems de la epic de entrega; no se crearon tareas ficticias.

## Validación de esta entrega

- `cargo check --offline`: aprobado.
- `npm.cmd run build`: aprobado (comprobación TypeScript y compilación Vite).
- `npm.cmd run test:ipc`: 13/13 aprobadas.
- `npm.cmd run test:ui`: 9/9 aprobadas; utilizan IPC/renderizado de prueba, no hardware real.
- `cargo test --offline` dentro del aislamiento: 86/87 aprobadas; falla `windows_key_custody_reopens_dpapi_key_without_storing_it_in_sqlite` con `AppError::Crypto` al abrir la custodia.
- Repetición exacta del caso DPAPI fuera del aislamiento: 1/1 aprobada. No se modificó la protección ni la prueba. Esto acredita creación/cifrado/reapertura en un directorio temporal con la identidad Windows de ejecución; no acredita traslado entre usuarios/equipos ni una recuperación instalada completa.
- Suite Rust completa fuera del aislamiento: 87/87 aprobadas, sin pruebas omitidas. Incluye la custodia DPAPI y las pruebas antiguas con las rutas archivadas.
- Las 13 migraciones archivadas conservan el mismo hash Git que sus originales en HEAD. Solo 001 y 002 normalizadas permanecen en la raíz de migraciones; no queda `add_migrations` en `lib.rs`.
- `git diff --check`: aprobado. No se ejecutó instalación, cámara, impresión física ni aceptación con personal.

## Estado del árbol

El repositorio ya tenía numerosos cambios sin confirmar, archivos nuevos y pruebas eliminadas al iniciar esta revisión. Se conservaron. Los movimientos quedaron preparados en el índice por `git mv`; no se creó un commit que incluyera de forma accidental el trabajo previo. La entrega debe incorporarse junto con la base de refactorización pendiente antes de continuar a otra fase, como exige `docs/prompts-fases.md`.

## Continuación autorizada del backend

La continuación solicitada por el usuario completa las diferencias de correo y entrega estimada en la auditoría de órdenes (HRL-97), su identidad en el DTO de Bitácora (HRL-98), las pruebas de cambios opcionales y exclusión de secretos (HRL-100), la validación Rust del correo/web de Empresa (HRL-108/109) y la inyección de repositorios colaboradores con fallos sustituibles (HRL-120/121).

HRL-135 retira del ejecutable el plugin SQL y todos los comandos anteriores. Sus módulos y pruebas se trasladan a `src-tauri/legacy/`, se compilan solo para pruebas y conservan los 92 casos de la suite ampliada. `cargo check --offline` pasa con el arranque normalizado. La compilación de pruebas históricas informa 43 advertencias de funciones archivadas sin llamadas.

La custodia exige ahora el archivo DPAPI existente cuando SQLite contiene credenciales cifradas. Se añade una prueba de arranque ante archivo ausente/corrupto y recuperación al restituir el original. El procedimiento conjunto de base, recursos y clave se registra en `docs/instalacion-recuperacion-local.md` para HRL-131 y HRL-180.

La validación tras el retiro del backend anterior aprobó 92/92 pruebas Rust. La suite final aprobó 93/93, incluyendo el arranque ante custodia ausente/corrupta. El build y las 22 pruebas frontend aprobaron. El cambio de DTO conserva el resto de los campos del contrato. La presentación de identidad y etiquetas nuevas de Bitácora corresponde a HRL-99.

El commit del backend normalizado integra la refactorización de backend previamente pendiente con las correcciones de esta continuación. Los cambios de frontend previos siguen en el árbol para su fase correspondiente.
