use super::{money::Money, EntityId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationMode {
    Manual,
    Auto,
}

#[derive(Debug, Clone)]
pub struct RepairTracking {
    pub order_id: EntityId,
    pub technician: String,
    pub public_note: String,
    pub internal_note: String,
    pub publication_mode: PublicationMode,
    pub revision: i64,
}

#[derive(Debug, Clone)]
pub struct OrderStage {
    pub id: EntityId,
    pub order_id: EntityId,
    pub stage_id: Option<EntityId>,
    pub custom_name: Option<String>,
    pub position: i64,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PublicationSetting {
    pub order_id: EntityId,
    pub key: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ProgressEvent {
    pub id: EntityId,
    pub order_stage_id: EntityId,
    pub author_id: EntityId,
    pub title: String,
    pub detail: String,
    pub public: bool,
    pub occurred_at: String,
}

#[derive(Debug, Clone)]
pub struct TechnicalNote {
    pub id: EntityId,
    pub order_id: EntityId,
    pub author_id: EntityId,
    pub text: String,
    pub public: bool,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct Part {
    pub id: EntityId,
    pub order_id: EntityId,
    pub author_id: EntityId,
    pub name: String,
    pub quantity: i64,
    pub unit_cost: Money,
}

#[derive(Debug, Clone)]
pub struct Warranty {
    pub id: EntityId,
    pub order_id: EntityId,
    pub author_id: EntityId,
    pub coverage: String,
    pub days: i64,
}
