pub use crate::domain::backup::safe_path;
use crate::error::AppError;
use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, Payload},
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"HRLBKP01";
const CHUNK: usize = 1024 * 1024;
fn invalid() -> AppError {
    AppError::Validation(
        "El respaldo está incompleto, fue alterado o la clave maestra no corresponde.".into(),
    )
}
pub fn random<const N: usize>() -> Result<[u8; N], AppError> {
    let mut bytes = [0; N];
    OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|_| AppError::Crypto)?;
    Ok(bytes)
}
pub fn derive(password: &str, salt: &[u8; 16]) -> Result<Zeroizing<[u8; 32]>, AppError> {
    let mut key = Zeroizing::new([0; 32]);
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19 * 1024, 2, 1, Some(32)).map_err(|_| AppError::Crypto)?,
    )
    .hash_password_into(password.as_bytes(), salt, key.as_mut())
    .map_err(|_| AppError::Crypto)?;
    Ok(key)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    size: u64,
    hash: [u8; 32],
}
fn hash_file(path: &Path) -> Result<[u8; 32], AppError> {
    let mut file = File::open(path).map_err(|_| invalid())?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0; CHUNK];
    loop {
        let n = file.read(&mut buffer).map_err(|_| invalid())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(hash.finalize().into())
}
struct Records<T> {
    io: T,
    cipher: Aes256Gcm,
    header: [u8; 40],
    index: u64,
}
impl<T> Records<T> {
    fn aad(&self) -> Vec<u8> {
        [self.header.as_slice(), self.index.to_le_bytes().as_slice()].concat()
    }
}
impl<T: Write> Records<T> {
    fn put(&mut self, bytes: &[u8]) -> Result<(), AppError> {
        let nonce = random::<12>()?;
        let encrypted = self
            .cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: bytes,
                    aad: &self.aad(),
                },
            )
            .map_err(|_| AppError::Crypto)?;
        self.io
            .write_all(&(encrypted.len() as u32).to_le_bytes())
            .and_then(|_| self.io.write_all(&nonce))
            .and_then(|_| self.io.write_all(&encrypted))
            .map_err(|_| invalid())?;
        self.index = self.index.checked_add(1).ok_or_else(invalid)?;
        Ok(())
    }
}
impl<T: Read> Records<T> {
    fn get(&mut self) -> Result<Zeroizing<Vec<u8>>, AppError> {
        let mut length = [0; 4];
        self.io.read_exact(&mut length).map_err(|_| invalid())?;
        let length = u32::from_le_bytes(length) as usize;
        if !(17..=CHUNK + 17).contains(&length) {
            return Err(invalid());
        }
        let mut nonce = [0; 12];
        self.io.read_exact(&mut nonce).map_err(|_| invalid())?;
        let mut bytes = vec![0; length];
        self.io.read_exact(&mut bytes).map_err(|_| invalid())?;
        let clear = self
            .cipher
            .decrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: &bytes,
                    aad: &self.aad(),
                },
            )
            .map_err(|_| invalid())?;
        self.index = self.index.checked_add(1).ok_or_else(invalid)?;
        Ok(Zeroizing::new(clear))
    }
}

