use super::{
    access::AccessService,
    backup_archive as archive,
    crypto::{CryptoService, KeyProvider},
    key_store,
};
use crate::{
    domain::{backup::*, contracts::AccessScope, security::AccessLevel},
    error::AppError,
    repositories::backups::BackupRepository,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
use zeroize::Zeroizing;

fn failure(text: &str) -> AppError {
    AppError::Validation(text.into())
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| {
            failure("No se pudo crear el archivo de respaldo. Revisa espacio y permisos.")
        })?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| failure("No se pudo guardar el archivo completo."))
}
fn unique() -> Result<String, AppError> {
    Ok(format!(
        "{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError::Crypto)?
            .as_millis(),
        u64::from_le_bytes(archive::random()?)
    ))
}
struct Stage(PathBuf);
impl Stage {
    fn new(parent: &Path) -> Result<Self, AppError> {
        let path = parent.join(format!(".hrl-respaldo-{}", unique()?));
        fs::create_dir(&path).map_err(|_| {
            failure("No se pudo preparar la copia temporal. Revisa espacio y permisos.")
        })?;
        Ok(Self(path))
    }
}
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[derive(Serialize, Deserialize)]
struct AutomaticKey {
    salt: [u8; 16],
    key: [u8; 32],
    master_fingerprint: [u8; 32],
}
impl Drop for AutomaticKey {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.key.zeroize();
    }
}
struct ImportedKey {
    id: String,
    value: Zeroizing<[u8; 32]>,
}
impl KeyProvider for ImportedKey {
    fn active_key_id(&self) -> &str {
        &self.id
    }
    fn key(&self, id: &str) -> Result<&[u8; 32], AppError> {
        if id == self.id {
            Ok(&self.value)
        } else {
            Err(AppError::Crypto)
        }
    }
}

