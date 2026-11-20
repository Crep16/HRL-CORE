use super::{access::AccessService, audit::AuditService};
use crate::{
    domain::{contracts::AccessScope, repair_contracts::*},
    error::AppError,
    repositories::{audit::AuditRecord, ledger::LedgerReader, repairs::RepairRepository},
};
use sqlx::SqlitePool;
use std::sync::Arc;
pub struct RepairService {
    pool: SqlitePool,
    repository: Arc<RepairRepository>,
    ledger: Arc<dyn LedgerReader>,
    access: Arc<AccessService>,
    audit: Arc<AuditService>,
    publication: Arc<super::portal::PublicationQueue>,
}
impl RepairService {
    pub fn new(
        pool: SqlitePool,
        repository: Arc<RepairRepository>,
        ledger: Arc<dyn LedgerReader>,
        access: Arc<AccessService>,
        audit: Arc<AuditService>,
        publication: Arc<super::portal::PublicationQueue>,
    ) -> Self {
        Self {
            pool,
            repository,
            ledger,
            access,
            audit,
            publication,
        }
    }
    pub async fn load(&self, id: i64) -> Result<Snapshot, AppError> {
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(
                &mut tx,
                "repairs.view",
                AccessScope::Repairs,
                Some(id),
                false,
            )
            .await?;
        self.ledger.active(&mut tx, id).await?;
        let (repair, revision) = self.repository.load(&mut tx, id).await?;
        let payment = self.ledger.state(&mut tx, id).await?;
        tx.commit().await?;
        Ok(Snapshot {
            repair,
            revision,
            payment,
        })
    }
    pub async fn stages(&self, id: i64) -> Result<Vec<String>, AppError> {
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(
                &mut tx,
                "repairs.view",
                AccessScope::Repairs,
                Some(id),
                false,
            )
            .await?;
        self.ledger.active(&mut tx, id).await?;
        let result = self.repository.stages(&mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn hide_stage_name(&self, id: i64, name: String) -> Result<Vec<String>, AppError> {
        let name = name.trim();
        if name.is_empty() || name.len() > 100 {
            return Err(AppError::Validation("Nombre de etapa inválido.".into()));
        }
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(
                &mut tx,
                "repairs.view",
                AccessScope::Repairs,
                Some(id),
                false,
            )
            .await?;
        let user = self
            .access
            .require(&mut tx, "repairs.edit", AccessScope::Repairs, None, true)
            .await?;
        self.ledger.active(&mut tx, id).await?;
        if !self.repository.hide_stage_name(&mut tx, name).await? {
            return Err(AppError::Validation("Solo puedes quitar nombres personalizados guardados; las opciones predeterminadas se conservan.".into()));
        }
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: user.id,
                    operation: "repairs.edit",
                    entity: "nombres_etapas_ocultos",
                    entity_id: None,
                    order_id: Some(id),
                    reason: None,
                },
                &[("stageSuggestion", Some(name.into()), None)],
            )
            .await?;
        let names = self.repository.stages(&mut tx).await?;
        tx.commit().await?;
        Ok(names)
    }
    pub async fn summary(&self, id: i64) -> Result<PublicSummary, AppError> {
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        match self
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
            Ok(_) => (),
            Err(AppError::Forbidden) => {
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
            Err(e) => return Err(e),
        }
        self.ledger.active(&mut tx, id).await?;
        let (repair, _) = self.repository.load(&mut tx, id).await?;
        tx.commit().await?;
        Ok(PublicSummary {
            technician: repair.technician,
            public_note: repair.public_note,
            warranties: repair.warranties,
        })
    }
    pub async fn save(&self, payload: SaveRepairPayload) -> Result<Snapshot, AppError> {
        validate(&payload.repair).map_err(AppError::Validation)?;
        let next = &payload.repair;
        if next.events.len() > 10000
            || next.parts.len() > 10000
            || next.warranties.len() > 10000
            || next.visibility.len() > 20
        {
            return Err(AppError::Validation(
                "Revisa las etapas y registros del seguimiento.".into(),
            ));
        }
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        let id = payload.order_id;
        let user = self
            .access
            .require(
                &mut tx,
                "repairs.view",
                AccessScope::Repairs,
                Some(id),
                false,
            )
            .await?;
        self.ledger.active(&mut tx, id).await?;
        let (old, revision) = self.repository.load(&mut tx, id).await?;
        if payload.revision != revision {
            return Err(AppError::Conflict(
                "El seguimiento cambió. Conserva el borrador y vuelve a consultarlo.".into(),
            ));
        }
        let operations = operations(&old, next).map_err(AppError::Validation)?;
        let changed = !operations.is_empty();
        if next.events[..next.events.len() - old.events.len()]
            .iter()
            .any(|e| !next.stages.contains(&e.stage))
        {
            return Err(AppError::Validation(
                "Los avances nuevos deben pertenecer a una etapa activa.".into(),
            ));
        }
        for operation in &operations {
            self.access
                .require(&mut tx, operation, AccessScope::Repairs, Some(id), true)
                .await?;
        }
        for name in old.stages.iter().filter(|name| !next.stages.contains(name)) {
            if self
                .repository
                .stage_was_completed(&mut tx, id, name)
                .await?
            {
                return Err(AppError::Validation(
                    "No puedes eliminar una etapa que ya fue completada.".into(),
                ));
            }
        }
        if !operations.is_empty() {
            self.repository
                .save(&mut tx, id, user.id, &old, next, revision)
                .await?;
            let mut changes = vec![];
            for (field, a, b) in [
                ("technician", &old.technician, &next.technician),
                ("publicNote", &old.public_note, &next.public_note),
                ("internalNote", &old.internal_note, &next.internal_note),
                ("syncMode", &old.sync_mode, &next.sync_mode),
            ] {
                if a != b {
                    changes.push((field, Some(a.clone()), Some(b.clone())));
                }
            }
            if old.completed != next.completed {
                changes.push((
                    "completed",
                    Some(old.completed.to_string()),
                    Some(next.completed.to_string()),
                ));
            }
            changes.push((
                "revision",
                Some(revision.to_string()),
                Some((revision + 1).to_string()),
            ));
            for operation in operations {
                self.audit
                    .record(
                        &mut tx,
                        AuditRecord {
                            author_id: user.id,
                            operation,
                            entity: "seguimientos",
                            entity_id: Some(id),
                            order_id: Some(id),
                            reason: None,
                        },
                        &changes,
                    )
                    .await?;
            }
            let mut details: Vec<(String, Option<String>, Option<String>)> = vec![];
            if old.stages != next.stages {
                details.push((
                    "stages".into(),
                    Some(old.stages.join(" → ")),
                    Some(next.stages.join(" → ")),
                ));
            }
            for (index, event) in next.events[..next.events.len() - old.events.len()]
                .iter()
                .enumerate()
            {
                for (field, value) in [
                    ("title", event.title.clone()),
                    ("detail", event.detail.clone()),
                    ("date", event.date.clone()),
                    ("stage", event.stage.clone()),
                    ("public", event.public.to_string()),
                ] {
                    details.push((format!("events.{}.{}", index + 1, field), None, Some(value)));
                }
            }
            for (index, warranty) in next.warranties[old.warranties.len()..].iter().enumerate() {
                details.push((
                    format!("warranties.{}.name", index + 1),
                    None,
                    Some(warranty.name.clone()),
                ));
                details.push((
                    format!("warranties.{}.days", index + 1),
                    None,
                    Some(warranty.days.to_string()),
                ));
            }
            for (index, part) in next.parts[old.parts.len()..].iter().enumerate() {
                details.push((
                    format!("parts.{}.name", index + 1),
                    None,
                    Some(part.name.clone()),
                ));
                details.push((
                    format!("parts.{}.priceMinor", index + 1),
                    None,
                    Some(part.price.to_string()),
                ));
            }
            for visibility in &next.visibility {
                let previous = old
                    .visibility
                    .iter()
                    .find(|v| v.label == visibility.label)
                    .map(|v| v.enabled);
                if previous != Some(visibility.enabled) {
                    details.push((
                        format!("visibility.{}", visibility.label),
                        previous.map(|v| v.to_string()),
                        Some(visibility.enabled.to_string()),
                    ));
                }
            }
            if !details.is_empty() {
                let borrowed = details
                    .iter()
                    .map(|(field, old, new)| (field.as_str(), old.clone(), new.clone()))
                    .collect::<Vec<_>>();
                self.audit
                    .record(
                        &mut tx,
                        AuditRecord {
                            author_id: user.id,
                            operation: "repairs.save",
                            entity: "seguimientos",
                            entity_id: Some(id),
                            order_id: Some(id),
                            reason: None,
                        },
                        &borrowed,
                    )
                    .await?;
            }
        }
        if changed {
            self.publication
                .capture(&mut tx, id, user.id, false)
                .await?;
        }
        let (repair, revision) = self.repository.load(&mut tx, id).await?;
        let payment = self.ledger.state(&mut tx, id).await?;
        tx.commit().await?;
        Ok(Snapshot {
            repair,
            revision,
            payment,
        })
    }
}
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSummary {
    pub technician: String,
    pub public_note: String,
    pub warranties: Vec<Warranty>,
}
