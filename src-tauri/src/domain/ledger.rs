use super::{money::Money, EntityId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementKind {
    Receipt,
    Adjustment,
    Refund,
}

#[derive(Debug, Clone)]
pub struct FinancialRequest {
    pub id: String,
    pub author_id: EntityId,
    pub kind: MovementKind,
    pub fingerprint: [u8; 32],
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct Receipt {
    pub id: EntityId,
    pub order_id: EntityId,
    pub author_id: EntityId,
    pub method: Option<String>,
    pub amount: Money,
    pub paid_at: Option<String>,
    pub reason: Option<String>,
    pub request_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct Adjustment {
    pub id: EntityId,
    pub receipt_id: EntityId,
    pub author_id: EntityId,
    pub amount: Money,
    pub reason: String,
    pub request_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct Refund {
    pub id: EntityId,
    pub order_id: EntityId,
    pub author_id: EntityId,
    pub method: String,
    pub amount: Money,
    pub reason: String,
    pub request_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct DirectExpense {
    pub id: EntityId,
    pub order_id: EntityId,
    pub author_id: EntityId,
    pub description: String,
    pub amount: Money,
    pub reason: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub id: EntityId,
    pub author_id: EntityId,
    pub operation_id: EntityId,
    pub order_id: Option<EntityId>,
    pub entity: String,
    pub entity_id: Option<String>,
    pub reason: Option<String>,
    pub result: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct AuditDetail {
    pub id: EntityId,
    pub audit_id: EntityId,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}
