use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::time::{Duration, Instant};
use tauri::State;
use tauri_plugin_sql::{DbInstances, DbPool};
use tokio::sync::Mutex;

const DATABASE_ERROR: &str =
    "No se pudo consultar la base local. Conserva los datos y vuelve a intentar.";
const INVALID_LOGIN: &str = "Usuario o clave incorrectos.";

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub name: String,
    pub access_level: String,
}

#[derive(Default)]
struct Session {
    user: Option<User>,
    failures: u32,
    retry_at: Option<Instant>,
}

#[derive(Default)]
pub struct AccessState(Mutex<Session>, Mutex<()>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessStatus {
    setup_complete: bool,
    user: Option<User>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstSetup {
    name: String,
    password: String,
    confirmation: String,
    master_password: String,
    master_confirmation: String,
}

pub(crate) async fn pool(instances: &DbInstances) -> Result<SqlitePool, String> {
    let databases = instances.0.read().await;
    let Some(DbPool::Sqlite(pool)) = databases.get("sqlite:hrl_datos.db") else {
        return Err("La base local no está disponible. Vuelve a abrir la aplicación.".into());
    };
    Ok(pool.clone())
}

pub(crate) fn validate_password(password: &str, confirmation: &str) -> Result<(), String> {
    if password.chars().count() < 8 || password.len() > 1024 {
        return Err("La clave debe tener al menos 8 caracteres y no superar 1024 bytes.".into());
    }
    if password != confirmation {
        return Err("Las dos claves deben coincidir.".into());
    }
    Ok(())
}

pub(crate) fn login_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control) {
        return Err("Escribe un usuario de 1 a 100 caracteres, sin saltos de línea.".into());
    }
    Ok(name.to_lowercase())
}

pub(crate) fn random_id() -> String {
    let mut bytes = [0_u8; 16];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) async fn hash_password(password: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| "No se pudo proteger la clave.".to_string())
    })
    .await
    .map_err(|_| "No se pudo proteger la clave.".to_string())?
}

pub(crate) async fn verify_password(password: String, hash: String) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        PasswordHash::new(&hash)
            .map(|parsed| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &parsed)
                    .is_ok()
            })
            .map_err(|_| {
                "El acceso guardado necesita revisión manual. No se han modificado los datos."
                    .to_string()
            })
    })
    .await
    .map_err(|_| "No se pudo comprobar la clave.".to_string())?
}

async fn setup_complete(pool: &SqlitePool) -> Result<bool, String> {
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(pool)
        .await
        .map_err(|_| DATABASE_ERROR)?;
    let settings: i64 = sqlx::query_scalar("SELECT count(*) FROM access_settings")
        .fetch_one(pool)
        .await
        .map_err(|_| DATABASE_ERROR)?;
    if (users == 0) != (settings == 0) {
        return Err("La configuración de acceso está incompleta y necesita revisión manual. No se han modificado los datos.".into());
    }
    Ok(users > 0)
}

impl AccessState {
    #[cfg(test)]
    pub(crate) async fn test_admin(pool: &SqlitePool) -> Self {
        let state = Self::default();
        state
            .first_setup(
                pool,
                FirstSetup {
                    name: "Test admin".into(),
                    password: "test-user-123".into(),
                    confirmation: "test-user-123".into(),
                    master_password: "test-master-123".into(),
                    master_confirmation: "test-master-123".into(),
                },
            )
            .await
            .unwrap();
        state
    }

    #[cfg(test)]
    pub(crate) async fn test_login(&self, pool: &SqlitePool, name: &str) {
        self.login(pool, name.into(), "test-user-123".into())
            .await
            .unwrap();
    }

