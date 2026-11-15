use serde::{Deserialize, Serialize};
use std::path::{Component, Path};
use zeroize::Zeroize;
pub fn safe_path(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 500
        && !name.starts_with('/')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-._/".contains(c))
        && name.split('/').all(|p| {
            !p.is_empty()
                && !p.ends_with('.')
                && !matches!(
                    p.split('.')
                        .next()
                        .unwrap_or("")
                        .to_ascii_uppercase()
                        .as_str(),
                    "CON"
                        | "PRN"
                        | "AUX"
                        | "NUL"
                        | "COM1"
                        | "COM2"
                        | "COM3"
                        | "COM4"
                        | "COM5"
                        | "COM6"
                        | "COM7"
                        | "COM8"
                        | "COM9"
                        | "LPT1"
                        | "LPT2"
                        | "LPT3"
                        | "LPT4"
                        | "LPT5"
                        | "LPT6"
                        | "LPT7"
                        | "LPT8"
                        | "LPT9"
                )
        })
        && Path::new(name)
            .components()
            .all(|p| matches!(p, Component::Normal(_)))
        && (name == "hrl_core_3fn.db" || name.starts_with("assets-3fn/"))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupAuth {
    pub master_password: String,
}
impl Drop for BackupAuth {
    fn drop(&mut self) {
        self.master_password.zeroize();
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupConfiguration {
    pub master_password: String,
    pub automatic: bool,
}
impl Drop for BackupConfiguration {
    fn drop(&mut self) {
        self.master_password.zeroize();
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestoreBackup {
    pub current_master_password: String,
    pub backup_master_password: String,
}
impl Drop for RestoreBackup {
    fn drop(&mut self) {
        self.current_master_password.zeroize();
        self.backup_master_password.zeroize();
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatus {
    pub folder: String,
    pub automatic: bool,
    pub warning: Option<String>,
    pub files: Vec<BackupFile>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFile {
    pub name: String,
    pub bytes: u64,
}
