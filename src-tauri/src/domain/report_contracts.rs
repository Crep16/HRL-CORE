use serde::{Deserialize, Serialize};
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    pub amount_minor: i64,
    pub occurred_at: Option<String>,
    pub kind: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Expense {
    pub description: String,
    pub amount_minor: i64,
    pub occurred_at: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialOrder {
    pub database_id: i64,
    pub id: String,
    pub client: String,
    pub equipment: String,
    pub status: String,
    pub currency: String,
    pub total_minor: i64,
    pub collected_minor: i64,
    pub balance_minor: i64,
    pub costs_minor: i64,
    pub profit_minor: i64,
    pub received_minor: i64,
    pub adjusted_minor: i64,
    pub refunded_minor: i64,
    pub credit_minor: i64,
    pub delivered: bool,
    pub paid: bool,
    pub classification: String,
    pub excluded: bool,
    pub received: String,
    pub delivered_at: Option<String>,
    pub settled_at: Option<String>,
    pub payments: Vec<Payment>,
    pub expenses: Vec<Expense>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LogbookFilters {
    pub query: String,
    pub operation: String,
    pub author_id: String,
    pub page: i64,
    #[serde(default)]
    pub order_id: String,
    #[serde(default)]
    pub from_utc: String,
    #[serde(default)]
    pub to_utc: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogbookEntry {
    pub id: i64,
    pub entity: String,
    pub entity_id: Option<String>,
    pub order_id: String,
    pub author_id: String,
    pub author_name: String,
    pub operation: String,
    pub details: Vec<super::contracts::OrderHistoryChange>,
    pub occurred_at: String,
    pub event_date: Option<String>,
    pub currency: String,
    pub reason: Option<String>,
    pub result: String,
}
#[derive(Serialize)]
pub struct Author {
    pub id: String,
    pub name: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogbookPage {
    pub entries: Vec<LogbookEntry>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub authors: Vec<Author>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpensePayload {
    pub order_id: i64,
    pub description: String,
    pub amount_minor: i64,
    pub reason: String,
}