    pub(crate) async fn authorize(
        &self,
        db: &mut sqlx::SqliteConnection,
        operation: &str,
    ) -> Result<User, String> {
        let id = self
            .0
            .lock()
            .await
            .user
            .as_ref()
            .ok_or("Inicia sesión para continuar.")?
            .id
            .clone();
        let row = sqlx::query("SELECT id,name,access_level FROM users WHERE id=? AND active=1")
            .bind(&id)
            .fetch_optional(&mut *db)
            .await
            .map_err(|_| DATABASE_ERROR)?
            .ok_or("La sesión ya no está activa. Vuelve a iniciar sesión.")?;
        let user = User {
            id: row.get("id"),
            name: row.get("name"),
            access_level: row.get("access_level"),
        };
        let (_, section, action) = crate::permissions::OPERATIONS
            .iter()
            .find(|(id, _, _)| *id == operation)
            .ok_or("Operación desconocida.")?;
        let allowed = match user.access_level.as_str() {
            "complete" => true,
            "limited" => crate::permissions::limited_allows(operation),
            "custom" if operation != "users.manage" => sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM user_permissions WHERE user_id=? AND section=? AND action=?",
            )
            .bind(&id)
            .bind(section)
            .bind(action)
            .fetch_one(&mut *db)
            .await
            .map_err(|_| DATABASE_ERROR)?
                > 0,
            _ => false,
        };
        if !allowed {
            return Err("Tu acceso no permite realizar esta acción.".into());
        }
        Ok(user)
    }

    async fn status(&self, pool: &SqlitePool) -> Result<AccessStatus, String> {
        let setup_complete = setup_complete(pool).await?;
        let previous = self.0.lock().await.user.clone();
        let refreshed = if let Some(user) = &previous {
            sqlx::query("SELECT id,name,access_level FROM users WHERE id=? AND active=1")
                .bind(&user.id)
                .fetch_optional(pool)
                .await
                .map_err(|_| DATABASE_ERROR)?
                .map(|row| User {
                    id: row.get("id"),
                    name: row.get("name"),
                    access_level: row.get("access_level"),
                })
        } else {
            None
        };
        let mut session = self.0.lock().await;
        if session.user == previous {
            session.user = refreshed;
        }
        Ok(AccessStatus {
            setup_complete,
            user: session.user.clone(),
        })
    }

    async fn first_setup(&self, pool: &SqlitePool, input: FirstSetup) -> Result<User, String> {
        let key = login_name(&input.name)?;
        validate_password(&input.password, &input.confirmation)?;
        validate_password(&input.master_password, &input.master_confirmation)?;
        let _gate = self.1.lock().await;
        if setup_complete(pool).await? {
            return Err("El primer administrador ya está configurado.".into());
        }
        let password_hash = hash_password(input.password).await?;
        let master_hash = hash_password(input.master_password).await?;
        let user = User {
            id: random_id(),
            name: input.name.trim().to_string(),
            access_level: "complete".into(),
        };
        let mut tx = pool.begin().await.map_err(|_| DATABASE_ERROR)?;
        sqlx::query("INSERT INTO access_settings(id,master_hash) VALUES (1,?)")
            .bind(master_hash)
            .execute(&mut *tx)
            .await
            .map_err(
                |_| "La configuración ya existe o no pudo guardarse. Vuelve a abrir la aplicación.",
            )?;
        sqlx::query("INSERT INTO users(id,name,password_hash,access_level,login_name) VALUES (?,?,?,'complete',?)")
            .bind(&user.id).bind(&user.name).bind(password_hash).bind(key).execute(&mut *tx).await.map_err(|_| DATABASE_ERROR)?;
        tx.commit().await.map_err(|_| DATABASE_ERROR)?;
        let mut session = self.0.lock().await;
        session.user = Some(user.clone());
        session.failures = 0;
        session.retry_at = None;
        Ok(user)
    }

    async fn login(
        &self,
        pool: &SqlitePool,
        name: String,
        password: String,
    ) -> Result<User, String> {
        let key = login_name(&name)?;
        if password.is_empty() || password.len() > 1024 {
            return Err(INVALID_LOGIN.into());
        }
        let _gate = self.1.lock().await;
        {
            let mut session = self.0.lock().await;
            if session.retry_at.is_some_and(|time| time > Instant::now()) {
                return Err("Espera unos segundos antes de volver a intentar el acceso.".into());
            }
            session.user = None;
        }
        let row = sqlx::query(
            "SELECT id,name,password_hash,access_level FROM users WHERE login_name=? AND active=1",
        )
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|_| DATABASE_ERROR)?;
        let valid = if let Some(row) = &row {
            verify_password(password, row.get("password_hash")).await?
        } else {
            false
        };
        if !valid {
            let mut session = self.0.lock().await;
            session.failures += 1;
            if session.failures >= 5 {
                session.retry_at = Some(Instant::now() + Duration::from_secs(5));
            }
            return Err(INVALID_LOGIN.into());
        }
        let row = row.ok_or(INVALID_LOGIN)?;
        let user = User {
            id: row.get("id"),
            name: row.get("name"),
            access_level: row.get("access_level"),
        };
        let mut session = self.0.lock().await;
        session.user = Some(user.clone());
        session.failures = 0;
        session.retry_at = None;
        Ok(user)
    }
}