pub struct BackupService {
    pool: SqlitePool,
    repository: Arc<BackupRepository>,
    access: Arc<AccessService>,
    keys: Arc<dyn KeyProvider>,
    root: Option<PathBuf>,
    gate: Mutex<()>,
    restored: AtomicBool,
}
impl BackupService {
    pub fn new(
        pool: SqlitePool,
        repository: Arc<BackupRepository>,
        access: Arc<AccessService>,
        keys: Arc<dyn KeyProvider>,
        root: Option<PathBuf>,
    ) -> Self {
        Self {
            pool,
            repository,
            access,
            keys,
            root,
            gate: Mutex::new(()),
            restored: AtomicBool::new(false),
        }
    }
    fn root(&self) -> Result<&Path, AppError> {
        self.root
            .as_deref()
            .ok_or_else(|| failure("Los respaldos requieren una base instalada en disco."))
    }
    async fn authorize(&self, password: &str, blank: bool) -> Result<(), AppError> {
        if blank && !self.repository.configured(&self.pool).await? {
            return Ok(());
        }
        let mut db = self.pool.acquire().await?;
        let user = self
            .access
            .require(&mut db, "users.manage", AccessScope::Users, None, false)
            .await?;
        if user.access_level != AccessLevel::Complete {
            return Err(AppError::Forbidden);
        }
        self.access.verify_master(&mut db, password).await
    }
    pub async fn authorize_restore(&self, input: &RestoreBackup) -> Result<(), AppError> {
        if input.current_master_password.len() > 1024
            || input.backup_master_password.len() > 1024
            || input.backup_master_password.chars().count() < 8
        {
            return Err(failure("Revisa la clave maestra del respaldo."));
        }
        let _gate = self.access.lock().await;
        self.authorize(&input.current_master_password, true).await
    }
    fn config_path(&self) -> Result<PathBuf, AppError> {
        Ok(self.root()?.join("keys").join("respaldo-automatico.dpapi"))
    }
    fn configuration(&self) -> Result<Option<AutomaticKey>, AppError> {
        let path = self.config_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let metadata = fs::symlink_metadata(&path).map_err(|_| AppError::Crypto)?;
        if archive::is_link(&metadata) || metadata.len() > 4096 {
            return Err(AppError::Crypto);
        }
        let protected = fs::read(path).map_err(|_| AppError::Crypto)?;
        let clear = Zeroizing::new(key_store::protect(&protected, true)?);
        Ok(Some(
            serde_json::from_slice(&clear).map_err(|_| AppError::Crypto)?,
        ))
    }
    pub async fn status(&self, auth: &BackupAuth) -> Result<BackupStatus, AppError> {
        let _job = self.gate.lock().await;
        let _gate = self.access.lock().await;
        self.authorize(&auth.master_password, false).await?;
        let folder = self.root()?.join("Respaldos");
        fs::create_dir_all(&folder)
            .map_err(|_| failure("No se pudo abrir la carpeta de respaldos."))?;
        let mut files = vec![];
        for entry in fs::read_dir(&folder)
            .map_err(|_| failure("No se pudo consultar la carpeta de respaldos."))?
        {
            let entry = entry.map_err(|_| AppError::Crypto)?;
            let metadata = fs::symlink_metadata(entry.path()).map_err(|_| AppError::Crypto)?;
            if metadata.is_file()
                && !archive::is_link(&metadata)
                && entry.path().extension().is_some_and(|e| e == "hrlbackup")
            {
                files.push(BackupFile {
                    name: entry.file_name().to_string_lossy().into_owned(),
                    bytes: metadata.len(),
                });
            }
        }
        files.sort_by(|a, b| b.name.cmp(&a.name));
        let (configured, mut warning) = match self.configuration() {
            Ok(config) => (config, None),
            Err(_) => (None, Some("No se pudo leer la configuración del respaldo al cerrar. Vuelve a activar la opción con la clave maestra.".to_owned())),
        };
        let automatic = if let Some(config) = configured {
            let matches = config.master_fingerprint
                == <[u8; 32]>::from(Sha256::digest(
                    self.repository.master_hash(&self.pool).await?.as_bytes(),
                ));
            if !matches {
                warning = Some(
                    "Cambió la clave maestra. Vuelve a activar el respaldo al cerrar.".to_owned(),
                );
            }
            matches
        } else {
            false
        };
        Ok(BackupStatus {
            folder: folder.to_string_lossy().into_owned(),
            automatic,
            warning,
            files,
        })
    }
    pub async fn configure(&self, input: &BackupConfiguration) -> Result<(), AppError> {
        let _job = self.gate.lock().await;
        let _gate = self.access.lock().await;
        self.authorize(&input.master_password, false).await?;
        let path = self.config_path()?;
        if !input.automatic {
            if path.exists() {
                fs::remove_file(path).map_err(|_| AppError::Crypto)?;
            }
            return Ok(());
        }
        let salt = archive::random()?;
        let key = archive::derive(&input.master_password, &salt)?;
        let config = AutomaticKey {
            salt,
            key: *key,
            master_fingerprint: Sha256::digest(
                self.repository.master_hash(&self.pool).await?.as_bytes(),
            )
            .into(),
        };
        let clear = Zeroizing::new(serde_json::to_vec(&config).map_err(|_| AppError::Crypto)?);
        let protected = key_store::protect(&clear, false)?;
        fs::create_dir_all(path.parent().ok_or(AppError::Crypto)?).map_err(|_| AppError::Crypto)?;
        let temporary = path.with_file_name(format!("respaldo-{}.partial", unique()?));
        write_new(&temporary, &protected)?;
        // En Windows rename no reemplaza un archivo existente. Conservarlo hasta tener el nuevo completo.
        let previous = path.with_file_name(format!("respaldo-anterior-{}.dpapi", unique()?));
        if path.exists() {
            fs::rename(&path, &previous).map_err(|_| AppError::Crypto)?;
        }
        if fs::rename(&temporary, &path).is_err() {
            if previous.exists() {
                let _ = fs::rename(&previous, &path);
            }
            return Err(AppError::Crypto);
        }
        if previous.exists() {
            let _ = fs::remove_file(previous);
        }
        Ok(())
    }
    pub async fn create(&self, auth: &BackupAuth) -> Result<String, AppError> {
        let _job = self.gate.lock().await;
        let _gate = self.access.lock().await;
        self.authorize(&auth.master_password, false).await?;
        let salt = archive::random()?;
        let key = archive::derive(&auth.master_password, &salt)?;
        self.capture(&salt, &key).await
    }
    pub async fn automatic(&self) -> Result<Option<String>, AppError> {
        let _job = self.gate.lock().await;
        let _gate = self.access.lock().await;
        let Some(config) = self.configuration().map_err(|_| failure("No se pudo leer la configuración del respaldo al cerrar. Un usuario Completo debe volver a activarlo desde Herramientas con la clave maestra."))? else {
            return Ok(None);
        };
        if config.master_fingerprint
            != <[u8; 32]>::from(Sha256::digest(
                self.repository.master_hash(&self.pool).await?.as_bytes(),
            ))
        {
            return Err(failure(
                "Cambió la clave maestra. Un usuario Completo debe volver a activar el respaldo al cerrar desde Herramientas.",
            ));
        }
        Ok(Some(self.capture(&config.salt, &config.key).await?))
    }
    async fn capture(&self, salt: &[u8; 16], key: &[u8; 32]) -> Result<String, AppError> {
        let root = self.root()?;
        let stage = Stage::new(root)?;
        self.repository
            .snapshot(&self.pool, &stage.0.join("hrl_core_3fn.db"))
            .await?;
        copy_assets(&root.join("assets-3fn"), &stage.0.join("assets-3fn"))?;
        let validated = self.repository.validate(&stage.0).await?;
        let verification = async {
            let crypto = CryptoService::new(self.keys.clone());
            for credential in self.repository.credentials(validated.pool()).await? {
                crypto.decrypt(&credential)?;
            }
            Ok::<(), AppError>(())
        }
        .await;
        validated.close().await;
        verification?;
        let files = inventory(&stage.0)?;
        let folder = root.join("Respaldos");
        fs::create_dir_all(&folder)
            .map_err(|_| failure("No se pudo crear la carpeta Respaldos."))?;
        let name = format!(
            "HRL-{}-UTC-{}.hrlbackup",
            self.repository.timestamp(&self.pool).await?,
            u64::from_le_bytes(archive::random()?)
        );
        let output = folder.join(&name);
        let partial = output.with_extension("partial");
        let id = self.keys.active_key_id().to_owned();
        let secret = Zeroizing::new(*self.keys.key(&id)?);
        let result = archive::write_archive(&partial, &stage.0, &files, &id, &secret, salt, key)
            .and_then(|_| archive::verify_archive(&partial, key));
        if let Err(error) = result {
            let _ = fs::remove_file(&partial);
            return Err(error);
        }
        if fs::rename(&partial, &output).is_err() {
            let _ = fs::remove_file(&partial);
            return Err(failure("No se pudo confirmar el archivo de respaldo."));
        }
        Ok(name)
    }
    pub async fn restore(&self, path: &Path, input: &RestoreBackup) -> Result<(), AppError> {
        if input.current_master_password.len() > 1024
            || input.backup_master_password.len() > 1024
            || input.backup_master_password.chars().count() < 8
        {
            return Err(failure("Revisa la clave maestra del respaldo."));
        }
        let _job = self.gate.lock().await;
        let _gate = self.access.lock().await;
        self.authorize(&input.current_master_password, true).await?;
        let root = self.root()?;
        let parent = root.parent().ok_or(AppError::Crypto)?;
        let stage = Stage::new(parent)?;
        let restored = archive::read_archive(path, &stage.0, &input.backup_master_password)?;
        if restored.id != "windows-dpapi-v1" {
            return Err(failure(
                "La custodia de claves de este respaldo no es compatible.",
            ));
        }
        let db = self.repository.validate(&stage.0).await?;
        let verification = async {
            let master_hash = self.repository.master_hash(db.pool()).await?;
            if master_hash.len() > 1024 || !master_hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$") {
                return Err(failure("El formato de la clave maestra del respaldo no es compatible con esta versión."));
            }
            if !super::access::verify(
                &input.backup_master_password,
                &master_hash,
            )
            .await?
            {
                return Err(AppError::InvalidCredentials);
            }
            let crypto = CryptoService::new(Arc::new(ImportedKey {
                id: restored.id.clone(),
                value: Zeroizing::new(*restored.key),
            }));
            for credential in self.repository.credentials(db.pool()).await? {
                crypto.decrypt(&credential)?;
            }
            Ok::<(), AppError>(())
        }
        .await;
        db.close().await;
        verification?;
        fs::create_dir_all(stage.0.join("keys")).map_err(|_| AppError::Crypto)?;
        write_new(
            &stage.0.join("keys/device-password-v1.dpapi"),
            &key_store::protect(restored.key.as_ref(), false)?,
        )?;
        let previous = parent.join(format!("hrl-anterior-{}", unique()?));
        self.pool.close().await;
        if fs::rename(root, &previous).is_err() {
            return Err(failure(
                "No se pudo apartar la instalación actual. Se conserva; reinicia HRL CORE.",
            ));
        }
        if fs::rename(&stage.0, root).is_err() {
            let _ = fs::rename(&previous, root);
            return Err(failure(
                "No se pudo colocar el respaldo. Se conservó la copia anterior; reinicia HRL CORE.",
            ));
        }
        self.restored.store(true, Ordering::SeqCst);
        Ok(())
    }
    pub fn needs_restart(&self) -> bool {
        self.restored.load(Ordering::SeqCst)
    }
}
fn copy_assets(source: &Path, target: &Path) -> Result<(), AppError> {
    if !source.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(source).map_err(|_| AppError::Crypto)?;
    if archive::is_link(&metadata) {
        return Err(failure(
            "Los recursos contienen un enlace que no puede respaldarse.",
        ));
    }
    if metadata.is_file() {
        fs::copy(source, target)
            .map_err(|_| failure("No se pudo copiar un recurso del respaldo."))?;
        return Ok(());
    }
    fs::create_dir_all(target).map_err(|_| AppError::Crypto)?;
    for entry in fs::read_dir(source).map_err(|_| AppError::Crypto)? {
        let entry = entry.map_err(|_| AppError::Crypto)?;
        copy_assets(&entry.path(), &target.join(entry.file_name()))?;
    }
    Ok(())
}
fn inventory(root: &Path) -> Result<Vec<String>, AppError> {
    fn visit(root: &Path, path: &Path, files: &mut Vec<String>) -> Result<(), AppError> {
        for entry in fs::read_dir(path).map_err(|_| AppError::Crypto)? {
            let entry = entry.map_err(|_| AppError::Crypto)?;
            let metadata = fs::symlink_metadata(entry.path()).map_err(|_| AppError::Crypto)?;
            if archive::is_link(&metadata) {
                return Err(AppError::Crypto);
            }
            if metadata.is_dir() {
                visit(root, &entry.path(), files)?;
            } else if metadata.is_file() {
                let name = entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|_| AppError::Crypto)?
                    .to_string_lossy()
                    .replace('\\', "/");
                if name == "hrl_core_3fn.db" || name.starts_with("assets-3fn/") {
                    files.push(name);
                }
            }
        }
        Ok(())
    }
    let mut files = vec![];
    visit(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

pub fn choose_backup(owner: usize) -> Result<Option<PathBuf>, AppError> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::Controls::Dialogs::{
            CommDlgExtendedError, GetOpenFileNameW, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR,
            OFN_PATHMUSTEXIST, OPENFILENAMEW,
        };
        let filter: Vec<u16> = "Respaldos HRL CORE\0*.hrlbackup\0\0"
            .encode_utf16()
            .collect();
        let title: Vec<u16> = "Seleccionar respaldo de HRL CORE\0"
            .encode_utf16()
            .collect();
        let mut path = vec![0_u16; 32768];
        let mut dialog: OPENFILENAMEW = unsafe { std::mem::zeroed() };
        dialog.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        dialog.hwndOwner = owner as *mut std::ffi::c_void;
        dialog.lpstrFilter = filter.as_ptr();
        dialog.lpstrTitle = title.as_ptr();
        dialog.lpstrFile = path.as_mut_ptr();
        dialog.nMaxFile = path.len() as u32;
        dialog.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
        if unsafe { GetOpenFileNameW(&mut dialog) } == 0 {
            if unsafe { CommDlgExtendedError() } != 0 {
                return Err(failure("No se pudo abrir el selector de respaldos."));
            }
            return Ok(None);
        }
        let length = path.iter().position(|c| *c == 0).ok_or(AppError::Crypto)?;
        Ok(Some(PathBuf::from(
            String::from_utf16(&path[..length]).map_err(|_| AppError::Crypto)?,
        )))
    }
    #[cfg(not(windows))]
    {
        Err(failure(
            "La selección de respaldos está disponible en Windows.",
        ))
    }
}
