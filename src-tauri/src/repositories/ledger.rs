use crate::{domain::ledger_contracts::*, error::AppError};
use sqlx::{Row, SqliteConnection};
pub struct LedgerRepository;

pub trait LedgerReader: Send + Sync {
    fn active<'a>(
        &'a self,
        db: &'a mut SqliteConnection,
        id: i64,
    ) -> super::RepositoryFuture<'a, ()>;

    fn state<'a>(
        &'a self,
        db: &'a mut SqliteConnection,
        id: i64,
    ) -> super::RepositoryFuture<'a, PaymentState>;
}

impl LedgerReader for LedgerRepository {
    fn active<'a>(
        &'a self,
        db: &'a mut SqliteConnection,
        id: i64,
    ) -> super::RepositoryFuture<'a, ()> {
        Box::pin(LedgerRepository::active(self, db, id))
    }

    fn state<'a>(
        &'a self,
        db: &'a mut SqliteConnection,
        id: i64,
    ) -> super::RepositoryFuture<'a, PaymentState> {
        Box::pin(LedgerRepository::state(self, db, id))
    }
}

impl LedgerRepository {
    pub async fn active(&self, db: &mut SqliteConnection, id: i64) -> Result<(), AppError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM ordenes WHERE id=? AND eliminada_en IS NULL)",
        )
        .bind(id)
        .fetch_one(db)
        .await?;
        if exists {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }
    pub async fn state(
        &self,
        db: &mut SqliteConnection,
        id: i64,
    ) -> Result<PaymentState, AppError> {
        let r = sqlx::query("SELECT * FROM vista_saldos_orden WHERE id_orden=?")
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(PaymentState {
            total_minor: r.try_get("precio_centavos")?,
            collected_minor: r.try_get("cobro_neto_centavos")?,
            received_minor: r.try_get("cobrado_centavos")?,
            adjusted_minor: r.try_get("ajustado_centavos")?,
            refunded_minor: r.try_get("devuelto_centavos")?,
            credit_minor: r.try_get("saldo_favor_centavos")?,
            paid: r.get::<i64, _>("pendiente_centavos") == 0,
        })
    }
    pub async fn movements(
        &self,
        db: &mut SqliteConnection,
        id: i64,
    ) -> Result<Vec<MoneyMovement>, AppError> {
        Ok(sqlx::query("SELECT cast(a.id AS TEXT) id,'receipt' tipo,NULL abono,a.importe_centavos importe,a.importe_centavos+coalesce((SELECT sum(j.importe_centavos) FROM ajustes_abono j WHERE j.id_abono=a.id),0) efectivo,a.metodo_pago metodo,a.motivo motivo,a.fecha_pago fecha FROM abonos a WHERE a.id_orden=?1 UNION ALL SELECT 'adjustment-'||j.id,'adjustment',cast(j.id_abono AS TEXT),j.importe_centavos,NULL,NULL,j.motivo,j.creado_en FROM ajustes_abono j JOIN abonos a ON a.id=j.id_abono WHERE a.id_orden=?1 UNION ALL SELECT 'refund-'||id,'refund',NULL,importe_centavos,NULL,metodo_pago,motivo,creado_en FROM devoluciones WHERE id_orden=?1 ORDER BY fecha DESC,id DESC").bind(id).fetch_all(db).await?.iter().map(|r|MoneyMovement{id:r.get("id"),kind:r.get("tipo"),payment_id:r.get("abono"),amount_minor:r.get("importe"),effective_minor:r.get("efectivo"),method:r.get("metodo"),reason:r.get("motivo"),occurred_at:r.get("fecha")}).collect())
    }
    pub async fn request(
        &self,
        db: &mut SqliteConnection,
        id: &str,
    ) -> Result<Option<(i64, Vec<u8>)>, AppError> {
        Ok(
            sqlx::query("SELECT id_autor,huella_solicitud FROM solicitudes_financieras WHERE id=?")
                .bind(id)
                .fetch_optional(db)
                .await?
                .map(|r| (r.get("id_autor"), r.get("huella_solicitud"))),
        )
    }
    pub async fn insert(
        &self,
        db: &mut SqliteConnection,
        author: i64,
        r: &MoneyRequest,
        hash: &[u8],
    ) -> Result<i64, AppError> {
        sqlx::query("INSERT INTO solicitudes_financieras(id,id_autor,tipo,huella_solicitud) VALUES (?,?,?,?)").bind(&r.request_id).bind(author).bind(&r.kind).bind(hash).execute(&mut *db).await?;
        let result=match r.kind.as_str(){
            "receipt"=>sqlx::query("INSERT INTO abonos(id_orden,id_autor,importe_centavos,metodo_pago,motivo,id_solicitud,fecha_pago) VALUES (?,?,?,?,?,?,strftime('%Y-%m-%dT%H:%M:%fZ','now'))").bind(r.order_id).bind(author).bind(r.amount_minor).bind(r.method.trim()).bind(&r.reason).bind(&r.request_id).execute(db).await?,
            "adjustment"=>sqlx::query("INSERT INTO ajustes_abono(id_abono,id_autor,importe_centavos,motivo,id_solicitud) VALUES (?,?,?,?,?)").bind(r.payment_id).bind(author).bind(r.amount_minor).bind(&r.reason).bind(&r.request_id).execute(db).await?,
            _=>sqlx::query("INSERT INTO devoluciones(id_orden,id_autor,importe_centavos,metodo_pago,motivo,id_solicitud) VALUES (?,?,?,?,?,?)").bind(r.order_id).bind(author).bind(r.amount_minor).bind(r.method.trim()).bind(&r.reason).bind(&r.request_id).execute(db).await?,
        };
        Ok(result.last_insert_rowid())
    }
}