#[tauri::command]
pub async fn access_status(
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<AccessStatus, String> {
    state.status(&pool(&instances).await?).await
}

#[tauri::command]
pub async fn first_setup(
    input: FirstSetup,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<User, String> {
    state.first_setup(&pool(&instances).await?, input).await
}

#[tauri::command]
pub async fn login(
    name: String,
    password: String,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<User, String> {
    state.login(&pool(&instances).await?, name, password).await
}

#[tauri::command]
pub async fn logout(state: State<'_, AccessState>) -> Result<(), String> {
    let _gate = state.1.lock().await;
    state.0.lock().await.user = None;
    Ok(())
}

#[tauri::command]
pub async fn access_permissions(
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<Vec<String>, String> {
    let pool = pool(&instances).await?;
    let mut db = pool.acquire().await.map_err(|_| DATABASE_ERROR)?;
    let mut result = Vec::new();
    for (id, _, _) in crate::permissions::OPERATIONS {
        if state.authorize(&mut db, id).await.is_ok() {
            result.push(id.to_string());
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn database() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        for schema in [
            include_str!("../migrations/001_access.sql"),
            include_str!("../migrations/002_workshop.sql"),
            include_str!("../migrations/003_authentication.sql"),
        ] {
            sqlx::raw_sql(schema).execute(&pool).await.unwrap();
        }
        pool
    }
    fn input() -> FirstSetup {
        FirstSetup {
            name: "  Héctor  ".into(),
            password: "test-user-123".into(),
            confirmation: "test-user-123".into(),
            master_password: "test-master-123".into(),
            master_confirmation: "test-master-123".into(),
        }
    }
    #[test]
    fn session_refresh_does_not_block_authorization_on_a_busy_single_connection_pool() {
        tauri::async_runtime::block_on(async {
            let pool = database().await;
            let state = AccessState::test_admin(&pool).await;
            let mut db = pool.acquire().await.unwrap();
            let mut refresh = Box::pin(state.status(&pool));
            std::future::poll_fn(|context| {
                assert!(std::future::Future::poll(refresh.as_mut(), context).is_pending());
                std::task::Poll::Ready(())
            })
            .await;
            let authorized = tokio::time::timeout(
                Duration::from_secs(2),
                state.authorize(&mut db, "clients.view"),
            )
            .await;
            assert!(authorized.unwrap().is_ok());
            drop(db);
            assert!(refresh.await.unwrap().user.is_some());
            pool.close().await;
        });
    }

    #[test]
    fn setup_login_logout_and_revocation() {
        tauri::async_runtime::block_on(async {
            let pool = database().await;
            let state = AccessState::default();
            assert!(!state.status(&pool).await.unwrap().setup_complete);
            let user = state.first_setup(&pool, input()).await.unwrap();
            assert_eq!(user.name, "Héctor");
            let hash: String = sqlx::query_scalar("SELECT password_hash FROM users")
                .fetch_one(&pool)
                .await
                .unwrap();
            assert!(hash.starts_with("$argon2id$"));
            assert!(!hash.contains("test-user"));
            let master: String = sqlx::query_scalar("SELECT master_hash FROM access_settings")
                .fetch_one(&pool)
                .await
                .unwrap();
            assert!(master.starts_with("$argon2id$"));
            assert_ne!(hash, master);
            assert!(state.first_setup(&pool, input()).await.is_err());
            state.0.lock().await.user = None;
            assert!(state.status(&pool).await.unwrap().user.is_none());
            assert!(
                state
                    .login(&pool, "Héctor".into(), "wrong".into())
                    .await
                    .is_err()
            );
            assert!(
                state
                    .login(&pool, "missing".into(), "test-user-123".into())
                    .await
                    .is_err()
            );
            assert!(
                state
                    .login(&pool, "Héctor".into(), "test-master-123".into())
                    .await
                    .is_err()
            );
            assert_eq!(
                state
                    .login(&pool, " HÉCTOR ".into(), "test-user-123".into())
                    .await
                    .unwrap(),
                user
            );
            assert!(
                AccessState::default()
                    .status(&pool)
                    .await
                    .unwrap()
                    .user
                    .is_none(),
                "Sessions must not survive a process restart"
            );
            sqlx::query("INSERT INTO users(id,name,password_hash,access_level,login_name) VALUES ('b','Other',?,'complete','other')").bind(hash).execute(&pool).await.unwrap();
            sqlx::query("UPDATE users SET active=0 WHERE id=?")
                .bind(&user.id)
                .execute(&pool)
                .await
                .unwrap();
            assert!(state.status(&pool).await.unwrap().user.is_none());
            assert!(
                state
                    .login(&pool, "Héctor".into(), "test-user-123".into())
                    .await
                    .is_err()
            );
            pool.close().await;
        });
    }
    #[test]
    fn accounts_survive_database_reopen_without_restoring_session() {
        tauri::async_runtime::block_on(async {
            let directory = std::env::temp_dir().join(format!("hrl-auth-{}", random_id()));
            std::fs::create_dir_all(&directory).unwrap();
            let file = directory.join("test.db");
            let options = sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&file)
                .create_if_missing(true);
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options.clone())
                .await
                .unwrap();
            for schema in [
                include_str!("../migrations/001_access.sql"),
                include_str!("../migrations/002_workshop.sql"),
                include_str!("../migrations/003_authentication.sql"),
            ] {
                sqlx::raw_sql(schema).execute(&pool).await.unwrap();
            }
            AccessState::default()
                .first_setup(&pool, input())
                .await
                .unwrap();
            pool.close().await;
            drop(pool);
            let reopened = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .unwrap();
            let state = AccessState::default();
            let status = state.status(&reopened).await.unwrap();
            assert!(status.setup_complete);
            assert!(status.user.is_none());
            assert_eq!(
                state
                    .login(&reopened, "Héctor".into(), "test-user-123".into())
                    .await
                    .unwrap()
                    .access_level,
                "complete"
            );
            reopened.close().await;
            drop(reopened);
            std::fs::remove_dir_all(directory).unwrap();
        });
    }

    #[test]
    fn permission_checks_reject_unauthenticated_limited_and_revoked_operations() {
        tauri::async_runtime::block_on(async {
            let pool = database().await;
            let state = AccessState::default();
            {
                let mut db = pool.acquire().await.unwrap();
                assert!(state.authorize(&mut db, "clients.view").await.is_err());
            }
            let admin = state.first_setup(&pool, input()).await.unwrap();
            let account = |name: &str, role: &str| crate::users::UserInput {
                id: None,
                name: name.into(),
                password: "test-account-123".into(),
                confirmation: "test-account-123".into(),
                access_level: role.into(),
                active: true,
                permissions: vec![],
                master_password: "test-master-123".into(),
            };
            let mut wrong = account("Wrong", "limited");
            wrong.master_password = "wrong".into();
            assert!(crate::users::save(&pool, &state, wrong).await.is_err());
            crate::users::save(&pool, &state, account("Técnico", "limited"))
                .await
                .unwrap();
            assert!(
                crate::users::save(&pool, &state, account("TÉCNICO", "limited"))
                    .await
                    .is_err()
            );
            let mut custom = account("Consulta", "custom");
            custom.permissions.push(crate::users::Grant {
                section: "clients".into(),
                action: "view".into(),
            });
            crate::users::save(&pool, &state, custom).await.unwrap();
            state
                .login(&pool, "Técnico".into(), "test-account-123".into())
                .await
                .unwrap();
            {
                let mut db = pool.acquire().await.unwrap();
                for (id, _, _) in crate::permissions::OPERATIONS {
                    assert_eq!(
                        state.authorize(&mut db, id).await.is_ok(),
                        crate::permissions::limited_allows(id),
                        "{id}"
                    );
                }
            }
            assert!(
                crate::users::save(&pool, &state, account("Escalation", "complete"))
                    .await
                    .is_err()
            );
            let custom = state
                .login(&pool, "Consulta".into(), "test-account-123".into())
                .await
                .unwrap();
            {
                let mut db = pool.acquire().await.unwrap();
                assert!(state.authorize(&mut db, "clients.view").await.is_ok());
                assert!(state.authorize(&mut db, "orders.edit").await.is_err());
                assert!(state.authorize(&mut db, "users.manage").await.is_err());
            }
            sqlx::query("DELETE FROM user_permissions WHERE user_id=?")
                .bind(&custom.id)
                .execute(&pool)
                .await
                .unwrap();
            {
                let mut db = pool.acquire().await.unwrap();
                assert!(state.authorize(&mut db, "clients.view").await.is_err());
            }
            state
                .login(&pool, "Héctor".into(), "test-user-123".into())
                .await
                .unwrap();
            let mut deactivate = account("Héctor", "complete");
            deactivate.id = Some(admin.id);
            deactivate.active = false;
            deactivate.password.clear();
            assert!(
                crate::users::save(&pool, &state, deactivate).await.is_err(),
                "Last administrator must remain active"
            );
            pool.close().await;
        });
    }

    #[test]
    fn validation_and_failed_setup_leave_no_partial_accounts() {
        tauri::async_runtime::block_on(async {
            let pool = database().await;
            let state = AccessState::default();
            let mut invalid = input();
            invalid.confirmation = "different".into();
            assert!(state.first_setup(&pool, invalid).await.is_err());
            let mut invalid = input();
            invalid.master_password = "short".into();
            assert!(state.first_setup(&pool, invalid).await.is_err());
            sqlx::raw_sql("CREATE TRIGGER reject_user BEFORE INSERT ON users BEGIN SELECT RAISE(ABORT,'Test failure'); END").execute(&pool).await.unwrap();
            assert!(state.first_setup(&pool, input()).await.is_err());
            assert_eq!(
                sqlx::query_scalar::<_, i64>("SELECT count(*) FROM access_settings")
                    .fetch_one(&pool)
                    .await
                    .unwrap(),
                0
            );
            assert!(!state.status(&pool).await.unwrap().setup_complete);
            pool.close().await;
        });
    }
}
