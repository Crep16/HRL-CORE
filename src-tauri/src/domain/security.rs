use super::{EntityId, ModelError};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AccessLevel {
    #[serde(rename = "complete")]
    Complete,
    #[serde(rename = "limited")]
    Limited,
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Administrator,
    Technician,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: EntityId,
    pub name: String,
    pub login_name: String,
    pub role: Role,
    pub access_level: AccessLevel,
    pub active: bool,
    pub created_at: String,
}

pub struct PasswordHash(String);

impl PasswordHash {
    pub fn from_encoded(encoded: String) -> Result<Self, ModelError> {
        if !encoded.starts_with("$argon2id$") || encoded.len() > 1024 {
            return Err(ModelError::InvalidCredential);
        }
        Ok(Self(encoded))
    }

    pub fn encoded(&self) -> &str {
        &self.0
    }
}

pub struct UserAuthentication {
    pub user_id: EntityId,
    pub password_hash: PasswordHash,
}

pub struct MasterCredential {
    pub password_hash: PasswordHash,
}

#[derive(Debug, Clone, Serialize)]
pub struct Permission {
    pub id: EntityId,
    pub section_id: EntityId,
    pub action_id: EntityId,
    pub complete_only: bool,
}

#[derive(Debug, Clone)]
pub struct UserPermission {
    pub user_id: EntityId,
    pub permission_id: EntityId,
}

#[derive(Debug, Clone)]
pub struct TemporaryAccess {
    pub id: EntityId,
    pub recipient_id: EntityId,
    pub approver_id: EntityId,
    pub section_id: EntityId,
    pub order_id: Option<EntityId>,
    pub reason: String,
    pub created_at: String,
    pub ended_at: Option<String>,
    pub end_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TemporaryPermission {
    pub access_id: EntityId,
    pub permission_id: EntityId,
}

// Sin Serialize ni Debug: el sobre cifrado no es un DTO ni parte de la auditoría.
pub struct DeviceCredential {
    order_id: EntityId,
    key_id: String,
    nonce: [u8; 12],
    ciphertext: Vec<u8>,
}

impl DeviceCredential {
    pub fn from_encrypted(
        order_id: EntityId,
        key_id: String,
        nonce: [u8; 12],
        ciphertext: Vec<u8>,
    ) -> Result<Self, ModelError> {
        if order_id <= 0 || key_id.trim().is_empty() || ciphertext.len() < 16 {
            return Err(ModelError::InvalidCredential);
        }
        Ok(Self {
            order_id,
            key_id,
            nonce,
            ciphertext,
        })
    }

    pub const fn order_id(&self) -> EntityId {
        self.order_id
    }
    pub fn key_id(&self) -> &str {
        &self.key_id
    }
    pub const fn version(&self) -> i64 {
        1
    }
    pub const fn algorithm(&self) -> &'static str {
        "AES-256-GCM"
    }
    pub fn nonce(&self) -> &[u8; 12] {
        &self.nonce
    }
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_user_has_no_credentials_and_ciphertext_requires_an_envelope() {
        let user = User {
            id: 1,
            name: "Prueba".into(),
            login_name: "prueba".into(),
            role: Role::Technician,
            access_level: AccessLevel::Limited,
            active: true,
            created_at: "2026-11-04T00:00:00Z".into(),
        };
        let json = serde_json::to_value(user).unwrap();
        assert!(json.get("passwordHash").is_none());
        assert!(json.get("ciphertext").is_none());
        assert_eq!(json["accessLevel"], "limited");
        assert!(DeviceCredential::from_encrypted(1, "".into(), [0; 12], vec![0; 16]).is_err());
        assert!(DeviceCredential::from_encrypted(1, "key-1".into(), [0; 12], vec![0; 15]).is_err());
        let credential =
            DeviceCredential::from_encrypted(1, "key-1".into(), [0; 12], vec![0; 16]).unwrap();
        assert_eq!(credential.ciphertext().len(), 16);
    }
}
