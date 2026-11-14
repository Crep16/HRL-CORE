use crate::{error::AppError, repositories::access::RecoveryAccount};
use ed25519_dalek::{Signature, VerifyingKey};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

// Clave pública del vector de prueba 1 de RFC 8032, reservada para las pruebas.
// La semilla privada de prueba solo existe bajo cfg(test). Nunca se incluye en la aplicación.
const EXAMPLE_PUBLIC_KEY: [u8; 32] = [
    0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64, 0x07, 0x3a,
    0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68, 0xf7, 0x07, 0x51, 0x1a,
];
pub const SUPPORT_RECOVERY_PUBLIC_KEY: [u8; 32] = [
    0xf8, 0x4b, 0x7f, 0x52, 0xb8, 0xed, 0x01, 0x58, 0x79, 0xb8, 0xa0, 0xbb, 0x6d, 0x5f, 0xef, 0x58,
    0x3f, 0xab, 0xbb, 0x1e, 0x33, 0xdb, 0xf8, 0x2e, 0xc3, 0x10, 0x30, 0xf0, 0x1a, 0x8c, 0x6a, 0x42,
];
pub const CHALLENGE_TTL_SECONDS: u32 = 15 * 60;

pub(crate) fn configured_public_key() -> [u8; 32] {
    // Las pruebas no requieren la clave privada del desarrollador al reemplazar la pública.
    #[cfg(test)]
    {
        EXAMPLE_PUBLIC_KEY
    }
    #[cfg(not(test))]
    {
        SUPPORT_RECOVERY_PUBLIC_KEY
    }
}

fn validate_public_key(bytes: &[u8; 32], allow_example: bool) -> Result<VerifyingKey, AppError> {
    if !allow_example && bytes == &EXAMPLE_PUBLIC_KEY {
        return Err(AppError::RecoveryUnavailable);
    }
    let key = VerifyingKey::from_bytes(bytes).map_err(|_| AppError::RecoveryUnavailable)?;
    if key.is_weak() {
        return Err(AppError::RecoveryUnavailable);
    }
    Ok(key)
}

pub(crate) fn account_fingerprint(account: &RecoveryAccount) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(account.id.to_le_bytes());
    for text in [&account.login, &account.master_hash, &account.password_hash] {
        hash.update((text.len() as u64).to_le_bytes());
        hash.update(text.as_bytes());
    }
    hash.finalize().into()
}

pub(crate) struct RecoveryBinding {
    pub admin_id: i64,
    pub fingerprint: [u8; 32],
}
struct PendingChallenge {
    token: String,
    expires_at: Instant,
    binding: RecoveryBinding,
}

#[derive(Default)]
pub(crate) struct RecoveryState {
    pending: Option<PendingChallenge>,
    last_issued: Option<Instant>,
    failed_attempts: u32,
    retry_at: Option<Instant>,
}

impl RecoveryState {
    fn public_key() -> Result<VerifyingKey, AppError> {
        validate_public_key(
            &configured_public_key(),
            cfg!(debug_assertions) || cfg!(test),
        )
    }
    pub fn issue(&mut self, account: &RecoveryAccount) -> Result<String, AppError> {
        self.issue_at(account, Instant::now())
    }
    fn issue_at(&mut self, account: &RecoveryAccount, now: Instant) -> Result<String, AppError> {
        Self::public_key()?;
        self.check_rate(now)?;
        if self
            .last_issued
            .is_some_and(|last| now.duration_since(last) < Duration::from_secs(5))
        {
            return Err(AppError::RateLimited);
        }
        let mut nonce = [0_u8; 32];
        OsRng
            .try_fill_bytes(&mut nonce)
            .map_err(|_| AppError::Crypto)?;
        let encoded: String = nonce.iter().map(|byte| format!("{byte:02x}")).collect();
        let token = format!("HRL-CORE:master-recovery:v1:{}:{encoded}", account.id);
        self.pending = Some(PendingChallenge {
            token: token.clone(),
            expires_at: now + Duration::from_secs(CHALLENGE_TTL_SECONDS.into()),
            binding: RecoveryBinding {
                admin_id: account.id,
                fingerprint: account_fingerprint(account),
            },
        });
        self.last_issued = Some(now);
        Ok(token)
    }
    pub fn consume(&mut self, token: &str, signature: &str) -> Result<RecoveryBinding, AppError> {
        self.consume_at(token, signature, Instant::now())
    }
    fn consume_at(
        &mut self,
        token: &str,
        signature: &str,
        now: Instant,
    ) -> Result<RecoveryBinding, AppError> {
        self.check_rate(now)?;
        let key = Self::public_key()?;
        let valid = self.pending.as_ref().is_some_and(|pending| {
            pending.expires_at > now
                && pending.token == token
                && decode_signature(signature).is_some_and(|signature| {
                    key.verify_strict(token.as_bytes(), &signature).is_ok()
                })
        });
        if !valid {
            self.failed_attempts = self.failed_attempts.saturating_add(1);
            if self.failed_attempts >= 5 {
                self.pending = None;
                self.retry_at = Some(now + Duration::from_secs(5));
            }
            return Err(AppError::RecoveryRejected);
        }
        self.failed_attempts = 0;
        self.retry_at = None;
        // Se consume antes de modificar SQLite: tampoco se reutiliza tras un rollback o commit incierto.
        self.pending
            .take()
            .map(|pending| pending.binding)
            .ok_or(AppError::RecoveryRejected)
    }
    fn check_rate(&self, now: Instant) -> Result<(), AppError> {
        if self.retry_at.is_some_and(|retry| now < retry) {
            Err(AppError::RateLimited)
        } else {
            Ok(())
        }
    }
}

