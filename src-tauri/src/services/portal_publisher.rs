use super::{portal::PortalService, portal_transport::*};
use crate::{
    domain::{contracts::AccessScope, portal::fingerprint_document, security::AccessLevel},
    error::AppError,
    repositories::{
        audit::AuditRecord,
        portal::{PortalConfig, PublicationRecord},
    },
};
use rand_core::{OsRng, RngCore};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroizing;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublisherLogin {
    pub email: String,
    pub password: String,
}

fn validation(message: &str) -> AppError {
    AppError::Validation(message.into())
}
fn remote_error(error: RemoteError) -> AppError {
    validation(match error {
        RemoteError::Transient => "No se pudo contactar con Firebase. La cola local se conserva.",
        RemoteError::Authentication => "Vuelve a conectar la cuenta publicadora de Firebase.",
        RemoteError::Denied => {
            "Firebase denegó el acceso. Revisa el UID y las reglas del proyecto."
        }
        RemoteError::Invalid => {
            "Firebase rechazó la solicitud. Revisa la configuración y el contrato."
        }
        RemoteError::Missing => "El documento remoto no está disponible.",
    })
}
fn now() -> Result<i64, AppError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| validation("Revisa el reloj de Windows."))?
        .as_secs() as i64)
}
pub fn retry_delay(attempts: i64, random: u32) -> i64 {
    let base = (1_i64 << attempts.clamp(1, 9)).min(300);
    (base + i64::from(random) % (base / 4 + 1)).min(300)
}

