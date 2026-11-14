use super::{
    access::{hash, login_name, validate_password, AccessService},
    audit::AuditService,
};
use crate::{
    domain::{contracts::AccessScope, user_contracts::*},
    error::AppError,
    repositories::{audit::AuditRecord, users::UserRepository},
};
use sqlx::SqlitePool;
use std::sync::Arc;
pub struct UserService {
    pool: SqlitePool,
    repository: Arc<UserRepository>,
    access: Arc<AccessService>,
    audit: Arc<AuditService>,
}
impl UserService {
    pub fn new(
        pool: SqlitePool,
        repository: Arc<UserRepository>,
        access: Arc<AccessService>,
        audit: Arc<AuditService>,
    ) -> Self {
        Self {
            pool,
            repository,
            access,
            audit,
        }
    }
    pub async fn list(&self) -> Result<Vec<Account>, AppError> {
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(&mut tx, "users.manage", AccessScope::Users, None, false)
            .await?;
        let accounts = self.repository.list(&mut tx).await?;
        tx.commit().await?;
        Ok(accounts)
    }
    pub async fn save(&self, input: SaveUserPayload) -> Result<(), AppError> {
        let login = login_name(&input.name)?;
        if !["complete", "limited", "custom"].contains(&input.access_level.as_str())
            || !["administrator", "technician"].contains(&input.role.as_str())
            || input.id.is_some_and(|id| id <= 0)
            || input.permissions.len() > 32
            || input.permissions.iter().any(|g| {
                if g.section == "payments" {
                    !["add", "refund"].contains(&g.action.as_str())
                } else {
                    !["clients", "repairs", "company", "logbook"].contains(&g.section.as_str())
                        || !["view", "create", "edit", "delete"].contains(&g.action.as_str())
                }
            })
        {
            return Err(AppError::Validation(
                "Revisa el nivel, rol y permisos de la cuenta.".into(),
            ));
        }
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        let author = self
            .access
            .require(&mut tx, "users.manage", AccessScope::Users, None, true)
            .await?;
        self.access
            .verify_master(&mut tx, &input.master_password)
            .await?;
        let previous = self.repository.list(&mut tx).await?;
        let old = previous.iter().find(|u| Some(u.id) == input.id);
        if input.id.is_some() && old.is_none() {
            return Err(AppError::NotFound);
        }
        if previous
            .iter()
            .any(|u| Some(u.id) != input.id && u.name.trim().to_lowercase() == login)
        {
            return Err(AppError::Validation(
                "Ya existe una cuenta con ese usuario.".into(),
            ));
        }
        if old
            .is_some_and(|u| u.active && u.access_level == "complete" && u.role == "administrator")
            && !(input.active && input.access_level == "complete" && input.role == "administrator")
            && !previous.iter().any(|u| {
                Some(u.id) != input.id
                    && u.active
                    && u.access_level == "complete"
                    && u.role == "administrator"
            })
        {
            return Err(AppError::Validation(
                "Debe permanecer un administrador activo con acceso completo.".into(),
            ));
        }
        let hash = if input.id.is_none() || !input.password.is_empty() {
            validate_password(&input.password, &input.confirmation)?;
            Some(hash(&input.password).await?)
        } else {
            if !input.confirmation.is_empty() {
                return Err(AppError::Validation(
                    "Deja ambas claves vacías para conservar la actual.".into(),
                ));
            }
            None
        };
        let id = self
            .repository
            .save(&mut tx, &input, &login, hash.as_deref())
            .await?;
        let mut changes = vec![
            (
                "name",
                old.map(|u| u.name.clone()),
                Some(input.name.trim().into()),
            ),
            (
                "accessLevel",
                old.map(|u| u.access_level.clone()),
                Some(input.access_level.clone()),
            ),
            (
                "role",
                old.map(|u| u.role.clone()),
                Some(input.role.clone()),
            ),
            (
                "active",
                old.map(|u| u.active.to_string()),
                Some(input.active.to_string()),
            ),
        ];
        if hash.is_some() {
            changes.push(("credentialsUpdated", None, Some("true".into())))
        }
        let permission_text = |grants: &[Grant]| {
            let mut values = grants
                .iter()
                .map(|g| format!("{}.{}", g.section, g.action))
                .collect::<Vec<_>>();
            values.sort();
            values.dedup();
            values.join(", ")
        };
        let new = self
            .repository
            .list(&mut tx)
            .await?
            .into_iter()
            .find(|u| u.id == id)
            .ok_or(AppError::NotFound)?;
        changes.push((
            "permissions",
            old.map(|u| permission_text(&u.permissions)),
            Some(permission_text(&new.permissions)),
        ));
        changes.retain(|(_, old, new)| old != new);
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: author.id,
                    operation: if input.id.is_none() {
                        "users.create"
                    } else {
                        "users.edit"
                    },
                    entity: "usuarios",
                    entity_id: Some(id),
                    order_id: None,
                    reason: None,
                },
                &changes,
            )
            .await?;
        tx.commit().await.map_err(AppError::Commit)?;
        Ok(())
    }
}
