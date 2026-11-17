# Estado del backlog HRL Proyecto

Corte: 17 de noviembre de 2026. Revisión estática del árbol de trabajo; no se ejecutaron la aplicación, builds ni pruebas. Hecho significa respaldo en código de los criterios y reglas revisados, no certificación nativa. La existencia de una prueba no acredita que haya pasado. En progreso también incluye criterios respaldados con reglas obligatorias incompletas. Verificado en el CSV significa dictamen obtenido por lectura; No verificado exige ejecución o validación externa.

Fuentes originales leídas: docs/hrl-proyecto-backlog.md y docs/hrl-proyecto-jira.csv. Las rutas homónimas pedidas bajo docs/backlog/ no existen. docs/contexto-vistas-modulos.md tampoco existe: se leyó docs/contexto-funcional-vistas-y-estructura.md. Se leyeron AGENTS.md, arquitectura-objetivo.md y las seis especificaciones de negocio pedidas. La documentación se usa para contrastar; los dictámenes se basan en el código actual. Los cambios locales previos en documentación se consideran parte del árbol leído y no se modifican. Inventario de existencias excluido.

C1, C2, C3 siguen el orden de criterios del original. «Sin respaldo» diferencia ausencia o discrepancia de lo que requiere una comprobación externa. Las referencias sin función corresponden a plantillas, DDL o módulos declarativos.

## HRL-1 · Base técnica y arquitectura

### HRL-10 Esquema SQLite normalizado y migraciones

Estado: **En progreso**. Sprint 1 · 8 puntos.

Evidencia: `src-tauri/src/db/migrations.rs::migrator/apply`; `src-tauri/src/db/connection.rs::Database::open_file/connect`; `src-tauri/src/commands/mod.rs::BackendState::services`; `src-tauri/migrations/001_esquema_objetivo.sql::abonos_no_update/abonos_no_delete/devoluciones_no_update/auditoria_no_delete`.

Respaldo de criterios: C2: detecta Ordenes con datos_json y devuelve LegacyDatabase. C3: triggers BEFORE UPDATE/DELETE abortan mutaciones. C1: crea el archivo normalizado, pero aplica tres migraciones.

Criterios no cumplidos o sin verificar: C1: no aplica «solo 001 y 002»; incluye 003_antecedentes_equipo.sql.

Discrepancias: El backlog y la sección de cierre del cargador en arquitectura-objetivo.md describen 001–002; el cargador activo incluye 003.

### HRL-11 Arquitectura backend en capas

Estado: **En progreso**. Sprint 1 · 8 puntos.

Evidencia: `src-tauri/src/commands/mod.rs::create_order`; `src-tauri/src/services/orders.rs::OrderService::new/create`; `src-tauri/src/domain/money.rs::Money`; `src-tauri/src/services/access.rs::AccessService::require`; `src-tauri/src/db/migrations.rs::apply`.

Respaldo de criterios: C1: los comandos delegan y no contienen SQL. C2: Money encapsula i64 y los DTO usan price_minor. C3: los servicios llaman a require dentro de transacciones.

Criterios no cumplidos o sin verificar: Ningún criterio explícito sin respaldo; la regla «repositorios contienen el SQL» no se cumple de forma exclusiva: db/migrations.rs::apply ejecuta SELECT de reconocimiento del esquema.

Discrepancias: AGENTS.md exige SQL exclusivamente en repositories; la detección del esquema anterior está en db/migrations.rs. Se conserva el estado En progreso por esta regla arquitectónica.

### HRL-12 Arquitectura frontend MVVM con Pinia

Estado: **Hecho**. Sprint 1 · 5 puntos.

Evidencia: `src/services/api/transport.ts::request`; `src/stores/useAuthStore.ts::clearSession/assertSessionVersion`; `src/stores/useOrderStore.ts::listOrders/acceptOrder`; `src/stores/useCompanyStore.ts::load`; `src/stores/useUsersStore.ts::load`; `src/main.ts`.

Respaldo de criterios: C1: la búsqueda de invoke en src solo encuentra transport.ts. C2: watches de sessionVersion limpian datos. C3: versiones de sesión impiden aceptar respuestas antiguas.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: El contexto funcional antiguo describe llamadas IPC dispersas y ausencia de Pinia; src/services/api y src/stores ya implementan la separación.

### HRL-13 Contrato IPC: errores tipificados y control de revisión

Estado: **Hecho**. Sprint 1 · 5 puntos.

Evidencia: `src-tauri/src/error.rs::AppError::code/serialize`; `src/services/api/errors.ts::normalizeIpcError`; `src-tauri/src/services/orders.rs::OrderService::edit`; `src/composables/usePaymentsPanel.ts::submit/retryPending`; `src/stores/useOrderStore.ts::acceptOrder`.

Respaldo de criterios: C1: códigos VALIDATION_ERROR, FORBIDDEN, SESSION_CHANGED, CONFLICT y RESULT_UNCERTAIN reconocidos por la UI. C2: expected_revision se compara antes del cambio. C3: se conserva el sobre financiero exacto en localStorage y se reutiliza.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: El contrato contiene más códigos que las cinco categorías resumidas en el backlog; no implica pérdida de esas categorías.

### HRL-14 Custodia de claves con Windows DPAPI

Estado: **No verificado**. Sprint 1 · 5 puntos.

Evidencia: `src-tauri/src/services/key_store.rs::OsKeyStore::open_key/protect`; `src-tauri/src/services/crypto.rs::CryptoService`; `src-tauri/src/services/mod.rs::BackendServices::with_os_key_store`.

Respaldo de criterios: C1: clave AES en archivo DPAPI externo; SQLite conserva el sobre. C2: ausencia/corrupción de custodia devuelve Crypto y no habilita texto claro. C3: CryptProtectData/CryptUnprotectData usa contexto de usuario.

Criterios no cumplidos o sin verificar: C3 no verificado: rechazo del descifrado desde otro perfil Windows.

Discrepancias: El respaldo portable ahora tiene código propio; copiar solamente SQLite sigue sin incluir archivos y custodia.

Para verificar: Ejecutar instalación Windows con dos perfiles; guardar una credencial y comprobar recuperación en el original y rechazo en el segundo, incluida ausencia/corrupción del archivo DPAPI.

### HRL-15 Retirar código anterior y cerrar la migración

Estado: **Hecho**. Sprint 4 · 5 puntos.

Evidencia: `src-tauri/src/lib.rs::run`; `src-tauri/src/commands/mod.rs::BackendState::services`; `src/WorkshopApp.vue::<script setup>/<template>`; `src/views/`; `legacy/frontend/`; `src-tauri/legacy/`.

Respaldo de criterios: C1: el registro productivo solo incluye commands normalizados; módulos antiguos bajo cfg(test). C2: la ruta activa abre hrl_core_3fn.db, sin inspección de hrl_datos.db. C3: WorkshopApp compone vistas y delega a useWorkshop.

Criterios no cumplidos o sin verificar: Ninguno en la lectura estática.

Discrepancias: El backlog presenta el retiro como pendiente; el árbol conserva el código histórico archivado, fuera del flujo productivo. El adaptador utils/legacyOrder.ts transforma DTO, no usa la base anterior.


