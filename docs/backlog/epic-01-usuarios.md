# HRL-1 [Epic] Módulo de Usuarios, registro e inicio de sesión

Estado: Hecho por inspección estática de sus historias. La ejecución y el recorrido nativo no están verificados en esta revisión.
Prioridad: Alta, porque autentica y autoriza los demás módulos.
Puntos: 42 (suma de historias; no se suman otra vez al sprint).
Sprint: 1 y 2, según historia.
Subtareas: agrupador; conserva HRL-2 a HRL-10 como hijas. No recibe subtareas técnicas adicionales.

Criterios de aceptación del epic (fuentes: `docs/accesos.md`, `docs/acceso-validacion.md`, `docs/acceso-temporal.md`):

- Dada una instalación nueva, cuando se configura el acceso, entonces administrador y clave maestra se crean atómicamente sin credenciales predeterminadas.
- Dada una sesión activa, cuando ejecuta una operación protegida, entonces Rust revalida sus facultades permanentes o temporales vigentes.
- Dado un intento de desactivar al último administrador o usar un préstamo fuera de alcance, cuando se procesa, entonces se deniega sin perder la administración ni elevar permisos.

Lectura del árbol de trabajo del 11 de noviembre de 2026, incluidos archivos modificados y nuevos todavía sin commit. «Hecho» acredita implementación localizada, no pruebas ejecutadas. Los puntos estiman alcance completo, no esfuerzo restante. Las historias hechas no reciben trabajo ficticio; la validación nativa transversal se concentra en HRL-64. Los epics se aceptan al cumplir los criterios de sus hijas y no se cuentan como historias independientes.

## HRL-2 [Historia] Configuración inicial: definir clave maestra y crear primer administrador (envío atómico)

Estado: Hecho (evidencia: `src-tauri/src/services/access.rs::AccessService::initialize`, `src-tauri/src/repositories/access.rs::AccessRepository::bootstrap`, `src/composables/useAccessForm.ts::submit`). El servicio confirma administrador, hash maestro y auditoría en una transacción; la sesión se publica después del commit.
Prioridad: Alta, porque habilita el acceso inicial y evita una instalación sin administrador.
Puntos: 5
Sprint: 1 (26-30 oct de 2026)
Subtareas: ninguna faltante detectada por lectura. Prueba existente, no ejecutada: `src-tauri/src/services/tests.rs::setup_and_login_audit_failures_do_not_leave_partial_state_or_sessions`.
Criterios de aceptación (fuentes: `docs/acceso-validacion.md`, `docs/accesos.md`):

- Dado un equipo sin configurar, cuando se envían claves válidas y confirmadas, entonces se crea un administrador activo completo y una clave maestra independiente protegidos con Argon2.
- Dado el primer uso, cuando falla el alta o la auditoría, entonces no queda configuración parcial ni sesión autenticada.
- Dado un acceso ya configurado, cuando se repite el alta inicial, entonces se rechaza sin sobrescribir cuentas ni claves.

## HRL-3 [Historia] Inicio de sesión con usuario y contraseña (Argon2, solo cuentas activas)

Estado: Hecho (evidencia: `src-tauri/src/services/access.rs::AccessService::login` y `verify`, `src-tauri/src/repositories/access.rs::AccessRepository::by_login`, `src/stores/useAuthStore.ts::loginUser`). La consulta excluye cuentas inactivas y el servicio verifica Argon2.
Prioridad: Alta, porque protege el acceso a datos y operaciones.
Puntos: 3
Sprint: 1 (26-30 oct de 2026)
Subtareas: ninguna faltante detectada. Prueba presente, no ejecutada: `src-tauri/src/services/tests.rs::initial_setup_and_login_return_only_public_identity_and_permanent_permissions`.
Criterios de aceptación (fuentes: `docs/accesos.md`, `docs/acceso-validacion.md`):

- Dada una cuenta activa y credenciales correctas, cuando inicia sesión, entonces recibe identidad pública y permisos permanentes sin hashes.
- Dada una contraseña incorrecta o cuenta inactiva, cuando intenta entrar, entonces se rechaza sin montar el taller autenticado.
- Dadas cuentas persistidas, cuando se reinicia la aplicación, entonces se exige un nuevo inicio de sesión.

## HRL-4 [Historia] Cierre de sesión e invalidación de datos de la sesión anterior

Estado: Hecho (evidencia: `src-tauri/src/services/access.rs::AccessService::logout`, `src/stores/useAuthStore.ts::clearSession`, `logoutUser` y `assertSessionVersion`, `src/App.vue` montaje condicional de `WorkshopApp`). El cierre retira sesión y préstamo en memoria; las respuestas tardías se invalidan por versión.
Prioridad: Alta, porque impide reutilizar datos o facultades de otro operador.
Puntos: 3
Sprint: 1 (26-30 oct de 2026)
Subtareas: ninguna faltante detectada; guardas específicas cubiertas por HRL-23. Pruebas de stores presentes en `tests/ipc-stores.test.ts`, sin ejecutar.
Criterios de aceptación (fuentes: `docs/acceso-validacion.md`, `docs/acceso-temporal.md`):

