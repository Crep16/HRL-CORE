use crate::{
    domain::{contracts::SecretText, security::DeviceCredential},
    error::AppError,
};
use aes_gcm::{
    aead::{Aead, Payload},
    Aes256Gcm, KeyInit, Nonce,
};
use hmac::{Hmac, Mac};
use rand_core::{OsRng, RngCore};
use sha2::Sha256;
use std::sync::Arc;
use zeroize::Zeroizing;

pub trait KeyProvider: Send + Sync {
    fn active_key_id(&self) -> &str;
    fn key(&self, id: &str) -> Result<&[u8; 32], AppError>;
}

pub struct CryptoService {
    keys: Arc<dyn KeyProvider>,
}

impl CryptoService {
    pub fn new(keys: Arc<dyn KeyProvider>) -> Self {
        Self { keys }
    }

    pub fn encrypt(&self, order_id: i64, plaintext: &str) -> Result<DeviceCredential, AppError> {
        let id = self.keys.active_key_id();
        let cipher = Aes256Gcm::new_from_slice(self.keys.key(id)?).map_err(|_| AppError::Crypto)?;
        let mut nonce = [0_u8; 12];
        OsRng
            .try_fill_bytes(&mut nonce)
            .map_err(|_| AppError::Crypto)?;
        let context = format!("HRL-CORE:device-password:v1:{order_id}");
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: plaintext.as_bytes(),
                    aad: context.as_bytes(),
                },
            )
            .map_err(|_| AppError::Crypto)?;
        DeviceCredential::from_encrypted(order_id, id.to_owned(), nonce, ciphertext)
            .map_err(|_| AppError::Crypto)
    }

    pub fn decrypt(&self, encrypted: &DeviceCredential) -> Result<SecretText, AppError> {
        let cipher = Aes256Gcm::new_from_slice(self.keys.key(encrypted.key_id())?)
            .map_err(|_| AppError::Crypto)?;
        let context = format!("HRL-CORE:device-password:v1:{}", encrypted.order_id());
        let bytes = Zeroizing::new(
            cipher
                .decrypt(
                    Nonce::from_slice(encrypted.nonce()),
                    Payload {
                        msg: encrypted.ciphertext(),
                        aad: context.as_bytes(),
                    },
                )
                .map_err(|_| AppError::Crypto)?,
        );
        let text = std::str::from_utf8(&bytes).map_err(|_| AppError::Crypto)?;
        Ok(SecretText(Zeroizing::new(text.to_owned())))
    }

    // Una huella autenticada evita guardar o exponer el PIN incluido en la solicitud.
    pub fn request_fingerprint(&self, bytes: &[u8]) -> Result<[u8; 32], AppError> {
        let mut mac =
            <Hmac<Sha256> as Mac>::new_from_slice(self.keys.key(self.keys.active_key_id())?)
                .map_err(|_| AppError::Crypto)?;
        mac.update(b"HRL-CORE:create-order-request:v1\0");
        mac.update(bytes);
        Ok(mac.finalize().into_bytes().into())
    }
}
