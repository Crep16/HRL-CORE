pub mod backup;
pub mod company;
pub mod company_contracts;
pub mod contracts;
pub mod evidence;
pub mod ledger;
pub mod money;
pub mod orders;
pub mod permissions;
pub mod portal;
pub mod repairs;
pub mod security;
pub mod user_contracts;

use std::fmt;

pub type EntityId = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelError {
    InvalidAmount,
    Overflow,
    InvalidCredential,
    InvalidChoice,
}

impl fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidAmount => "El importe no es válido.",
            Self::Overflow => "El cálculo supera el rango entero admitido.",
            Self::InvalidCredential => "La credencial protegida no es válida.",
            Self::InvalidChoice => "Selecciona un valor válido.",
        })
    }
}

impl std::error::Error for ModelError {}

pub mod ledger_contracts;
pub mod repair_contracts;

pub mod report_contracts;

pub mod evidence_contracts;