- Dada una sesión abierta sin borradores, cuando se cierra, entonces se desmonta el taller y se invalidan los permisos y datos de esa sesión.
- Dado un préstamo activo, cuando se cierra sesión, entonces el préstamo deja de autorizar operaciones posteriores.
- Dada una solicitud anterior todavía en curso, cuando llega tras cambiar de sesión, entonces no incorpora datos a la sesión nueva.

## HRL-5 [Historia] Recuperación de clave maestra con soporte (desafío de 15 min, firma Ed25519)

Estado: Hecho (evidencia: `src-tauri/src/services/access.rs::AccessService::generate_recovery_challenge` y `verify_recovery_signature`, `src-tauri/src/services/recovery.rs::RecoveryState::consume_at`, `src/composables/useAccessForm.ts::verifySignature`). Existe clave pública de soporte separada del vector de prueba; no se inspeccionó ni se necesita la clave privada.
Prioridad: Alta, porque restablece credenciales privilegiadas.
Puntos: 8
Sprint: 2 (2-6 nov de 2026)
Subtareas: ninguna faltante detectada en el código. Prueba presente, no ejecutada: `src-tauri/src/services/tests.rs::recovery_audit_failure_rolls_back_credentials_and_loan_closure_and_consumes_signature`.
Criterios de aceptación (fuentes: `docs/backlog-base.md` HRL-5, `docs/contexto-completo-proyecto.md`, `docs/acceso-validacion.md` para la prohibición de puertas predeterminadas):

- Dada una instalación configurada, cuando soporte firma el desafío vigente con Ed25519, entonces se pueden restablecer la clave maestra y la del administrador sin sesión previa.
- Dado un desafío vencido a los 15 minutos, alterado o consumido, cuando se presenta su firma, entonces se rechaza el restablecimiento.
- Dado un restablecimiento aceptado, cuando termina, entonces se revocan accesos temporales, se exige iniciar sesión y se audita sin guardar claves.

Discrepancia D-01: `docs/acceso-validacion.md` y `docs/accesos.md` dicen que el procedimiento sigue por definir o no existe recuperación automática; el backlog vigente solicita recuperación con soporte y el código ya la implementa. No se modifica ni se resuelve aquí esa documentación histórica.

## HRL-6 [Historia] Gestión de usuarios: crear, editar, activar/desactivar y cambiar contraseña

Estado: Hecho (evidencia: `src-tauri/src/services/users.rs::UserService::save` y `list`, `src-tauri/src/repositories/users.rs::UserRepository::save`, `src/composables/useUsersWorkspace.ts::save`). Revalida `users.manage` y clave maestra, conserva clave si ambos campos quedan vacíos y audita cambios.
Prioridad: Alta, porque controla identidad, privilegios y continuidad del acceso.
Puntos: 5
Sprint: 2 (2-6 nov de 2026)
Subtareas: ninguna faltante detectada. Prueba presente, no ejecutada: `src-tauri/src/services/tests.rs::restored_company_and_users_persist_with_permissions_and_revisions`.
Criterios de aceptación (fuentes: `docs/accesos.md`, `docs/acceso-temporal.md`):

- Dado permiso de administrar usuarios y clave maestra correcta, cuando se guarda una cuenta válida, entonces persisten su estado y permisos con auditoría.
- Dada una cuenta existente, cuando se dejan contraseña y confirmación vacías, entonces conserva su contraseña actual.
- Dado un operador sin permiso o con clave maestra incorrecta, cuando intenta administrar cuentas, entonces se rechaza sin modificar usuarios.

## HRL-7 [Historia] Niveles de acceso y permisos por apartado y acción (completo, limitado, personalizado)

Estado: Hecho (evidencia: `src-tauri/src/services/access.rs::AccessService::permanent` y `require`, `src-tauri/src/permissions.rs::limited_allows`, `src/stores/useAuthStore.ts::can`). Rust es la autoridad, independientemente de controles visibles.
Prioridad: Alta, porque bloquea operaciones no autorizadas en todas las capas funcionales.
Puntos: 5
Sprint: 1 (26-30 oct de 2026)
Subtareas: ninguna faltante detectada por lectura. Prueba presente, no ejecutada: `src-tauri/src/services/tests.rs::limited_accounts_require_independent_payment_permission_and_temporary_use_is_audited`.
Criterios de aceptación (fuentes: `docs/accesos.md`, `docs/reparaciones-permisos.md`, `docs/reglas-pagos.md`):

