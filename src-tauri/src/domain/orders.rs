use super::{money::Money, EntityId, ModelError};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Client {
    pub id: EntityId,
    pub full_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Equipment {
    pub id: EntityId,
    pub client_id: EntityId,
    pub item: String,
    pub brand: String,
    pub model: String,
    pub serial: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum OrderStatus {
    Catalog(EntityId),
    Custom(String),
}

impl OrderStatus {
    pub fn custom(text: String) -> Result<Self, ModelError> {
        if text.trim().is_empty() || text.len() > 16000 {
            return Err(ModelError::InvalidChoice);
        }
        Ok(Self::Custom(text))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RepairOrder {
    pub id: EntityId,
    pub folio: String,
    pub equipment_id: EntityId,
    pub received_by: EntityId,
    pub status: OrderStatus,
    pub currency: String,
    pub received_at: String,
    pub estimated_delivery_at: Option<String>,
    pub delivered_at: Option<String>,
    pub delivered: bool,
    pub reported_failure: String,
    pub planned_work: String,
    pub reception_notes: String,
    pub price: Money,
    pub previous_workshop: bool,
    pub previous_workshop_detail: String,
    pub suffered_impact: bool,
    pub impact_detail: String,
    pub liquid_spill: bool,
    pub liquid_detail: String,
    pub revision: i64,
    pub deleted_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct FolioSequence {
    pub prefix: String,
    pub next_number: i64,
}

#[derive(Debug, Clone)]
pub struct Diagnosis {
    pub order_id: EntityId,
    pub author_id: EntityId,
    pub text: String,
    pub revision: i64,
    pub updated_at: String,
}