pub fn write_archive(
    path: &Path,
    root: &Path,
    files: &[String],
    key_id: &str,
    device_key: &[u8; 32],
    salt: &[u8; 16],
    key: &[u8; 32],
) -> Result<(), AppError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| invalid())?;
    let mut header = [0; 40];
    header[..8].copy_from_slice(MAGIC);
    header[8..24].copy_from_slice(salt);
    header[24..].copy_from_slice(&random::<16>()?);
    file.write_all(&header).map_err(|_| invalid())?;
    let mut stream = Records {
        io: file,
        cipher: Aes256Gcm::new_from_slice(key).map_err(|_| AppError::Crypto)?,
        header,
        index: 0,
    };
    let mut secret = Zeroizing::new(vec![4]);
    secret.extend_from_slice(device_key);
    secret.extend_from_slice(key_id.as_bytes());
    stream.put(&secret)?;
    let mut buffer = Zeroizing::new(vec![0; CHUNK + 1]);
    buffer[0] = 2;
    for name in files {
        if !safe_path(name) {
            return Err(invalid());
        }
        let source = root.join(name);
        let metadata = fs::symlink_metadata(&source).map_err(|_| invalid())?;
        if !metadata.is_file() || is_link(&metadata) {
            return Err(invalid());
        }
        let entry = Entry {
            path: name.clone(),
            size: metadata.len(),
            hash: hash_file(&source)?,
        };
        let mut descriptor = vec![1];
        descriptor.extend(serde_json::to_vec(&entry).map_err(|_| invalid())?);
        stream.put(&descriptor)?;
        let mut source = File::open(source).map_err(|_| invalid())?;
        let mut copied = 0_u64;
        let mut hash = Sha256::new();
        loop {
            let n = source.read(&mut buffer[1..]).map_err(|_| invalid())?;
            if n == 0 {
                break;
            }
            copied += n as u64;
            hash.update(&buffer[1..=n]);
            stream.put(&buffer[..=n])?;
        }
        if copied != entry.size || <[u8; 32]>::from(hash.finalize()) != entry.hash {
            return Err(invalid());
        }
    }
    stream.put(&[3])?;
    stream.io.sync_all().map_err(|_| invalid())?;
    Ok(())
}
pub struct RestoredKey {
    pub id: String,
    pub key: Zeroizing<[u8; 32]>,
}
pub fn read_archive(
    path: &Path,
    destination: &Path,
    password: &str,
) -> Result<RestoredKey, AppError> {
    decode(path, Some(destination), password, None)
}
pub fn verify_archive(path: &Path, key: &[u8; 32]) -> Result<(), AppError> {
    decode(path, None, "", Some(key)).map(|_| ())
}
fn decode(
    path: &Path,
    destination: Option<&Path>,
    password: &str,
    key: Option<&[u8; 32]>,
) -> Result<RestoredKey, AppError> {
    let mut file = File::open(path).map_err(|_| invalid())?;
    let mut header = [0; 40];
    file.read_exact(&mut header).map_err(|_| invalid())?;
    if &header[..8] != MAGIC {
        return Err(invalid());
    }
    let key = if let Some(key) = key {
        Zeroizing::new(*key)
    } else {
        derive(password, header[8..24].try_into().map_err(|_| invalid())?)?
    };
    let mut stream = Records {
        io: file,
        cipher: Aes256Gcm::new_from_slice(key.as_ref()).map_err(|_| AppError::Crypto)?,
        header,
        index: 0,
    };
    let secret = stream.get()?;
    if secret.len() < 34 || secret.len() > 150 || secret[0] != 4 {
        return Err(invalid());
    }
    let restored = RestoredKey {
        id: String::from_utf8(secret[33..].to_vec()).map_err(|_| invalid())?,
        key: Zeroizing::new(secret[1..33].try_into().map_err(|_| invalid())?),
    };
    let mut seen = HashSet::new();
    loop {
        let record = stream.get()?;
        if record.as_slice() == [3] {
            let mut extra = [0];
            if stream.io.read(&mut extra).map_err(|_| invalid())? != 0
                || !seen.contains("hrl_core_3fn.db")
            {
                return Err(invalid());
            }
            return Ok(restored);
        }
        if record.first() != Some(&1) || record.len() > 4096 {
            return Err(invalid());
        }
        let entry: Entry = serde_json::from_slice(&record[1..]).map_err(|_| invalid())?;
        if !safe_path(&entry.path) || !seen.insert(entry.path.clone()) {
            return Err(invalid());
        }
        let mut output = if let Some(destination) = destination {
            let output = destination.join(&entry.path);
            fs::create_dir_all(output.parent().ok_or_else(invalid)?).map_err(|_| invalid())?;
            Some(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(output)
                    .map_err(|_| invalid())?,
            )
        } else {
            None
        };
        let mut size = 0_u64;
        let mut hash = Sha256::new();
        while size < entry.size {
            let record = stream.get()?;
            if record.first() != Some(&2) || record.len() < 2 {
                return Err(invalid());
            }
            size = size
                .checked_add((record.len() - 1) as u64)
                .ok_or_else(invalid)?;
            if size > entry.size {
                return Err(invalid());
            }
            if let Some(output) = output.as_mut() {
                output.write_all(&record[1..]).map_err(|_| invalid())?;
            }
            hash.update(&record[1..]);
        }
        if let Some(output) = output.as_mut() {
            output.sync_all().map_err(|_| invalid())?;
        }
        if <[u8; 32]>::from(hash.finalize()) != entry.hash {
            return Err(invalid());
        }
    }
}
pub fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encrypted_chunks_restore_without_source_and_reject_wrong_keys_truncation_and_path_escape() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        fs::write(source.path().join("hrl_core_3fn.db"), vec![42; CHUNK + 200]).unwrap();
        let salt = random().unwrap();
        let key = derive("maestra-de-prueba", &salt).unwrap();
        let archive = source.path().join("copia.hrlbackup");
        write_archive(
            &archive,
            source.path(),
            &["hrl_core_3fn.db".into()],
            "windows-dpapi-v1",
            &[7; 32],
            &salt,
            &key,
        )
        .unwrap();
        let wrong = tempfile::tempdir().unwrap();
        assert!(read_archive(&archive, wrong.path(), "incorrecta").is_err());
        let restored = read_archive(&archive, target.path(), "maestra-de-prueba").unwrap();
        assert_eq!(restored.key.as_ref(), &[7; 32]);
        assert_eq!(
            fs::read(target.path().join("hrl_core_3fn.db")).unwrap(),
            vec![42; CHUNK + 200]
        );
        let bytes = fs::read(&archive).unwrap();
        assert!(!bytes.windows(16).any(|part| part == [42; 16]));
        let second = source.path().join("otra.hrlbackup");
        write_archive(
            &second,
            source.path(),
            &["hrl_core_3fn.db".into()],
            "windows-dpapi-v1",
            &[7; 32],
            &salt,
            &key,
        )
        .unwrap();
        let mut mixed = fs::read(&second).unwrap();
        mixed[..40].copy_from_slice(&bytes[..40]);
        fs::write(&second, mixed).unwrap();
        assert!(verify_archive(&second, &key).is_err());
        fs::write(&archive, &bytes[..bytes.len() - 20]).unwrap();
        let partial = tempfile::tempdir().unwrap();
        assert!(read_archive(&archive, partial.path(), "maestra-de-prueba").is_err());
        for path in [
            "../hrl_core_3fn.db",
            "assets-3fn/../keys/x",
            "C:/datos",
            "/datos",
            "assets-3fn/x:ads",
            "assets-3fn\\x",
            "assets-3fn//x",
        ] {
            assert!(!safe_path(path));
        }
    }
}