- Dado acceso limitado, cuando registra notas, avances, garantías, refacciones o una recepción, entonces puede agregar sin obtener permiso genérico de editar o borrar.
- Dado acceso personalizado, cuando ejecuta una acción, entonces Rust exige el permiso de ese apartado y acción.
- Dado acceso limitado sin permiso de cobro, cuando registra dinero, entonces se rechaza aunque pueda agregar trabajo técnico.

## HRL-8 [Historia] Protección del último administrador activo (servicio + triggers)

Estado: Hecho (evidencia: `src-tauri/src/services/users.rs::UserService::save`; `src-tauri/migrations/001_esquema_objetivo.sql::usuarios_primer_administrador`, `usuarios_conservar_admin_al_borrar`, `usuarios_conservar_admin_al_editar`). Se exige rol administrador, acceso completo y estado activo.
Prioridad: Alta, porque evita bloquear la instalación por pérdida del último administrador.
Puntos: 3
Sprint: 2 (2-6 nov de 2026)
Subtareas: ninguna faltante detectada en servicio y SQL. No se ejecutaron sus pruebas.
Criterios de aceptación (fuentes: `docs/accesos.md`, `docs/arquitectura-objetivo.md` sección 7):

- Dado un único administrador activo completo, cuando se intenta desactivarlo o quitarle ese nivel o rol, entonces el servicio rechaza la operación.
- Dado el mismo caso, cuando una escritura directa intenta eliminarlo o degradarlo, entonces el trigger aborta.
- Dado otro administrador activo completo, cuando se modifica el primero con autorización, entonces permanece al menos uno válido.

## HRL-9 [Historia] Acceso temporal: autorización de otra persona con motivo, limitada a apartado u orden

Estado: Hecho (evidencia: `src-tauri/src/services/access.rs::AccessService::temporary`, `require`, `finish_temporary` y `recover`; `src/composables/useWorkshop.ts::navigate` y `finishTemporary`; `src/stores/useAuthStore.ts::requestTemporaryAccess`). La concesión conserva al operador y revalida al autorizador en cada uso.
Prioridad: Alta, porque presta facultades para acciones protegidas y requiere trazabilidad.
Puntos: 8
Sprint: 2 (2-6 nov de 2026)
Subtareas: ninguna faltante detectada en el flujo leído. Pruebas presentes, no ejecutadas: `src-tauri/src/services/tests.rs::temporary_grants_revalidate_approver_and_cannot_escape_their_order_scope` y `failed_end_audit_removes_memory_access_and_restart_never_restores_a_loan`.
Criterios de aceptación (fuente: `docs/acceso-temporal.md`):

- Dada otra persona con permiso permanente y motivo, cuando autoriza, entonces el operador conserva su identidad y recibe solo facultades del apartado u orden autorizados.
- Dado un préstamo para reparación A, cuando se intenta usarlo en B o se revoca al autorizador, entonces se deniega la operación.
- Dado un préstamo activo, cuando se sale del apartado, cambia de orden o cierra sesión, entonces se retira después de resolver el borrador.
- Dado un uso de facultad prestada para modificar datos, cuando confirma, entonces se registra `access.temporary.use` en la misma transacción; si falla la auditoría se revierte el cambio.

Discrepancia D-02: `docs/acceso-temporal.md` atribuye inicio y fin a triggers de la migración 011 anterior; la ruta normalizada los registra mediante `AccessService` y `AuditService` en `accesos_temporales`.

## HRL-10 [Historia] Perfil de ventana según estado (acceso, configuración inicial, taller)

Estado: Hecho (evidencia: `src-tauri/src/windowing.rs::Profile::for_session` y `WindowProfileState::apply_to`, `src-tauri/src/services/access.rs::AccessService::synchronize_window`, `src/composables/useWindowProfile.ts::synchronize`). La geometría nativa real queda No verificada y se incluye en HRL-64.
Prioridad: Media, porque permite trabajar y volver al acceso con geometría consistente.
Puntos: 2
Sprint: 1 (26-30 oct de 2026)
Subtareas: ninguna faltante de implementación detectada; recorrido nativo pendiente en HRL-64. Prueba presente, no ejecutada: `src-tauri/src/windowing.rs::tests::login_size_returns_after_logout_even_from_a_maximized_workshop`.
Criterios de aceptación (fuente: `docs/ventana-por-sesion.md`):

- Dado acceso o configuración inicial, cuando se sincroniza la ventana, entonces usa respectivamente 1000 × 680 o 1000 × 760, fija y sin maximización.
- Dado taller autenticado, cuando se abre, entonces inicia en 1280 × 760 con mínimo 1120 × 680 y admite redimensionar.
- Dado taller maximizado, cuando cierra sesión, entonces vuelve centrado y desmaximizado al tamaño de acceso.
- Dado un fallo nativo al ajustar tamaño, cuando se recibe el error, entonces ofrece reintento sin modificar la sesión.
