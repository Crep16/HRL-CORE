use super::{access::AccessService, audit::AuditService};
use crate::{
    domain::{contracts::AccessScope, ledger_contracts::*},
    error::AppError,
    repositories::{audit::AuditRecord, ledger::LedgerRepository},
};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::sync::Arc;
pub struct LedgerService {
    pool: SqlitePool,
    repository: Arc<LedgerRepository>,
    access: Arc<AccessService>,
    audit: Arc<AuditService>,
}
#[derive(serde::Serialize)]
pub struct Ledger {
    pub movements: Vec<MoneyMovement>,
    pub payment: PaymentState,
}
impl LedgerService {
    pub fn new(
        pool: SqlitePool,
        repository: Arc<LedgerRepository>,
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
    pub async fn list(&self, id: i64) -> Result<Ledger, AppError> {
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
        self.repository.active(&mut tx, id).await?;
        let result = Ledger {
            movements: self.repository.movements(&mut tx, id).await?,
            payment: self.repository.state(&mut tx, id).await?,
        };
        tx.commit().await?;
        Ok(result)
    }
    pub async fn record(&self, r: MoneyRequest) -> Result<PaymentState, AppError> {
        let invalid = |text: &str| AppError::Validation(text.into());
        let (operation, entity) = match r.kind.as_str() {
            "receipt" => ("payments.add", "abonos"),
            "refund" => ("payments.refund", "devoluciones"),
            "adjustment" => ("payments.adjust", "ajustes_abono"),
            _ => return Err(invalid("Movimiento desconocido.")),
        };
        if !(16..=100).contains(&r.request_id.len())
            || r.order_id <= 0
            || r.amount_minor == 0
            || !(-9_000_000_000_000..=9_000_000_000_000).contains(&r.amount_minor)
            || r.method.len() > 200
            || r.reason.len() > 2000
            || (r.kind != "adjustment"
                && (r.amount_minor < 0 || r.method.trim().is_empty() || r.payment_id.is_some()))
            || (r.kind != "receipt" && r.reason.trim().is_empty())
        {
            return Err(invalid(
                "Revisa importe, método, motivo e identificador del movimiento.",
            ));
        }
        let hash = Sha256::digest(
            serde_json::to_vec(&r).map_err(|_| invalid("No se pudo preparar la solicitud."))?,
        )
        .to_vec();
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        let user = self
            .access
            .require(
                &mut tx,
                operation,
                AccessScope::Repairs,
                Some(r.order_id),
                false,
            )
            .await?;
        self.repository.active(&mut tx, r.order_id).await?;
        if let Some((author, previous)) = self.repository.request(&mut tx, &r.request_id).await? {
            if author != user.id || previous != hash {
                return Err(AppError::Conflict(
                    "El identificador ya corresponde a otra solicitud.".into(),
                ));
            }
            let result = self.repository.state(&mut tx, r.order_id).await?;
            tx.commit().await?;
            return Ok(result);
        }
        let state = self.repository.state(&mut tx, r.order_id).await?;
        if r.kind == "adjustment" {
            let payment = r
                .payment_id
                .ok_or_else(|| invalid("Selecciona el abono original."))?
                .to_string();
            let rows = self.repository.movements(&mut tx, r.order_id).await?;
            let original = rows
                .iter()
                .find(|row| row.kind == "receipt" && row.id == payment)
                .ok_or_else(|| invalid("El abono no pertenece a esta orden."))?;
            let effective = original.effective_minor.unwrap_or(original.amount_minor);
            if effective != r.expected_minor {
                return Err(AppError::Conflict(
                    "El abono cambió. Actualiza los movimientos.".into(),
                ));
            }
            if !(0..=9_000_000_000_000).contains(&(effective + r.amount_minor))
                || state.collected_minor + r.amount_minor < 0
            {
                return Err(invalid(
                    "La corrección dejaría un importe negativo o fuera del límite.",
                ));
            }
        } else {
            if state.collected_minor != r.expected_minor {
                return Err(AppError::Conflict(
                    "El saldo cambió. Actualiza los movimientos.".into(),
                ));
            }
            if r.kind == "receipt" && r.amount_minor > state.total_minor - state.collected_minor {
                return Err(invalid("El abono supera el saldo pendiente."));
            }
            if r.kind == "refund" && r.amount_minor != state.credit_minor {
                return Err(invalid(
                    "La devolución debe cubrir todo el saldo a favor vigente.",
                ));
            }
        }
        self.access
            .require(
                &mut tx,
                operation,
                AccessScope::Repairs,
                Some(r.order_id),
                true,
            )
            .await?;
        let id = self.repository.insert(&mut tx, user.id, &r, &hash).await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: user.id,
                    operation,
                    entity,
                    entity_id: Some(id),
                    order_id: Some(r.order_id),
                    reason: Some(&r.reason),
                },
                &[
                    ("amountMinor", None, Some(r.amount_minor.to_string())),
                    ("method", None, Some(r.method.clone())),
                ],
            )
            .await?;
        let result = self.repository.state(&mut tx, r.order_id).await?;
        tx.commit().await?;
        Ok(result)
    }
}
