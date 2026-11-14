use super::audit::AuditService;
use super::recovery::{
    account_fingerprint, configured_public_key, RecoveryState, CHALLENGE_TTL_SECONDS,
};
use crate::{
    domain::{
        contracts::*,
        security::{AccessLevel, User},
    },
    error::AppError,
    repositories::{access::AccessRepository, audit::AuditRecord},
};
use argon2::{
    password_hash::{PasswordHash, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use sqlx::{SqliteConnection, SqlitePool};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;
use tokio::sync::{Mutex, MutexGuard};
use zeroize::Zeroizing;

#[derive(Default)]
struct Attempts {
    failures: u32,
    retry_at: Option<Instant>,
}
impl Attempts {
    fn check(&self) -> Result<(), AppError> {
        if self.retry_at.is_some_and(|t| t > Instant::now()) {
            Err(AppError::RateLimited)
        } else {
            Ok(())
        }
    }
    fn fail(&mut self) {
        self.failures = self.failures.saturating_add(1);
        if self.failures >= 5 {
            self.retry_at = Some(Instant::now() + Duration::from_secs(5));
        }
    }
    fn reset(&mut self) {
        self.failures = 0;
        self.retry_at = None;
    }
}
#[derive(Default)]
struct Session {
    user_id: Option<i64>,
    grant: Option<TemporaryAccessResponse>,
    login_attempts: Attempts,
    approval_attempts: Attempts,
    master_attempts: Attempts,
}

pub struct AccessService {
    pool: SqlitePool,
    repository: Arc<AccessRepository>,
    audit: Arc<AuditService>,
    gate: Mutex<()>,
    session: Mutex<Session>,
    dummy_hash: String,
    recovery: Mutex<RecoveryState>,
}

fn validation(message: &str) -> AppError {
    AppError::Validation(message.into())
}
pub(crate) fn login_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control) {
        return Err(validation(
            "Escribe un usuario de 1 a 100 caracteres, sin caracteres de control.",
        ));
    }
    Ok(name.to_lowercase())
}
pub(crate) fn validate_password(password: &str, confirmation: &str) -> Result<(), AppError> {
    if password.chars().count() < 8 || password.len() > 1024 || password != confirmation {
        return Err(validation(
            "Las claves deben coincidir y tener entre 8 caracteres y 1024 bytes.",
        ));
    }
    Ok(())
}
pub(crate) async fn hash(password: &str) -> Result<String, AppError> {
    let password = Zeroizing::new(password.to_owned());
    tauri::async_runtime::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|hash| hash.to_string())
            .map_err(|_| AppError::Crypto)
    })
    .await
    .map_err(|_| AppError::Crypto)?
}
async fn verify(password: &str, encoded: &str) -> Result<bool, AppError> {
    let password = Zeroizing::new(password.to_owned());
    let encoded = encoded.to_owned();
    tauri::async_runtime::spawn_blocking(move || {
        let parsed = PasswordHash::new(&encoded).map_err(|_| AppError::Crypto)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    })
    .await
    .map_err(|_| AppError::Crypto)?
}
fn scope_matches(operation: &str, scope: AccessScope) -> bool {
    let Some((_, section, _)) = crate::domain::permissions::OPERATIONS
        .iter()
        .find(|(name, _, _)| *name == operation)
    else {
        return false;
    };
    (if *section == "payments" {
        "repairs"
    } else {
        section
    }) == scope.code()
        || scope == AccessScope::Clients && operation == "payments.add"
        || scope == AccessScope::Logbook && operation == "repairs.edit"
}

