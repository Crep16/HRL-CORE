use serde::{Deserialize, Serialize};
use zeroize::Zeroize;
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Grant {
    pub section: String,
    pub action: String,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub access_level: String,
    pub active: bool,
    pub permissions: Vec<Grant>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveUserPayload {
    pub id: Option<i64>,
    pub name: String,
    pub role: String,
    pub access_level: String,
    pub active: bool,
    pub permissions: Vec<Grant>,
    pub password: String,
    pub confirmation: String,
    pub master_password: String,
}
impl Drop for SaveUserPayload {
    fn drop(&mut self) {
        self.password.zeroize();
        self.confirmation.zeroize();
        self.master_password.zeroize();
    }
}
