# Resumen de revisión del backlog

Corte: 17 de noviembre de 2026. Solo lectura de código; sin ejecución de pruebas, builds ni validación nativa. Revisión de 13/59 historias y tareas; los ítems aún sin revisar en commits intermedios no suman como Hecho. Epics excluidas de puntos y porcentajes; sus columnas nuevas quedan vacías para no atribuirles un estado no solicitado. Los cuatro ítems sin sprint forman Backlog.

| Sprint | Fechas 2026 | Ítems | Puntos totales | Puntos Hecho | Porcentaje |
| --- | --- | ---: | ---: | ---: | ---: |
| Sprint 1 | 26–30 oct | 13 | 70 | 28 | 40.0% |
| Sprint 2 | 2–6 nov | 12 | 70 | 21 | 30.0% |
| Sprint 3 | 9–13 nov | 17 | 76 | 0 | 0.0% |
| Sprint 4 | 16–20 nov | 13 | 69 | 5 | 7.2% |
| Backlog | sin fecha | 4 | 29 | 0 | 0.0% |

## Discrepancias entre documentación y código

- HRL-10: El backlog y la sección de cierre del cargador en arquitectura-objetivo.md describen 001–002; el cargador activo incluye 003. Evidencia: src-tauri/src/db/migrations.rs::migrator/apply.
- HRL-11: AGENTS.md exige SQL exclusivamente en repositories; la detección del esquema anterior está en db/migrations.rs. Se conserva el estado En progreso por esta regla arquitectónica. Evidencia: src-tauri/src/commands/mod.rs::create_order.
- HRL-12: El contexto funcional antiguo describe llamadas IPC dispersas y ausencia de Pinia; src/services/api y src/stores ya implementan la separación. Evidencia: src/services/api/transport.ts::request.
- HRL-13: El contrato contiene más códigos que las cinco categorías resumidas en el backlog; no implica pérdida de esas categorías. Evidencia: src-tauri/src/error.rs::AppError::code/serialize.
- HRL-14: El respaldo portable ahora tiene código propio; copiar solamente SQLite sigue sin incluir archivos y custodia. Evidencia: src-tauri/src/services/key_store.rs::OsKeyStore::open_key/protect.
- HRL-15: El backlog presenta el retiro como pendiente; el árbol conserva el código histórico archivado, fuera del flujo productivo. El adaptador utils/legacyOrder.ts transforma DTO, no usa la base anterior. Evidencia: src-tauri/src/lib.rs::run.
- HRL-20: La regla transversal del backlog exige revisión en toda edición; SaveUserPayload y UserService::save no incluyen expected_revision. Los criterios específicos de HRL-20 sí tienen respaldo; queda una brecha transversal que debe planificarse. Evidencia: src-tauri/src/services/users.rs::UserService::save.
- HRL-21: El documento acceso-temporal.md describe tablas históricas y triggers 011; el flujo vigente usa accesos_temporales/acceso_temporal_permiso y AuditService. Evidencia: src-tauri/src/services/access.rs::AccessService::temporary/require/finish_temporary/logout.

Las rutas originales reales están en docs/, no en docs/backlog/. El contexto solicitado se sustituyó por el documento funcional existente. Los totales declarados por el original se contrastan con las filas del CSV, sin corregir los originales.

## No verificado

- HRL-14: Ejecutar instalación Windows con dos perfiles; guardar una credencial y comprobar recuperación en el original y rechazo en el segundo, incluida ausencia/corrupción del archivo DPAPI.
- HRL-18: Abrir configuración, acceso y taller en Windows; maximizar taller, cerrar sesión y medir dimensiones, límites y capacidad de redimensionar.

## Sprint 4 (16–20 noviembre): trabajo que no está Hecho

- HRL-44 Bitácora global e historial de una orden (8 pts): Aún sin revisar en este commit intermedio.
- HRL-45 Finanzas: filtros y clasificación de órdenes (8 pts): Aún sin revisar en este commit intermedio.
- HRL-46 Indicadores financieros por moneda (5 pts): Aún sin revisar en este commit intermedio.
- HRL-47 Gastos directos (3 pts): Aún sin revisar en este commit intermedio.
- HRL-52 Cláusulas y textos de las notas (5 pts): Aún sin revisar en este commit intermedio.
- HRL-53 Hoja de impresión (3 pts): Aún sin revisar en este commit intermedio.
- HRL-54 Notas de entrada y de salida (8 pts): Aún sin revisar en este commit intermedio.
- HRL-58 Integrar la publicación con el servicio web (8 pts): Aún sin revisar en este commit intermedio.
- HRL-62 Pruebas de bitácora, finanzas y empresa (5 pts): Aún sin revisar en este commit intermedio.
- HRL-63 Validación nativa: cámara, impresora y DPAPI (3 pts): Aún sin revisar en este commit intermedio.
- HRL-64 Instalación y recuperación local (5 pts): Aún sin revisar en este commit intermedio.
- HRL-65 Pruebas piloto con personal del taller (3 pts): Aún sin revisar en este commit intermedio.

## Alcance y trazabilidad

Solo se generan estos tres archivos dentro de docs/backlog/. Se conservan las 68 filas originales, las claves de referencia HRL, los títulos, los padres, sprints, prioridades, puntos y todas las columnas originales. La verificación del CSV compara valores de celdas; las comillas y los saltos físicos pueden cambiar. No se elimina ni ejecuta código histórico. No se incluyen existencias de refacciones.

El usuario autorizó un proyecto nuevo HRLP, porque HRL ya pertenece a HRL CORE; las referencias HRL se conservarán en las descripciones de Jira. Estado de creación/sincronización: pendiente de completar mediante las herramientas disponibles.

Control de entrega documental: PASS, los dictámenes incluyen referencias a código y distinguen comprobaciones externas. PASS, los puntos se calculan desde el CSV original. PASS, no se atribuyen resultados a builds o pruebas no ejecutados. Las reglas visuales de antislop no aplican a esta auditoría sin cambios de UI.