fn decode_signature(encoded: &str) -> Option<Signature> {
    if encoded.len() != 128 {
        return None;
    }
    let mut bytes = [0_u8; 64];
    for (byte, digits) in bytes.iter_mut().zip(encoded.as_bytes().chunks_exact(2)) {
        let digits = std::str::from_utf8(digits).ok()?;
        *byte = u8::from_str_radix(digits, 16).ok()?;
    }
    Some(Signature::from_bytes(&bytes))
}

#[cfg(test)]
pub(crate) fn sign_test_challenge(token: &str) -> String {
    use ed25519_dalek::{Signer, SigningKey};
    let seed = [
        0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60, 0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec, 0x2c,
        0xc4, 0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19, 0x70, 0x3b, 0xac, 0x03, 0x1c, 0xae,
        0x7f, 0x60,
    ];
    let key = SigningKey::from_bytes(&seed);
    assert_eq!(key.verifying_key().to_bytes(), EXAMPLE_PUBLIC_KEY);
    key.sign(token.as_bytes())
        .to_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn account() -> RecoveryAccount {
        RecoveryAccount {
            id: 1,
            login: "admin".into(),
            password_hash: "test-hash".into(),
            master_hash: "master-test-hash".into(),
        }
    }
    #[test]
    fn signatures_are_strict_single_use_and_bound_to_the_issued_challenge() {
        use ed25519_dalek::{Signer, SigningKey};
        let mut state = RecoveryState::default();
        let token = state.issue(&account()).unwrap();
        let signature = sign_test_challenge(&token);
        assert!(state.consume(&(token.clone() + "x"), &signature).is_err());
        assert!(state.consume(&token, &"00".repeat(64)).is_err());
        assert!(state.consume(&token, "é").is_err());
        let other_key = SigningKey::generate(&mut OsRng);
        let wrong_signature: String = other_key
            .sign(token.as_bytes())
            .to_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert!(state.consume(&token, &wrong_signature).is_err());
        let binding = state.consume(&token, &signature).unwrap();
        assert_eq!(binding.admin_id, 1);
        assert!(state.consume(&token, &signature).is_err());
        assert!(RecoveryState::default()
            .consume(&token, &signature)
            .is_err());
    }
    #[test]
    fn distribution_rejects_example_and_weak_keys_but_accepts_a_developer_key() {
        use ed25519_dalek::SigningKey;
        assert!(validate_public_key(&SUPPORT_RECOVERY_PUBLIC_KEY, false).is_ok());
        assert!(matches!(
            validate_public_key(&EXAMPLE_PUBLIC_KEY, false),
            Err(AppError::RecoveryUnavailable)
        ));
        assert!(validate_public_key(&EXAMPLE_PUBLIC_KEY, true).is_ok());
        assert!(validate_public_key(&[0; 32], false).is_err());
        let developer = SigningKey::generate(&mut OsRng).verifying_key();
        assert!(validate_public_key(&developer.to_bytes(), false).is_ok());
    }
    #[test]
    fn expiry_replacement_rate_limits_and_external_keys_are_enforced() {
        let now = Instant::now();
        let mut state = RecoveryState::default();
        let first = state.issue_at(&account(), now).unwrap();
        assert!(matches!(
            state.issue_at(&account(), now),
            Err(AppError::RateLimited)
        ));
        let next = now + Duration::from_secs(6);
        let second = state.issue_at(&account(), next).unwrap();
        assert_ne!(first, second);
        assert!(state
            .consume_at(&first, &sign_test_challenge(&first), next)
            .is_err());
        assert!(state
            .consume_at(
                &second,
                &sign_test_challenge(&second),
                next + Duration::from_secs(900)
            )
            .is_err());
        let mut state = RecoveryState::default();
        let token = state.issue_at(&account(), now).unwrap();
        for _ in 0..5 {
            assert!(state.consume_at(&token, &"ff".repeat(64), now).is_err());
        }
        assert!(matches!(
            state.consume_at(&token, &sign_test_challenge(&token), now),
            Err(AppError::RateLimited)
        ));
        assert!(state
            .consume_at(&token, &sign_test_challenge(&token), next)
            .is_err());
    }
}
