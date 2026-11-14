use super::{access::AccessService, audit::AuditService};
use crate::{
    domain::{contracts::AccessScope, report_contracts::*},
    error::AppError,
    repositories::{audit::AuditRecord, orders::OrderReader, reports::ReportRepository},
};
use sqlx::SqlitePool;
use std::sync::Arc;
pub struct ReportService {
    pool: SqlitePool,
    repository: Arc<ReportRepository>,
    orders: Arc<dyn OrderReader>,
    access: Arc<AccessService>,
    audit: Arc<AuditService>,
}
impl ReportService {
    pub fn new(
        pool: SqlitePool,
        repository: Arc<ReportRepository>,
        orders: Arc<dyn OrderReader>,
        access: Arc<AccessService>,
        audit: Arc<AuditService>,
    ) -> Self {
        Self {
            pool,
            repository,
            orders,
            access,
            audit,
        }
    }
    pub async fn logbook(&self, filters: LogbookFilters) -> Result<LogbookPage, AppError> {
        if filters.page < 0
            || filters.page > 1_000_000
            || [
                &filters.query,
                &filters.author_id,
                &filters.operation,
                &filters.order_id,
                &filters.from_utc,
                &filters.to_utc,
            ]
            .iter()
            .any(|v| v.len() > 200)
        {
            return Err(AppError::Validation(
                "Revisa los filtros de bitácora.".into(),
            ));
        }
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(&mut tx, "logbook.view", AccessScope::Logbook, None, false)
            .await?;
        let result = self.repository.logbook(&mut tx, &filters).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn finances(&self) -> Result<Vec<FinancialOrder>, AppError> {
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(&mut tx, "logbook.view", AccessScope::Logbook, None, false)
            .await?;
        let mut result = self.repository.finances(&mut tx).await?;
        for order in &mut result {
            order.costs_minor = order
                .expenses
                .iter()
                .try_fold(0_i64, |sum, e| sum.checked_add(e.amount_minor))
                .ok_or_else(|| {
                    AppError::Validation("Los costos superan el límite admitido.".into())
                })?;
            order.profit_minor = order.total_minor - order.costs_minor;
            order.classification = match (order.paid, order.delivered) {
                (true, true) => "completed",
                (true, false) => "paid_uncollected",
                (false, true) => "delivered_unpaid",
                _ => "pending",
            }
            .into();
            order.excluded |= order.status.trim().to_lowercase().starts_with("cancelad");
            let mut running = 0_i64;
            for p in &order.payments {
                let previous = running;
                running = running.checked_add(p.amount_minor).ok_or_else(|| {
                    AppError::Validation("Los cobros superan el límite admitido.".into())
                })?;
                if running < order.total_minor {
                    order.settled_at = None;
                } else if previous < order.total_minor {
                    order.settled_at = p.occurred_at.clone();
                }
            }
        }
        tx.commit().await?;
        Ok(result)
    }
    pub async fn expense(&self, p: ExpensePayload) -> Result<(), AppError> {
        if p.description.trim().is_empty()
            || p.description.len() > 500
            || p.reason.trim().is_empty()
            || p.reason.len() > 1000
            || !(1..=9_000_000_000_000).contains(&p.amount_minor)
        {
            return Err(AppError::Validation(
                "Escribe concepto, importe positivo y motivo del gasto.".into(),
            ));
        }
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        self.access
            .require(&mut tx, "logbook.view", AccessScope::Logbook, None, false)
            .await?;
        let user = self
            .access
            .require(
                &mut tx,
                "repairs.edit",
                AccessScope::Logbook,
                Some(p.order_id),
                true,
            )
            .await?;
        let order = self.orders
            .load_any(&mut tx, p.order_id, false)
            .await?
            .ok_or(AppError::NotFound)?;
        if order.status.trim().to_lowercase().starts_with("cancelad") {
            return Err(AppError::Validation("La orden está cancelada.".into()));
        }
        let id = self.repository.expense(&mut tx, &p, user.id).await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: user.id,
                    operation: "costs.add",
                    entity: "gastos_directos",
                    entity_id: Some(id),
                    order_id: Some(p.order_id),
                    reason: Some(&p.reason),
                },
                &[
                    ("direct_cost_minor", None, Some(p.amount_minor.to_string())),
                    ("expenseDescription", None, Some(p.description)),
                ],
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