impl AccessService {
    pub async fn verify_master(
        &self,
        db: &mut SqliteConnection,
        password: &str,
    ) -> Result<(), AppError> {
        self.session.lock().await.master_attempts.check()?;
        if password.len() > 1024
            || !verify(password, &self.repository.master_hash(db).await?).await?
        {
            self.session.lock().await.master_attempts.fail();
            return Err(AppError::InvalidCredentials);
        }
        self.session.lock().await.master_attempts.reset();
        Ok(())
    }
    pub async fn refresh_session(&self) -> Result<SessionResponse, AppError> {
        let _gate = self.lock().await;
        let id = self
            .session
            .lock()
            .await
            .user_id
            .ok_or(AppError::Unauthorized)?;
        let mut tx = self.pool.begin().await?;
        let user = self
            .repository
            .by_id(&mut tx, id)
            .await?
            .ok_or(AppError::Unauthorized)?
            .user;
        let permissions = self.permanent(&mut tx, &user).await?;
        tx.commit().await?;
        Ok(SessionResponse { user, permissions })
    }
    pub async fn new(
        pool: SqlitePool,
        repository: Arc<AccessRepository>,
        audit: Arc<AuditService>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            pool,
            repository,
            audit,
            gate: Mutex::new(()),
            session: Mutex::new(Session::default()),
            dummy_hash: hash("HRL-dummy-unavailable-account").await?,
            recovery: Mutex::new(RecoveryState::default()),
        })
    }
    pub(crate) async fn lock(&self) -> MutexGuard<'_, ()> {
        self.gate.lock().await
    }
    pub async fn setup_status(&self) -> Result<SetupStatus, AppError> {
        let mut db = self.pool.acquire().await?;
        Ok(SetupStatus {
            setup_complete: self.repository.setup_status(&mut db).await?,
        })
    }
    pub async fn login_names(&self) -> Result<Vec<String>, AppError> {
        let mut db = self.pool.acquire().await?;
        self.repository.login_names(&mut db).await
    }
    pub async fn synchronize_window(
        &self,
        apply: impl FnOnce(bool, bool) -> Result<(), String>,
    ) -> Result<(), AppError> {
        let _gate = self.lock().await;
        let mut db = self.pool.acquire().await?;
        let configured = self.repository.setup_status(&mut db).await?;
        let user_id = self.session.lock().await.user_id;
        let authenticated = match user_id {
            Some(id) => self.repository.by_id(&mut db, id).await?.is_some(),
            None => false,
        };
        apply(authenticated, configured).map_err(|_| AppError::Window)
    }
    pub async fn generate_recovery_challenge(&self) -> Result<RecoveryChallengeResponse, AppError> {
        let _gate = self.lock().await;
        let mut tx = self.pool.begin().await?;
        let account = self
            .repository
            .recovery_account(&mut tx)
            .await?
            .ok_or(AppError::RecoveryUnavailable)?;
        tx.commit().await?;
        let challenge = self.recovery.lock().await.issue(&account)?;
        Ok(RecoveryChallengeResponse {
            challenge,
            admin_id: account.id,
            admin_username: account.login,
            expires_in_seconds: CHALLENGE_TTL_SECONDS,
        })
    }

    pub async fn verify_recovery_signature(
        &self,
        payload: VerifyRecoveryPayload,
    ) -> Result<RecoveryResponse, AppError> {
        validate_password(&payload.new_master_password, &payload.new_master_password)?;
        validate_password(&payload.new_admin_password, &payload.new_admin_password)?;
        if payload.challenge.len() > 200 || payload.signature.len() != 128 {
            return Err(AppError::RecoveryRejected);
        }
        let _gate = self.lock().await;
        let binding = self
            .recovery
            .lock()
            .await
            .consume(&payload.challenge, &payload.signature)?;
        let master_hash = hash(&payload.new_master_password).await?;
        let admin_hash = hash(&payload.new_admin_password).await?;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let account = self
            .repository
            .recovery_account(&mut tx)
            .await?
            .ok_or(AppError::RecoveryRejected)?;
        if account.id != binding.admin_id
            || !bool::from(account_fingerprint(&account).ct_eq(&binding.fingerprint))
        {
            return Err(AppError::RecoveryRejected);
        }
        self.repository
            .reset_recovery_credentials(&mut tx, account.id, &master_hash, &admin_hash)
            .await?;
        for (id, recipient) in self.repository.open_grants(&mut tx).await? {
            self.close(
                &mut tx,
                id,
                recipient,
                "Revocación por recuperación de soporte firmada",
            )
            .await?;
        }
        let key_id: String = configured_public_key()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let challenge_id: String = Sha256::digest(payload.challenge.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        // El esquema exige un usuario local como autor. Se identifica la cuenta afectada
        // y se declara la autorización externa, sin atribuirle una sesión autenticada.
        for (entity, id, field) in [
            ("claves_maestras", 1, "masterPasswordReset"),
            ("usuarios", account.id, "adminPasswordReset"),
        ] {
            self.audit.record(&mut tx, AuditRecord {
                author_id: account.id, operation: "users.edit", entity,
                entity_id: Some(id), order_id: None,
                reason: Some("Recuperación autorizada por firma Ed25519 del desarrollador; cuenta afectada, sin sesión de operador"),
            }, &[
                (field, None, Some("true".into())),
                ("authorization", None, Some("external_developer_ed25519".into())),
                ("supportPublicKey", None, Some(key_id.clone())),
                ("challengeDigest", None, Some(challenge_id.clone())),
            ]).await?;
        }
        // También ante un commit incierto se exige un login nuevo y se retiran facultades en memoria.
        let mut session = self.session.lock().await;
        session.user_id = None;
        session.grant = None;
        session.login_attempts.reset();
        session.approval_attempts.reset();
        drop(session);
        tx.commit().await.map_err(AppError::Commit)?;
        Ok(RecoveryResponse {
            admin_id: account.id,
            requires_login: true,
        })
    }
    pub async fn initialize(&self, payload: SetupPayload) -> Result<SessionResponse, AppError> {
        let login = login_name(&payload.username)?;
        login_name(&payload.name)?;
        validate_password(&payload.password, &payload.confirmation)?;
        validate_password(&payload.master_password, &payload.master_confirmation)?;
        let _gate = self.lock().await;
        let password_hash = hash(&payload.password).await?;
        let master_hash = hash(&payload.master_password).await?;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        if self.repository.setup_occupied(&mut tx).await? {
            return Err(AppError::Conflict(
                "El acceso inicial ya existe o está incompleto; no se sobrescribió.".into(),
            ));
        }
        let id = self
            .repository
            .bootstrap(
                &mut tx,
                payload.name.trim(),
                &login,
                &password_hash,
                &master_hash,
            )
            .await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: id,
                    operation: "users.create",
                    entity: "usuarios",
                    entity_id: Some(id),
                    order_id: None,
                    reason: Some("Configuración inicial del administrador y la clave maestra"),
                },
                &[("setupCompleted", None, Some("true".into()))],
            )
            .await?;
        let user = self
            .repository
            .by_id(&mut tx, id)
            .await?
            .ok_or(AppError::NotFound)?
            .user;
        let permissions = self.permanent(&mut tx, &user).await?;
        tx.commit().await.map_err(AppError::Commit)?;
        let mut session = self.session.lock().await;
        session.user_id = Some(id);
        session.grant = None;
        session.login_attempts.reset();
        Ok(SessionResponse { user, permissions })
    }
    pub async fn login(&self, payload: LoginPayload) -> Result<SessionResponse, AppError> {
        let login = login_name(&payload.username)?;
        if payload.password.is_empty() || payload.password.len() > 1024 {
            return Err(AppError::InvalidCredentials);
        }
        let _gate = self.lock().await;
        self.session.lock().await.login_attempts.check()?;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let account = self.repository.by_login(&mut tx, &login).await?;
        let valid = verify(
            &payload.password,
            account
                .as_ref()
                .map(|a| a.hash.as_str())
                .unwrap_or(&self.dummy_hash),
        )
        .await?;
        if !valid || account.is_none() {
            self.session.lock().await.login_attempts.fail();
            return Err(AppError::InvalidCredentials);
        }
        let user = account.ok_or(AppError::InvalidCredentials)?.user;
        if let Some(grant) = self.session.lock().await.grant.clone() {
            self.close(&mut tx, grant.id, grant.recipient_id, "Cambio de sesión")
                .await?;
        }
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: user.id,
                    operation: "users.edit",
                    entity: "usuarios",
                    entity_id: Some(user.id),
                    order_id: None,
                    reason: Some("Inicio de sesión"),
                },
                &[("sessionStarted", None, Some("true".into()))],
            )
            .await?;
        let permissions = self.permanent(&mut tx, &user).await?;
        tx.commit().await.map_err(AppError::Commit)?;
        let mut session = self.session.lock().await;
        session.user_id = Some(user.id);
        session.grant = None;
        session.login_attempts.reset();
        session.approval_attempts.reset();
        Ok(SessionResponse { user, permissions })
    }
    pub async fn permanent(
        &self,
        db: &mut SqliteConnection,
        user: &User,
    ) -> Result<Vec<String>, AppError> {
        let grants = self.repository.permissions(db, user.id).await?;
        Ok(crate::domain::permissions::OPERATIONS
            .iter()
            .filter(|(operation, section, action)| match user.access_level {
                AccessLevel::Complete => true,
                AccessLevel::Limited if *section != "payments" => {
                    crate::domain::permissions::limited_allows(operation)
                }
                AccessLevel::Limited | AccessLevel::Custom
                    if *operation != "users.manage" && *operation != "payments.adjust" =>
                {
                    grants
                        .iter()
                        .any(|g| g.section == *section && g.action == *action)
                }
                _ => false,
            })
            .map(|(name, _, _)| name.to_string())
            .collect())
    }
    pub async fn require(
        &self,
        db: &mut SqliteConnection,
        operation: &str,
        scope: AccessScope,
        order: Option<i64>,
        audit_use: bool,
    ) -> Result<User, AppError> {
        let (id, grant) = {
            let session = self.session.lock().await;
            (
                session.user_id.ok_or(AppError::Unauthorized)?,
                session.grant.clone(),
            )
        };
        let user = self
            .repository
            .by_id(db, id)
            .await?
            .ok_or(AppError::Unauthorized)?
            .user;
        if self
            .permanent(db, &user)
            .await?
            .iter()
            .any(|name| name == operation)
        {
            return Ok(user);
        }
        let grant = grant.ok_or(AppError::Forbidden)?;
        if grant.recipient_id != id
            || grant.scope != scope
            || grant.order_id.is_some_and(|id| Some(id) != order)
            || !scope_matches(operation, scope)
            || !grant.operations.iter().any(|name| name == operation)
        {
            return Err(AppError::Forbidden);
        }
        let live = self
            .repository
            .live_grant(db, grant.id)
            .await?
            .ok_or(AppError::Forbidden)?;
        if live.recipient != id
            || live.approver != grant.approver_id
            || live.scope != scope.code()
            || live.order_id != grant.order_id
        {
            return Err(AppError::Forbidden);
        }
        let approver = self
            .repository
            .by_id(db, live.approver)
            .await?
            .ok_or(AppError::Forbidden)?
            .user;
        if !self
            .permanent(db, &approver)
            .await?
            .iter()
            .any(|name| name == operation)
        {
            return Err(AppError::Forbidden);
        }
        let (_, section, action) = crate::domain::permissions::OPERATIONS
            .iter()
            .find(|(name, _, _)| *name == operation)
            .ok_or(AppError::Forbidden)?;
        let permission = self.repository.permission_id(db, section, action).await?;
        if !self
            .repository
            .grant_contains(db, grant.id, permission)
            .await?
        {
            return Err(AppError::Forbidden);
        }
        if audit_use && !operation.ends_with(".view") {
            self.audit
                .record(
                    db,
                    AuditRecord {
                        author_id: id,
                        operation: "access.temporary.use",
                        entity: "accesos_temporales",
                        entity_id: Some(grant.id),
                        order_id: order,
                        reason: None,
                    },
                    &[
                        ("authorizedBy", None, Some(approver.id.to_string())),
                        ("temporaryOperation", None, Some(operation.to_owned())),
                    ],
                )
                .await?;
        }
        Ok(user)
    }
    pub async fn temporary(
        &self,
        payload: TemporaryAccessPayload,
    ) -> Result<TemporaryAccessResponse, AppError> {
        let login = login_name(&payload.username)?;
        if !scope_matches(&payload.operation, payload.scope)
            || payload.reason.trim().is_empty()
            || payload.reason.len() > 2000
            || payload.password.is_empty()
            || payload.password.len() > 1024
            || (payload.scope == AccessScope::Repairs) != payload.order_id.is_some()
            || payload.order_id.is_some_and(|id| id <= 0)
        {
            return Err(validation(
                "Revisa el apartado, folio, acción y motivo de la autorización.",
            ));
        }
        let _gate = self.lock().await;
        let (recipient, previous) = {
            let s = self.session.lock().await;
            s.approval_attempts.check()?;
            (s.user_id.ok_or(AppError::Unauthorized)?, s.grant.clone())
        };
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        self.repository
            .by_id(&mut tx, recipient)
            .await?
            .ok_or(AppError::Unauthorized)?;
        let account = self.repository.by_login(&mut tx, &login).await?;
        let valid = verify(
            &payload.password,
            account
                .as_ref()
                .map(|a| a.hash.as_str())
                .unwrap_or(&self.dummy_hash),
        )
        .await?;
        if !valid || account.is_none() {
            self.session.lock().await.approval_attempts.fail();
            return Err(AppError::InvalidCredentials);
        }
        let approver = account.ok_or(AppError::InvalidCredentials)?.user;
        if approver.id == recipient {
            return Err(validation("La autorización debe concederla otra persona."));
        }
        let operations: Vec<String> = self
            .permanent(&mut tx, &approver)
            .await?
            .into_iter()
            .filter(|name| scope_matches(name, payload.scope))
            .collect();
        if !operations.contains(&payload.operation) {
            return Err(AppError::Forbidden);
        }
        if let Some(order) = payload.order_id {
            if !self.repository.order_exists(&mut tx, order).await? {
                return Err(AppError::NotFound);
            }
        }
        if let Some(previous) = previous {
            self.close(
                &mut tx,
                previous.id,
                recipient,
                "Reemplazada por otra autorización",
            )
            .await?;
        }
        let id = self
            .repository
            .grant(
                &mut tx,
                recipient,
                approver.id,
                payload.scope.code(),
                payload.order_id,
                payload.reason.trim(),
            )
            .await?;
        for operation in &operations {
            let (_, section, action) = crate::domain::permissions::OPERATIONS
                .iter()
                .find(|(name, _, _)| *name == operation)
                .ok_or(AppError::Forbidden)?;
            let permission = self
                .repository
                .permission_id(&mut tx, section, action)
                .await?;
            self.repository
                .grant_permission(&mut tx, id, permission)
                .await?;
        }
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: recipient,
                    operation: "access.temporary.start",
                    entity: "accesos_temporales",
                    entity_id: Some(id),
                    order_id: payload.order_id,
                    reason: Some(payload.reason.trim()),
                },
                &[
                    ("authorizedBy", None, Some(approver.id.to_string())),
                    ("scope", None, Some(payload.scope.code().into())),
                ],
            )
            .await?;
        tx.commit().await.map_err(AppError::Commit)?;
        let result = TemporaryAccessResponse {
            id,
            recipient_id: recipient,
            approver_id: approver.id,
            scope: payload.scope,
            order_id: payload.order_id,
            operations,
        };
        let mut session = self.session.lock().await;
        session.grant = Some(result.clone());
        session.approval_attempts.reset();
        Ok(result)
    }
    async fn close(
        &self,
        db: &mut SqliteConnection,
        id: i64,
        recipient: i64,
        reason: &str,
    ) -> Result<(), AppError> {
        if self.repository.close_grant(db, id, reason).await? {
            self.audit
                .record(
                    db,
                    AuditRecord {
                        author_id: recipient,
                        operation: "access.temporary.end",
                        entity: "accesos_temporales",
                        entity_id: Some(id),
                        order_id: None,
                        reason: Some(reason),
                    },
                    &[],
                )
                .await?;
        }
        Ok(())
    }
    pub async fn finish_temporary(&self) -> Result<(), AppError> {
        let _gate = self.lock().await;
        let grant = self.session.lock().await.grant.take();
        if let Some(grant) = grant {
            let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
            self.close(
                &mut tx,
                grant.id,
                grant.recipient_id,
                "Finalización explícita del acceso temporal",
            )
            .await?;
            tx.commit().await.map_err(AppError::Commit)?;
        }
        Ok(())
    }
    pub async fn logout(&self) -> Result<(), AppError> {
        let _gate = self.lock().await;
        let (user, grant) = {
            let mut s = self.session.lock().await;
            (s.user_id.take(), s.grant.take())
        };
        if let Some(user) = user {
            let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
            if let Some(grant) = grant {
                self.close(&mut tx, grant.id, user, "Cierre de sesión")
                    .await?;
            }
            self.audit
                .record(
                    &mut tx,
                    AuditRecord {
                        author_id: user,
                        operation: "users.edit",
                        entity: "usuarios",
                        entity_id: Some(user),
                        order_id: None,
                        reason: Some("Cierre de sesión"),
                    },
                    &[("sessionStarted", Some("true".into()), Some("false".into()))],
                )
                .await?;
            tx.commit().await.map_err(AppError::Commit)?;
        }
        Ok(())
    }
    pub async fn recover(&self) -> Result<(), AppError> {
        let _gate = self.lock().await;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        for (id, recipient) in self.repository.open_grants(&mut tx).await? {
            self.close(
                &mut tx,
                id,
                recipient,
                "Fin detectado al recuperar el backend; hora exacta de cierre desconocida",
            )
            .await?;
        }
        tx.commit().await.map_err(AppError::Commit)?;
        Ok(())
    }
}
