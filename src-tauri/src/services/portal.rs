use super::{access::AccessService, audit::AuditService};
use crate::{
    domain::{contracts::AccessScope, portal::*},
    error::AppError,
    repositories::{
        audit::AuditRecord, orders::OrderRepository, portal::*, repairs::RepairRepository,
    },
};
use rand_core::{OsRng, RngCore};
use serde_json::{Value, json};
use sqlx::{SqliteConnection, SqlitePool};
use std::sync::Arc;

pub struct PublicationQueue {
    repository: Arc<PortalRepository>,
    audit: Arc<AuditService>,
}
impl PublicationQueue {
    pub fn new(repository: Arc<PortalRepository>, audit: Arc<AuditService>) -> Self {
        Self { repository, audit }
    }
    pub async fn delivered(
        &self,
        db: &mut SqliteConnection,
        id: i64,
        delivered: bool,
    ) -> Result<(), AppError> {
        self.repository.ensure(db, id).await?;
        self.repository.delivery(db, id, delivered).await
    }
    pub async fn ensure_code(
        &self,
        db: &mut SqliteConnection,
        id: i64,
    ) -> Result<String, AppError> {
        self.repository.ensure(db, id).await?;
        if let Some(code) = self.repository.code(db, id).await? {
            return Ok(code);
        }
        const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";
        for _ in 0..10 {
            let mut code = String::new();
            let mut count = 0;
            while count < 12 {
                let mut b = [0];
                OsRng.try_fill_bytes(&mut b).map_err(|_| AppError::Crypto)?;
                if usize::from(b[0]) >= 256 / ALPHABET.len() * ALPHABET.len() {
                    continue;
                }
                if count == 4 || count == 8 {
                    code.push('-');
                }
                code.push(ALPHABET[usize::from(b[0]) % ALPHABET.len()] as char);
                count += 1;
            }
            if self.repository.add_code(db, id, &code).await? {
                return Ok(code);
            }
        }
        Err(AppError::Conflict(
            "No se pudo reservar un código público único.".into(),
        ))
    }
    pub async fn withdraw(
        &self,
        db: &mut SqliteConnection,
        id: i64,
        actor: i64,
    ) -> Result<(), AppError> {
        self.repository.ensure(db, id).await?;
        self.repository.enabled(db, id, false).await?;
        self.capture(db, id, actor, true).await
    }
    pub async fn capture(
        &self,
        db: &mut SqliteConnection,
        id: i64,
        actor: i64,
        force: bool,
    ) -> Result<(), AppError> {
        let code = self.ensure_code(db, id).await?;
        let (enabled, mode, revision, delivery) = self.repository.policy(db, id).await?;
        if !force && (!enabled || mode == "manual") {
            if mode == "manual" {
                self.repository.issue(db, id, None).await?;
            }
            return Ok(());
        }
        let order = OrderRepository
            .load_any(db, id, true)
            .await?
            .ok_or(AppError::NotFound)?;
        let prepared = if !enabled {
            delivery
                .as_deref()
                .map(PublicTimestamp::parse)
                .transpose()
                .and_then(|at| PublicProjection::withdraw(code, revision + 1, at))
        } else {
            let (repair, _) = RepairRepository.load(db, id).await?;
            let timezone = self.repository.timezone(db).await?;
            let dates = self.repository.stage_dates(db, id).await?;
            (|| {
                if order.delivered && delivery.is_none() {
                    return Err(ProjectionError(
                        "Esta entrega conserva solo la fecha. Se requiere confirmar su instante para publicar.",
                    ));
                }
                let stages = repair
                    .stages
                    .iter()
                    .enumerate()
                    .map(|(i, name)| {
                        PublicStage::new(
                            name.clone(),
                            i < repair.completed,
                            dates
                                .get(i)
                                .and_then(|v| v.as_deref())
                                .map(PublicTimestamp::parse)
                                .transpose()?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let visible = |label: &str, default: bool| {
                    repair
                        .visibility
                        .iter()
                        .find(|v| v.label == label)
                        .map(|v| v.enabled)
                        .unwrap_or(default)
                };
                let mut events = repair.events.clone();
                if visible("Línea de tiempo", true) {
                    for e in events.iter_mut().filter(|e| e.public) {
                        e.date = PublicTimestamp::from_workshop_time(&e.date, &timezone)?
                            .as_str()
                            .to_owned();
                    }
                }
                let payment = if visible("Resumen de pago", true) {
                    Some(PublicPayment::new(
                        order.currency,
                        order.price_minor,
                        order.net_minor,
                        order.pending_minor,
                    )?)
                } else {
                    None
                };
                PublicProjection::publish(
                    code,
                    revision + 1,
                    ProjectionInput {
                        equipment: PublicEquipment::new(
                            order.equipment.item,
                            order.equipment.brand,
                            order.equipment.model,
                        )?,
                        technical_status: order.status,
                        stages,
                        events,
                        public_note: repair.public_note.clone(),
                        warranties: repair.warranties.clone(),
                        payment,
                        sections: PublicSections {
                            events: visible("Línea de tiempo", true),
                            note: visible("Notas del técnico", true),
                            warranties: visible("Garantías", true),
                            payment: visible("Resumen de pago", true),
                        },
                        received_at: Some(PublicTimestamp::parse(&calendar_timestamp(
                            &order.received_at,
                        )?)?),
                        estimated_delivery_at: order
                            .estimated_delivery_at
                            .as_deref()
                            .filter(|v| !v.is_empty())
                            .map(|s| calendar_timestamp(s).and_then(|v| PublicTimestamp::parse(&v)))
                            .transpose()?,
                        delivered_at: delivery
                            .as_deref()
                            .map(PublicTimestamp::parse)
                            .transpose()?,
                    },
                )
            })()
        };
        self.repository.issue(db, id, None).await?;
        let mut error = None;
        match prepared.and_then(PublicProjection::sign) {
            Ok(signed) => {
                let marks = self.repository.watermarks(db).await?;
                let document = signed
                    .document()
                    .map_err(|e| AppError::Validation(e.to_string()))?;
                self.repository.save(db, id, &document, marks).await?;
            }
            Err(e) => {
                error = Some(e.to_string());
                self.repository.issue(db, id, error.as_deref()).await?;
            }
        }
        self.audit
            .record(
                db,
                AuditRecord {
                    author_id: actor,
                    operation: "publication.edit",
                    entity: "portal_publicaciones",
                    entity_id: None,
                    order_id: Some(id),
                    reason: error.as_deref(),
                },
                &[(
                    "publicationRevision",
                    None,
                    Some((revision + 1).to_string()),
                )],
            )
            .await?;
        Ok(())
    }
}

fn calendar_timestamp(value: &str) -> Result<String, ProjectionError> {
    if value.len() == 10 {
        chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|_| ProjectionError("Fecha de calendario inválida."))?;
        Ok(format!("{value}T00:00:00.000Z"))
    } else {
        PublicTimestamp::parse(value)?;
        Ok(value.into())
    }
}

pub struct PortalService {
    pub(crate) runtime: super::portal_transport::PublisherRuntime,
    pub(crate) pool: SqlitePool,
    pub(crate) repository: Arc<PortalRepository>,
    pub(crate) queue: Arc<PublicationQueue>,
    pub(crate) access: Arc<AccessService>,
    pub(crate) audit: Arc<AuditService>,
}
impl PortalService {
    pub fn new(
        pool: SqlitePool,
        repository: Arc<PortalRepository>,
        queue: Arc<PublicationQueue>,
        access: Arc<AccessService>,
        audit: Arc<AuditService>,
        root: Option<std::path::PathBuf>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            runtime: super::portal_transport::PublisherRuntime::new(root)?,
            pool,
            repository,
            queue,
            access,
            audit,
        })
    }
    pub async fn status(&self, id: i64) -> Result<Value, AppError> {
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        if let Err(error) = self
            .access
            .require(
                &mut tx,
                "clients.view",
                AccessScope::Clients,
                Some(id),
                false,
            )
            .await
        {
            if !matches!(error, AppError::Forbidden) {
                return Err(error);
            }
            self.access
                .require(
                    &mut tx,
                    "repairs.view",
                    AccessScope::Repairs,
                    Some(id),
                    false,
                )
                .await?;
        }
        OrderRepository
            .load(&mut tx, id)
            .await?
            .ok_or(AppError::NotFound)?;
        let mut result = self.repository.status(&mut tx, id).await?;
        let config = self.repository.config(&mut tx).await?;
        result["configured"] = json!(!config.project_id.is_empty());
        result["paused"] = json!(config.paused);
        if let Some(code) = result["code"].as_str() {
            if !config.portal_url.is_empty() {
                result["url"] = json!(format!(
                    "{}/seguimiento/{code}",
                    config.portal_url.trim_end_matches('/')
                ));
            }
        }
        tx.commit().await?;
        Ok(result)
    }
    pub async fn action(&self, id: i64, action: &str) -> Result<Value, AppError> {
        if !["publish", "withdraw", "rotate"].contains(&action) {
            return Err(AppError::Validation(
                "Acción de publicación inválida.".into(),
            ));
        }
        {
            let _gate = self.access.lock().await;
            let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
            let user = self
                .access
                .require(
                    &mut tx,
                    "publication.edit",
                    AccessScope::Repairs,
                    Some(id),
                    true,
                )
                .await?;
            OrderRepository
                .load(&mut tx, id)
                .await?
                .ok_or(AppError::NotFound)?;
            self.queue.ensure_code(&mut tx, id).await?;
            if action == "rotate" {
                self.queue.withdraw(&mut tx, id, user.id).await?;
                self.repository.retire_code(&mut tx, id).await?;
            }
            self.repository
                .enabled(&mut tx, id, action != "withdraw")
                .await?;
            self.queue.capture(&mut tx, id, user.id, true).await?;
            tx.commit().await?;
        }
        self.status(id).await
    }
}
