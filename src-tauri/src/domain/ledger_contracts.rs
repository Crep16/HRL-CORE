use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentState {
    pub total_minor: i64,
    pub collected_minor: i64,
    pub received_minor: i64,
    pub adjusted_minor: i64,
    pub refunded_minor: i64,
    pub credit_minor: i64,
    pub paid: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyMovement {
    pub id: String,
    pub kind: String,
    pub payment_id: Option<String>,
    pub amount_minor: i64,
    pub effective_minor: Option<i64>,
    pub method: Option<String>,
    pub reason: Option<String>,
    pub occurred_at: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MoneyRequest {
    pub request_id: String,
    pub order_id: i64,
    pub kind: String,
    pub payment_id: Option<i64>,
    pub amount_minor: i64,
    pub expected_minor: i64,
    pub method: String,
    pub reason: String,
}
