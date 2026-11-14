use crate::{
    error::AppError,
    repositories::audit::{AuditRecord, AuditWriter},
};
use sqlx::SqliteConnection;
use std::sync::Arc;

pub struct AuditService {
    repository: Arc<dyn AuditWriter>,
}

impl AuditService {
    pub fn new(repository: Arc<dyn AuditWriter>) -> Self {
        Self { repository }
    }

    pub async fn record(
        &self,
        db: &mut SqliteConnection,
        record: AuditRecord<'_>,
        changes: &[(&str, Option<String>, Option<String>)],
    ) -> Result<(), AppError> {
        let id = self.repository.insert(db, &record).await?;
        for (field, old, new) in changes {
            self.repository
                .detail(db, id, field, old.as_deref(), new.as_deref())
                .await?;
        }
        Ok(())
    }
}
