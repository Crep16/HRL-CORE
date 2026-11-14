use crate::error::AppError;
use sqlx::SqliteConnection;

#[derive(Default)]
pub struct AuditRepository;

pub struct AuditRecord<'a> {
    pub author_id: i64,
    pub operation: &'a str,
    pub entity: &'a str,
    pub entity_id: Option<i64>,
    pub order_id: Option<i64>,
    pub reason: Option<&'a str>,
}

pub trait AuditWriter: Send + Sync {
    fn insert<'a>(
        &'a self,
        db: &'a mut SqliteConnection,
        record: &'a AuditRecord<'_>,
    ) -> super::RepositoryFuture<'a, i64>;

    fn detail<'a>(
        &'a self,
        db: &'a mut SqliteConnection,
        audit_id: i64,
        field: &'a str,
        old: Option<&'a str>,
        new: Option<&'a str>,
    ) -> super::RepositoryFuture<'a, ()>;
}

impl AuditWriter for AuditRepository {
    fn insert<'a>(
        &'a self,
        db: &'a mut SqliteConnection,
        record: &'a AuditRecord<'_>,
    ) -> super::RepositoryFuture<'a, i64> {
        Box::pin(AuditRepository::insert(self, db, record))
    }

    fn detail<'a>(
        &'a self,
        db: &'a mut SqliteConnection,
        audit_id: i64,
        field: &'a str,
        old: Option<&'a str>,
        new: Option<&'a str>,
    ) -> super::RepositoryFuture<'a, ()> {
        Box::pin(AuditRepository::detail(self, db, audit_id, field, old, new))
    }
}

impl AuditRepository {
    pub async fn insert(
        &self,
        db: &mut SqliteConnection,
        record: &AuditRecord<'_>,
    ) -> Result<i64, AppError> {
        let operation: i64 =
            sqlx::query_scalar("SELECT id FROM operaciones_auditoria WHERE codigo=?")
                .bind(record.operation)
                .fetch_one(&mut *db)
                .await?;
        Ok(sqlx::query("INSERT INTO auditoria(id_autor,id_operacion,id_orden,entidad,id_entidad,motivo) VALUES (?,?,?,?,?,?)")
            .bind(record.author_id).bind(operation).bind(record.order_id).bind(record.entity)
            .bind(record.entity_id.map(|id|id.to_string())).bind(record.reason).execute(db).await?.last_insert_rowid())
    }

    pub async fn detail(
        &self,
        db: &mut SqliteConnection,
        audit_id: i64,
        field: &str,
        old: Option<&str>,
        new: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO auditoria_detalle(id_auditoria,campo,valor_anterior,valor_nuevo) VALUES (?,?,?,?)")
            .bind(audit_id).bind(field).bind(old).bind(new).execute(db).await?;
        Ok(())
    }
}
