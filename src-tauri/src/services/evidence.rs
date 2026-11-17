use super::{access::AccessService, audit::AuditService};
use crate::{
    domain::{contracts::AccessScope, evidence_contracts::*},
    error::AppError,
    repositories::{
        audit::AuditRecord, evidence::EvidenceRepository, ledger::LedgerReader, orders::OrderReader,
    },
};
use sqlx::SqlitePool;
use std::{fs, io::Write, path::PathBuf, sync::Arc};
pub struct EvidenceService {
    pool: SqlitePool,
    repository: Arc<EvidenceRepository>,
    orders: Arc<dyn OrderReader>,
    ledger: Arc<dyn LedgerReader>,
    access: Arc<AccessService>,
    audit: Arc<AuditService>,
    root: Option<PathBuf>,
}
fn invalid(text: &str) -> AppError {
    AppError::Validation(text.into())
}
impl EvidenceService {
    pub fn new(
        pool: SqlitePool,
        repository: Arc<EvidenceRepository>,
        orders: Arc<dyn OrderReader>,
        ledger: Arc<dyn LedgerReader>,
        access: Arc<AccessService>,
        audit: Arc<AuditService>,
        root: Option<PathBuf>,
    ) -> Self {
        Self {
            pool,
            repository,
            orders,
            ledger,
            access,
            audit,
            root,
        }
    }
    fn relative(&self, order: i64, id: &str, mime: &str) -> Result<String, AppError> {
        valid_id(id).map_err(AppError::Validation)?;
        if order <= 0 {
            return Err(AppError::NotFound);
        }
        let (_, ext) =
            evidence_format(mime).ok_or_else(|| invalid("Formato de evidencia no permitido."))?;
        Ok(format!("evidencias/{order}/{id}.{ext}"))
    }
    fn path(&self, order: i64, id: &str, mime: &str) -> Result<PathBuf, AppError> {
        Ok(self
            .root
            .as_ref()
            .ok_or_else(|| invalid("No está disponible el almacenamiento de evidencias."))?
            .join(self.relative(order, id, mime)?))
    }
    pub async fn list(&self, id: i64, deleted: bool) -> Result<Vec<Evidence>, AppError> {
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(
                &mut tx,
                "evidence.view",
                AccessScope::Clients,
                Some(id),
                false,
            )
            .await?;
        if deleted {
            self.access
                .require(
                    &mut tx,
                    "evidence.delete",
                    AccessScope::Clients,
                    Some(id),
                    false,
                )
                .await?;
        }
        self.orders
            .load_any(&mut tx, id, true)
            .await?
            .ok_or(AppError::NotFound)?;
        let result = self.repository.list(&mut tx, id, deleted).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn read(&self, order: i64, id: &str) -> Result<Vec<u8>, AppError> {
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(
                &mut tx,
                "evidence.view",
                AccessScope::Clients,
                Some(order),
                false,
            )
            .await?;
        let stored = self
            .repository
            .get(&mut tx, id)
            .await?
            .ok_or(AppError::NotFound)?;
        let photo = stored.value;
        if photo.order_id != order.to_string() {
            return Err(AppError::NotFound);
        }
        if photo.deleted {
            self.access
                .require(
                    &mut tx,
                    "evidence.delete",
                    AccessScope::Clients,
                    Some(order),
                    false,
                )
                .await?;
        }
        let path = self.path(order, id, &photo.mime)?;
        if fs::metadata(&path)
            .map_err(|_| invalid("No se encontró la foto. Revisa el respaldo de evidencias."))?
            .len()
            != photo.size_bytes as u64
        {
            return Err(invalid("El archivo de la foto cambió fuera de HRL."));
        }
        let bytes = fs::read(path).map_err(|_| invalid("No se pudo leer la foto."))?;
        tx.commit().await?;
        Ok(bytes)
    }
    pub async fn add(&self, p: NewEvidence) -> Result<Evidence, AppError> {
        validate(&p).map_err(AppError::Validation)?;
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        let author = self
            .access
            .require(
                &mut tx,
                "evidence.add",
                AccessScope::Clients,
                Some(p.order_id),
                false,
            )
            .await?;
        self.ledger.active(&mut tx, p.order_id).await?;
        let path = self.path(p.order_id, &p.id, &p.mime)?;
        if let Some(old) = self.repository.get(&mut tx, &p.id).await? {
            if old.value.order_id != p.order_id.to_string()
                || old.author != author.id
                || old.value.original_name != p.original_name
                || old.value.mime != p.mime
                || old.value.source != p.source
                || old.initial_note != p.note
                || fs::read(&path)
                    .map_err(|_| invalid("No se pudo leer el archivo ya registrado."))?
                    != p.bytes
            {
                return Err(AppError::Conflict(
                    "El identificador corresponde a otra evidencia.".into(),
                ));
            }
            tx.commit().await?;
            return Ok(old.value);
        }
        self.access
            .require(
                &mut tx,
                "evidence.add",
                AccessScope::Clients,
                Some(p.order_id),
                true,
            )
            .await?;
        fs::create_dir_all(path.parent().ok_or_else(|| invalid("Carpeta inválida."))?)
            .map_err(|_| invalid("No se pudo crear la carpeta de evidencias."))?;
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                if file
                    .write_all(&p.bytes)
                    .and_then(|_| file.sync_all())
                    .is_err()
                {
                    drop(file);
                    let _ = fs::remove_file(&path);
                    return Err(invalid("No se pudo escribir la evidencia completa."));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if fs::read(&path)
                    .map_err(|_| invalid("No se pudo leer la evidencia pendiente."))?
                    != p.bytes
                {
                    return Err(AppError::Conflict(
                        "Ya existe otro archivo para esta solicitud.".into(),
                    ));
                }
            }
            Err(_) => return Err(invalid("No se pudo crear el archivo de evidencia.")),
        }
        let result = async {
            let id = self
                .repository
                .insert(
                    &mut tx,
                    &p,
                    author.id,
                    &self.relative(p.order_id, &p.id, &p.mime)?,
                )
                .await?;
            self.audit
                .record(
                    &mut tx,
                    AuditRecord {
                        author_id: author.id,
                        operation: "evidence.add",
                        entity: "evidencias",
                        entity_id: Some(id),
                        order_id: Some(p.order_id),
                        reason: None,
                    },
                    &[
                        ("evidenceId", None, Some(p.id.clone())),
                        ("evidenceName", None, Some(p.original_name.clone())),
                        ("evidenceSource", None, Some(p.source.clone())),
                        ("evidenceNote", None, Some(p.note.clone())),
                    ],
                )
                .await?;
            Ok::<_, AppError>(
                self.repository
                    .get(&mut tx, &p.id)
                    .await?
                    .ok_or(AppError::NotFound)?
                    .value,
            )
        }
        .await;
        let saved = match result {
            Ok(value) => value,
            Err(e) => {
                let _ = tx.rollback().await;
                let _ = fs::remove_file(path);
                return Err(e);
            }
        };
        tx.commit().await.map_err(AppError::Commit)?;
        Ok(saved)
    }
    pub async fn update(&self, p: UpdateEvidence) -> Result<Evidence, AppError> {
        valid_id(&p.id).map_err(AppError::Validation)?;
        if p.note.chars().count() > 500 {
            return Err(invalid("La descripción admite hasta 500 caracteres."));
        }
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(
                &mut tx,
                "evidence.view",
                AccessScope::Clients,
                Some(p.order_id),
                false,
            )
            .await?;
        self.ledger.active(&mut tx, p.order_id).await?;
        let stored = self
            .repository
            .get(&mut tx, &p.id)
            .await?
            .ok_or(AppError::NotFound)?;
        let old = stored.value;
        if old.order_id != p.order_id.to_string() {
            return Err(AppError::NotFound);
        }
        if old.revision != p.expected_revision {
            return Err(AppError::Conflict(
                "La foto cambió. Actualiza la galería.".into(),
            ));
        }
        let op = if old.deleted != p.deleted {
            "evidence.delete"
        } else {
            "evidence.edit"
        };
        let user = self
            .access
            .require(&mut tx, op, AccessScope::Clients, Some(p.order_id), true)
            .await?;
        if old.note != p.note && op != "evidence.edit" {
            self.access
                .require(
                    &mut tx,
                    "evidence.edit",
                    AccessScope::Clients,
                    Some(p.order_id),
                    true,
                )
                .await?;
        }
        if old.note == p.note && old.deleted == p.deleted {
            tx.commit().await?;
            return Ok(old);
        }
        self.repository.update(&mut tx, &p).await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: user.id,
                    operation: if op == "evidence.delete" && !p.deleted {
                        "evidence.restore"
                    } else {
                        op
                    },
                    entity: "evidencias",
                    entity_id: Some(stored.entity_id),
                    order_id: Some(p.order_id),
                    reason: None,
                },
                &[
                    ("evidenceId", Some(p.id.clone()), Some(p.id.clone())),
                    ("evidenceNote", Some(old.note), Some(p.note)),
                    (
                        "evidenceRemoved",
                        Some(old.deleted.to_string()),
                        Some(p.deleted.to_string()),
                    ),
                ],
            )
            .await?;
        let result = self
            .repository
            .get(&mut tx, &p.id)
            .await?
            .ok_or(AppError::NotFound)?
            .value;
        tx.commit().await.map_err(AppError::Commit)?;
        Ok(result)
    }
}
