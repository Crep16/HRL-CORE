use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompanySettings {
    pub name: String,
    pub tax_id: String,
    pub phone: String,
    pub whatsapp: String,
    pub email: String,
    pub website: String,
    pub address: String,
    pub hours: String,
    pub country: String,
    pub locale: String,
    pub timezone: String,
    pub currency: String,
    pub show_tax: bool,
    pub tax_rate_thousandths: i64,
    pub tax_name: String,
    pub labor_days: i64,
    pub parts_days: i64,
    pub logo_name: String,
    pub logo_print: bool,
    pub logo_web: bool,
    pub entry_footer: String,
    pub exit_footer: String,
    pub print_address: bool,
    pub signature: bool,
    pub web_reference: bool,
    pub clause_position: String,
    pub number_clauses: bool,
    pub paper: String,
    pub orientation: String,
    pub copies: i64,
    pub prefix: String,
    pub logo: String,
    pub next_number: i64,
    pub clauses: Vec<CompanyClause>,
    #[serde(default)]
    pub revision: i64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompanyClause {
    pub id: String,
    pub title: String,
    pub text: String,
    pub entry: bool,
    pub exit: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveCompanyPayload {
    pub value: CompanySettings,
    pub expected_revision: i64,
}
