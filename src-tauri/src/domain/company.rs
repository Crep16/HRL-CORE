use super::EntityId;

#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub id: EntityId,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Currency {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct StageDefinition {
    pub id: EntityId,
    pub name: String,
    pub predefined: bool,
    pub position: i64,
}

#[derive(Debug, Clone)]
pub struct StatusDefinition {
    pub id: EntityId,
    pub code: String,
    pub name: String,
    pub position: i64,
}

#[derive(Debug, Clone)]
pub struct Company {
    pub id: EntityId,
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
    pub labor_warranty_days: i64,
    pub parts_warranty_days: i64,
    pub revision: i64,
}

#[derive(Debug, Clone)]
pub struct DocumentSettings {
    pub id: EntityId,
    pub logo_path: Option<String>,
    pub logo_name: Option<String>,
    pub print_logo: bool,
    pub web_logo: bool,
    pub entry_footer: String,
    pub exit_footer: String,
    pub print_address: bool,
    pub request_signature: bool,
    pub web_reference: bool,
    pub clause_position: String,
    pub number_clauses: bool,
    pub paper: String,
    pub orientation: String,
    pub copies: i64,
    pub revision: i64,
}

#[derive(Debug, Clone)]
pub struct Clause {
    pub id: EntityId,
    pub title: String,
    pub text: String,
    pub entry: bool,
    pub exit: bool,
    pub position: i64,
}
