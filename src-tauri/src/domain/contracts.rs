use crate::domain::security::User;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupStatus {
    pub setup_complete: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryChallengeResponse {
    /// Se firman exactamente estos bytes UTF-8, sin decodificar ni agregar saltos de línea.
    pub challenge: String,
    pub admin_id: i64,
    pub admin_username: String,
    pub expires_in_seconds: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerifyRecoveryPayload {
    pub challenge: String,
    /// Firma Ed25519 de 64 bytes codificada como 128 caracteres hexadecimales.
    pub signature: String,
    pub new_master_password: String,
    pub new_admin_password: String,
}
impl Drop for VerifyRecoveryPayload {
    fn drop(&mut self) {
        self.new_master_password.zeroize();
        self.new_admin_password.zeroize();
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryResponse {
    pub admin_id: i64,
    pub requires_login: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

impl Drop for LoginPayload {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetupPayload {
    pub name: String,
    pub username: String,
    pub password: String,
    pub confirmation: String,
    pub master_password: String,
    pub master_confirmation: String,
}
impl Drop for SetupPayload {
    fn drop(&mut self) {
        self.password.zeroize();
        self.confirmation.zeroize();
        self.master_password.zeroize();
        self.master_confirmation.zeroize();
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub user: User,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccessScope {
    Clients,
    Repairs,
    Company,
    Logbook,
    Users,
}
impl AccessScope {
    pub fn code(self) -> &'static str {
        match self {
            Self::Clients => "clients",
            Self::Repairs => "repairs",
            Self::Company => "company",
            Self::Logbook => "logbook",
            Self::Users => "users",
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemporaryAccessPayload {
    pub username: String,
    pub password: String,
    pub scope: AccessScope,
    pub order_id: Option<i64>,
    pub operation: String,
    pub reason: String,
}
impl Drop for TemporaryAccessPayload {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryAccessResponse {
    pub id: i64,
    pub recipient_id: i64,
    pub approver_id: i64,
    pub scope: AccessScope,
    pub order_id: Option<i64>,
    pub operations: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClientInput {
    pub id: Option<i64>,
    pub full_name: String,
    pub phone: String,
    pub email: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EquipmentInput {
    pub id: Option<i64>,
    pub item: String,
    pub brand: String,
    pub model: String,
    pub serial: String,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EquipmentHistory {
    pub previous_workshop: bool,
    pub previous_workshop_detail: String,
    pub suffered_impact: bool,
    pub impact_detail: String,
    pub liquid_spill: bool,
    pub liquid_detail: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InitialPaymentInput {
    pub request_id: String,
    pub amount_minor: i64,
    pub method: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateOrderPayload {
    pub client: ClientInput,
    pub equipment: EquipmentInput,
    pub history: EquipmentHistory,
    pub folio_prefix: String,
    pub sequence_start: i64,
    pub status: String,
    pub currency: String,
    pub received_at: String,
    pub estimated_delivery_at: Option<String>,
    pub reported_failure: String,
    pub planned_work: String,
    pub reception_notes: String,
    pub price_minor: i64,
    pub device_password: Option<String>,
    pub initial_payment: Option<InitialPaymentInput>,
}
impl Drop for CreateOrderPayload {
    fn drop(&mut self) {
        if let Some(secret) = &mut self.device_password {
            secret.zeroize();
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetOrderPayload {
    #[serde(default)]
    pub include_deleted: bool,
    pub order_id: i64,
    pub scope: AccessScope,
    #[serde(default)]
    pub include_device_password: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListOrdersPayload {
    #[serde(default)]
    pub trashed: bool,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub offset: i64,
    pub limit: i64,
    #[serde(default)]
    pub field: OrderSearchField,
    #[serde(default)]
    pub oldest_first: bool,
}

#[derive(Default, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum OrderSearchField {
    #[default]
    All,
    Order,
    Client,
    Serial,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderListItem {
    pub id: i64,
    pub folio: String,
    pub client_name: String,
    pub serial: String,
    pub item: String,
    pub status: String,
    pub received_at: String,
    pub currency: String,
    pub price_minor: i64,
    pub net_minor: i64,
    pub pending_minor: i64,
    pub credit_minor: i64,
    pub delivered: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderListResponse {
    pub items: Vec<OrderListItem>,
    pub total: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FindOrderPayload {
    pub reference: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrderHistoryPayload {
    pub order_id: i64,
    #[serde(default)]
    pub offset: i64,
    pub limit: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderHistoryChange {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderHistoryEvent {
    pub id: i64,
    pub author_id: i64,
    pub author_name: String,
    pub operation: String,
    pub entity: String,
    pub entity_id: Option<String>,
    pub reason: Option<String>,
    pub result: String,
    pub created_at: String,
    pub changes: Vec<OrderHistoryChange>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderHistoryResponse {
    pub items: Vec<OrderHistoryEvent>,
    pub total: i64,
}

pub struct SecretText(pub zeroize::Zeroizing<String>);
impl Serialize for SecretText {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub id: i64,
    pub folio: String,
    pub client_id: i64,
    pub equipment_id: i64,
    pub received_by: i64,
    pub status: String,
    pub currency: String,
    pub received_at: String,
    pub estimated_delivery_at: Option<String>,
    pub reported_failure: String,
    pub planned_work: String,
    pub reception_notes: String,
    pub price_minor: i64,
    pub net_minor: i64,
    pub pending_minor: i64,
    pub credit_minor: i64,
    pub has_device_password: bool,
    pub client: ClientOutput,
    pub equipment: EquipmentOutput,
    pub history: EquipmentHistory,
    pub revision: i64,
    pub delivered: bool,
    pub delivered_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_password: Option<SecretText>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientOutput {
    pub id: i64,
    pub full_name: String,
    pub phone: String,
    pub email: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentOutput {
    pub id: i64,
    pub item: String,
    pub brand: String,
    pub model: String,
    pub serial: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditOrderPayload {
    pub order_id: i64,
    pub revision: i64,
    pub value: CreateOrderPayload,
    pub delivered: bool,
    pub change_password: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrashOrderPayload {
    pub order_id: i64,
    pub revision: i64,
    pub deleted: bool,
}