impl PortalService {
    async fn manager(&self, db: &mut sqlx::SqliteConnection, write: bool) -> Result<i64, AppError> {
        let user = self
            .access
            .require(
                db,
                if write {
                    "company.edit"
                } else {
                    "company.view"
                },
                AccessScope::Company,
                None,
                write,
            )
            .await?;
        if user.access_level != AccessLevel::Complete {
            return Err(AppError::Forbidden);
        }
        Ok(user.id)
    }
    pub async fn settings(&self) -> Result<Value, AppError> {
        let _access = self.access.lock().await;
        let mut db = self.pool.acquire().await?;
        self.manager(&mut db, false).await?;
        let config = self.repository.config(&mut db).await?;
        let connected =
            self.runtime.vault.load().ok().flatten().is_some_and(|v| {
                v.project_id == config.project_id && v.uid == config.publisher_uid
            });
        Ok(
            json!({"config":config,"connected":connected,"error":self.runtime.error.lock().await.clone()}),
        )
    }
    pub async fn configure(&self, mut config: PortalConfig) -> Result<(), AppError> {
        validate_config(&config)?;
        config.paused = true;
        let _job = self.runtime.gate.lock().await;
        let _lease = self.runtime.try_lease()?;
        let _access = self.access.lock().await;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let actor = self.manager(&mut tx, true).await?;
        let old = self.repository.config(&mut tx).await?;
        if !old.project_id.is_empty()
            && (old.project_id != config.project_id || old.publisher_uid != config.publisher_uid)
            && (self.runtime.vault.load()?.is_some()
                || self.repository.has_confirmed(&mut tx).await?)
        {
            return Err(validation(
                "El cambio de proyecto o emisor requiere una transferencia acordada; no se sobrescribe desde esta configuración.",
            ));
        }
        self.repository.configure(&mut tx, &config).await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: actor,
                    operation: "company.edit",
                    entity: "portal_configuracion",
                    entity_id: Some(1),
                    order_id: None,
                    reason: None,
                },
                &[(
                    "publisherConfiguration",
                    None,
                    Some("Configurada; envío pausado".into()),
                )],
            )
            .await?;
        tx.commit().await?;
        *self.runtime.session.lock().await = None;
        Ok(())
    }
    pub async fn connect(&self, input: PublisherLogin) -> Result<(), AppError> {
        let password = Zeroizing::new(input.password);
        if input.email.len() > 320
            || !input.email.contains('@')
            || password.is_empty()
            || password.len() > 1024
        {
            return Err(validation("Revisa el correo y la contraseña de Firebase."));
        }
        let _job = self.runtime.gate.lock().await;
        let _lease = self.runtime.try_lease()?;
        let config = {
            let _access = self.access.lock().await;
            let mut db = self.pool.acquire().await?;
            self.manager(&mut db, true).await?;
            let c = self.repository.config(&mut db).await?;
            validate_config(&c)?;
            c
        };
        let tokens = self
            .runtime
            .gateway
            .login(&config, input.email.trim(), &password)
            .await
            .map_err(remote_error)?;
        if tokens.uid != config.publisher_uid {
            return Err(validation(
                "La cuenta no coincide con el UID autorizado para publicar.",
            ));
        }
        let _access = self.access.lock().await;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let actor = self.manager(&mut tx, true).await?;
        self.repository.pause(&mut tx).await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: actor,
                    operation: "company.edit",
                    entity: "portal_configuracion",
                    entity_id: Some(1),
                    order_id: None,
                    reason: None,
                },
                &[(
                    "publisherConnection",
                    None,
                    Some("Cuenta verificada; pendiente de reanudar".into()),
                )],
            )
            .await?;
        let previous = self.runtime.vault.load()?;
        self.runtime.store(&config, tokens).await?;
        if let Err(error) = tx.commit().await {
            match previous {
                Some(v) => self.runtime.vault.save(&v)?,
                None => self.runtime.vault.clear()?,
            };
            *self.runtime.session.lock().await = None;
            return Err(error.into());
        }
        Ok(())
    }
    pub async fn disconnect(&self) -> Result<(), AppError> {
        self.pause().await?;
        let _job = self.runtime.gate.lock().await;
        let _lease = self.runtime.try_lease()?;
        let _access = self.access.lock().await;
        let mut db = self.pool.acquire().await?;
        self.manager(&mut db, true).await?;
        self.runtime.vault.clear()?;
        *self.runtime.session.lock().await = None;
        Ok(())
    }
    pub async fn pause(&self) -> Result<(), AppError> {
        let _job = self.runtime.gate.lock().await;
        let _lease = self.runtime.try_lease()?;
        let _access = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        let actor = self.manager(&mut tx, true).await?;
        self.repository.pause(&mut tx).await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: actor,
                    operation: "company.edit",
                    entity: "portal_configuracion",
                    entity_id: Some(1),
                    order_id: None,
                    reason: None,
                },
                &[("publisherPaused", None, Some("true".into()))],
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn resume(&self) -> Result<(), AppError> {
        let _job = self.runtime.gate.lock().await;
        let _lease = self.runtime.try_lease()?;
        let (mut config, pending) = {
            let _access = self.access.lock().await;
            let mut db = self.pool.acquire().await?;
            self.manager(&mut db, true).await?;
            (
                self.repository.config(&mut db).await?,
                self.repository.pending_codes(&mut db).await?,
            )
        };
        validate_config(&config)?;
        let token = self
            .runtime
            .token(&config, false)
            .await
            .map_err(remote_error)?;
        for id in pending {
            let record = self
                .repository
                .load(&mut *self.pool.acquire().await?, id)
                .await?;
            match self
                .runtime
                .gateway
                .read(&config, &token, &record.code)
                .await
            {
                Ok(remote) => {
                    if let Some(state) = reconciled_state(&record, &remote) {
                        self.transition(
                            &record,
                            state,
                            None,
                            0,
                            if state == "confirmed" {
                                Some(&remote.updated_at)
                            } else {
                                None
                            },
                        )
                        .await?;
                        if state == "conflict" {
                            return Err(validation(
                                "Hay una revisión remota con otra huella. Reconcilia esa orden antes de reanudar.",
                            ));
                        }
                    }
                }
                Err(RemoteError::Missing) => {}
                Err(error) => return Err(remote_error(error)),
            }
        }
        let _access = self.access.lock().await;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let actor = self.manager(&mut tx, true).await?;
        config.paused = false;
        self.repository.configure(&mut tx, &config).await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: actor,
                    operation: "company.edit",
                    entity: "portal_configuracion",
                    entity_id: Some(1),
                    order_id: None,
                    reason: None,
                },
                &[(
                    "publisherPaused",
                    None,
                    Some("false; revisiones remotas consultadas".into()),
                )],
            )
            .await?;
        tx.commit().await?;
        *self.runtime.error.lock().await = None;
        Ok(())
    }
    pub async fn reconcile_order(&self, id: i64) -> Result<Value, AppError> {
        let _job = self.runtime.gate.lock().await;
        let _lease = self.runtime.try_lease()?;
        let (config, code) = {
            let _access = self.access.lock().await;
            let mut db = self.pool.acquire().await?;
            self.access
                .require(
                    &mut db,
                    "publication.edit",
                    AccessScope::Repairs,
                    Some(id),
                    false,
                )
                .await?;
            (
                self.repository.config(&mut db).await?,
                self.repository.code(&mut db, id).await?.ok_or(validation(
                    "Publica la orden localmente antes de reconciliarla.",
                ))?,
            )
        };
        let token = self
            .runtime
            .token(&config, false)
            .await
            .map_err(remote_error)?;
        let remote = self.runtime.gateway.read(&config, &token, &code).await;
        let revision = match remote {
            Ok(r) => r.revision,
            Err(RemoteError::Missing) => 0,
            Err(e) => return Err(remote_error(e)),
        };
        if revision >= 9_007_199_254_740_991 {
            return Err(validation("La revisión remota supera el rango permitido."));
        }
        {
            let _access = self.access.lock().await;
            let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
            let actor = self
                .access
                .require(
                    &mut tx,
                    "publication.edit",
                    AccessScope::Repairs,
                    Some(id),
                    true,
                )
                .await?;
            if self.repository.code(&mut tx, id).await?.as_deref() != Some(&code) {
                return Err(AppError::Conflict(
                    "El código cambió; vuelve a consultar la orden.".into(),
                ));
            }
            self.repository.set_revision(&mut tx, id, revision).await?;
            self.repository.enabled(&mut tx, id, true).await?;
            self.queue.capture(&mut tx, id, actor.id, true).await?;
            tx.commit().await?;
        }
        drop(_job);
        self.status(id).await
    }
    async fn transition(
        &self,
        r: &PublicationRecord,
        state: &str,
        error: Option<&str>,
        next: i64,
        confirmed: Option<&str>,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let actor = self.repository.author(&mut tx, r).await?;
        self.repository
            .result(&mut tx, r.id, state, error, next, confirmed)
            .await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: actor,
                    operation: "publication.edit",
                    entity: "portal_envios",
                    entity_id: Some(r.id),
                    order_id: Some(r.order_id),
                    reason: error,
                },
                &[("publicationState", None, Some(state.into()))],
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn pump(&self) -> Result<bool, AppError> {
        let _job = self.runtime.gate.lock().await;
        let _lease = self.runtime.try_lease()?;
        let (config, unfinished) = {
            let mut db = self.pool.acquire().await?;
            (
                self.repository.config(&mut db).await?,
                self.repository.unfinished(&mut db).await?,
            )
        };
        for id in unfinished {
            let r = self
                .repository
                .load(&mut *self.pool.acquire().await?, id)
                .await?;
            self.transition(
                &r,
                "pending",
                Some("Envío recuperado; se comprobará la revisión remota."),
                0,
                None,
            )
            .await?;
        }
        if config.paused || config.project_id.is_empty() {
            return Ok(false);
        }
        if self.runtime.vault.load()?.is_none() {
            return Ok(false);
        }
        let id = self
            .repository
            .next(&mut *self.pool.acquire().await?, now()?)
            .await?;
        let Some(id) = id else { return Ok(false) };
        let record = self
            .repository
            .load(&mut *self.pool.acquire().await?, id)
            .await?;
        let expected = record.document["huella"]
            .as_str()
            .ok_or(validation("Falta la huella de la publicación."))?;
        if fingerprint_document(&record.document).map_err(|e| validation(&e.to_string()))?
            != expected
        {
            self.transition(
                &record,
                "error",
                Some("La revisión local no coincide con su huella; no se enviará."),
                0,
                None,
            )
            .await?;
            return Ok(true);
        }
        self.transition(&record, "sending", None, 0, None).await?;
        let result = match self.runtime.token(&config, false).await {
            Ok(token) => {
                let sent = self
                    .runtime
                    .gateway
                    .commit(&config, &token, &record.document)
                    .await;
                if matches!(sent, Err(RemoteError::Authentication)) {
                    match self.runtime.token(&config, true).await {
                        Ok(token) => {
                            self.runtime
                                .gateway
                                .commit(&config, &token, &record.document)
                                .await
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    sent
                }
            }
            Err(e) => Err(e),
        };
        match result {
            Ok(at)=>self.transition(&record,"confirmed",None,0,Some(&at)).await?,
            Err(RemoteError::Denied)=>{
                let check=match self.runtime.token(&config,false).await{Ok(token)=>self.runtime.gateway.read(&config,&token,&record.code).await,Err(e)=>Err(e)};
                match check{
                    Ok(remote)=>{let state=reconciled_state(&record,&remote).unwrap_or("error");let message=match state{"confirmed"|"superseded"=>None,"conflict"=>Some("La misma revisión remota tiene otra huella. Reconcilia la orden."),_=>Some("Firebase rechazó el contenido. Revisa las reglas y el contrato.")};
                        self.transition(&record,state,message,0,if state=="confirmed"{Some(&remote.updated_at)}else{None}).await?;},
                    Err(RemoteError::Transient)=>self.retry(&record).await?,
                    Err(RemoteError::Authentication)=>self.auth_required(&record).await?,
                    Err(_)=>self.transition(&record,"error",Some("Firebase denegó el acceso o rechazó el contenido. Revisa el UID y las reglas."),0,None).await?,
                }
            },
            Err(RemoteError::Transient)=>self.retry(&record).await?,
            Err(RemoteError::Authentication)=>self.auth_required(&record).await?,
            Err(_)=>self.transition(&record,"error",Some("Firebase rechazó el documento; corrige el contenido antes de publicar de nuevo."),0,None).await?,
        }
        Ok(true)
    }
    async fn retry(&self, record: &PublicationRecord) -> Result<(), AppError> {
        let delay = retry_delay(record.attempts + 1, OsRng.next_u32());
        self.transition(
            record,
            "pending",
            Some("Sin confirmación de Firebase; se reintentará automáticamente."),
            now()? + delay,
            None,
        )
        .await
    }
    async fn auth_required(&self, record: &PublicationRecord) -> Result<(), AppError> {
        self.transition(
            record,
            "pending",
            Some("Reconecta la cuenta publicadora para continuar."),
            0,
            None,
        )
        .await?;
        let mut tx = self.pool.begin().await?;
        let actor = self.repository.author(&mut tx, record).await?;
        self.repository.pause(&mut tx).await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: actor,
                    operation: "publication.edit",
                    entity: "portal_configuracion",
                    entity_id: Some(1),
                    order_id: Some(record.order_id),
                    reason: Some("Credencial publicadora no disponible"),
                },
                &[("publisherPaused", None, Some("true".into()))],
            )
            .await?;
        tx.commit().await?;
        *self.runtime.session.lock().await = None;
        *self.runtime.error.lock().await =
            Some("Reconecta la cuenta publicadora para continuar.".into());
        Ok(())
    }
    pub fn start_worker(self: &Arc<Self>) {
        if self.runtime.started.swap(true, Ordering::SeqCst) {
            return;
        }
        let weak = Arc::downgrade(self);
        tauri::async_runtime::spawn(async move {
            loop {
                let Some(service) = weak.upgrade() else { break };
                if service.pool.is_closed() {
                    break;
                }
                let immediate = match service.pump().await {
                    Ok(value) => value,
                    Err(error) => {
                        *service.runtime.error.lock().await = Some(error.to_string());
                        false
                    }
                };
                drop(service);
                tokio::time::sleep(Duration::from_secs(if immediate { 1 } else { 3 })).await;
            }
        });
    }
}

pub fn reconciled_state(
    record: &PublicationRecord,
    remote: &RemoteDocument,
) -> Option<&'static str> {
    if remote.revision > record.revision {
        Some("superseded")
    } else if remote.revision == record.revision {
        Some(
            if record.document["huella"].as_str() == Some(&remote.fingerprint) {
                "confirmed"
            } else {
                "conflict"
            },
        )
    } else {
        None
    }
}
fn validate_config(config: &PortalConfig) -> Result<(), AppError> {
    if !(3..=63).contains(&config.project_id.len())
        || !config
            .project_id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || config.api_key.is_empty()
        || config.api_key.len() > 200
        || !config
            .api_key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        || config.publisher_uid.is_empty()
        || config.publisher_uid.len() > 128
    {
        return Err(validation(
            "Revisa el projectId, la apiKey y el UID del publicador.",
        ));
    }
    if !config.portal_url.is_empty() {
        let url = reqwest::Url::parse(&config.portal_url)
            .map_err(|_| validation("Escribe una URL HTTPS válida para el portal."))?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.query().is_some()
        {
            return Err(validation(
                "La URL del portal debe usar HTTPS, sin credenciales, parámetros ni fragmentos.",
            ));
        }
    }
    Ok(())
}
