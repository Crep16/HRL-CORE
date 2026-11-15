use super::crypto::KeyProvider;
use crate::error::AppError;
use rand_core::{OsRng, RngCore};
use std::{fs, io::Write, path::Path};
use zeroize::Zeroizing;

/// Custodia de Windows: DPAPI del usuario protege la clave AES en un archivo externo.
/// Perder el archivo o cambiar de usuario de Windows impide recuperar las credenciales.
pub struct OsKeyStore {
    key: Zeroizing<[u8; 32]>,
}
const KEY_ID: &str = "windows-dpapi-v1";

impl OsKeyStore {
    pub fn open(directory: &Path) -> Result<Self, AppError> {
        Self::open_key(directory, true)
    }

    pub fn open_existing(directory: &Path) -> Result<Self, AppError> {
        Self::open_key(directory, false)
    }

    fn open_key(directory: &Path, allow_creation: bool) -> Result<Self, AppError> {
        fs::create_dir_all(directory).map_err(|_| AppError::Crypto)?;
        let file = directory.join("device-password-v1.dpapi");
        let bytes = if file.exists() {
            read_protected(&file)?
        } else {
            if !allow_creation {
                return Err(AppError::Crypto);
            }
            let mut key = Zeroizing::new([0_u8; 32]);
            OsRng
                .try_fill_bytes(key.as_mut())
                .map_err(|_| AppError::Crypto)?;
            let protected = protect(key.as_ref(), false)?;
            match fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&file)
            {
                Ok(mut output) => {
                    output
                        .write_all(&protected)
                        .and_then(|_| output.sync_all())
                        .map_err(|_| AppError::Crypto)?;
                    protected
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    read_protected(&file)?
                }
                Err(_) => return Err(AppError::Crypto),
            }
        };
        let clear = Zeroizing::new(protect(&bytes, true)?);
        if clear.len() != 32 {
            return Err(AppError::Crypto);
        }
        let mut key = Zeroizing::new([0_u8; 32]);
        key.copy_from_slice(&clear);
        Ok(Self { key })
    }
}
fn read_protected(file: &Path) -> Result<Vec<u8>, AppError> {
    let length = fs::metadata(file).map_err(|_| AppError::Crypto)?.len();
    if length == 0 || length > 4096 {
        return Err(AppError::Crypto);
    }
    fs::read(file).map_err(|_| AppError::Crypto)
}
impl KeyProvider for OsKeyStore {
    fn active_key_id(&self) -> &str {
        KEY_ID
    }
    fn key(&self, id: &str) -> Result<&[u8; 32], AppError> {
        if id != KEY_ID {
            return Err(AppError::Crypto);
        }
        Ok(&self.key)
    }
}

#[cfg(windows)]
pub(crate) fn protect(bytes: &[u8], decrypt: bool) -> Result<Vec<u8>, AppError> {
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
        },
    };
    use zeroize::Zeroize;
    let input = CRYPT_INTEGER_BLOB {
        cbData: bytes.len().try_into().map_err(|_| AppError::Crypto)?,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    // DPAPI entrega un búfer propio; se copia y libera con LocalFree, incluso al fallar.
    let ok = unsafe {
        if decrypt {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptProtectData(
                &input,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        }
    };
    if ok == 0 {
        if !output.pbData.is_null() {
            unsafe {
                LocalFree(output.pbData as _);
            }
        }
        return Err(AppError::Crypto);
    }
    let result =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    if decrypt {
        unsafe {
            std::slice::from_raw_parts_mut(output.pbData, output.cbData as usize).zeroize();
        }
    }
    unsafe {
        LocalFree(output.pbData as _);
    }
    Ok(result)
}

#[cfg(not(windows))]
pub(crate) fn protect(_bytes: &[u8], _decrypt: bool) -> Result<Vec<u8>, AppError> {
    Err(AppError::Crypto)
}
