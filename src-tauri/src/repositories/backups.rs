use crate::{db::Database, error::AppError};
use sqlx::{Row, SqlitePool};
use std::path::Path;

pub struct BackupRepository;
impl BackupRepository {
    pub async fn timestamp(&self, pool: &SqlitePool) -> Result<String, AppError> {
        Ok(sqlx::query_scalar("SELECT strftime('%Y%m%d-%H%M%f','now')")
            .fetch_one(pool)
            .await?)
    }
    pub async fn master_hash(&self, pool: &SqlitePool) -> Result<String, AppError> {
        Ok(
            sqlx::query_scalar("SELECT hash FROM claves_maestras WHERE id=1")
                .fetch_one(pool)
                .await?,
        )
    }
    pub async fn configured(&self, pool: &SqlitePool) -> Result<bool, AppError> {
        Ok(
            sqlx::query_scalar("SELECT configurado FROM vista_configuracion_acceso")
                .fetch_one(pool)
                .await?,
        )
    }
    pub async fn snapshot(&self, pool: &SqlitePool, path: &Path) -> Result<(), AppError> {
        sqlx::query("VACUUM INTO ?")
            .bind(path.to_string_lossy().as_ref())
            .execute(pool)
            .await?;
        Ok(())
    }
    pub async fn validate(&self, root: &Path) -> Result<Database, AppError> {
        let path = root.join("hrl_core_3fn.db");
        if !path.is_file() {
            return Err(AppError::Validation("Falta la base del respaldo.".into()));
        }
        let db = Database::open_file(&path).await?;
        let result = async {
            let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
                .fetch_one(db.pool())
                .await?;
            if integrity != "ok"
                || !sqlx::query("PRAGMA foreign_key_check")
                    .fetch_all(db.pool())
                    .await?
                    .is_empty()
            {
                return Err(AppError::Validation(
                    "La base del respaldo no supera la comprobación de integridad.".into(),
                ));
            }
            if !self.configured(db.pool()).await? {
                return Err(AppError::Validation(
                    "El respaldo no tiene un administrador configurado.".into(),
                ));
            }
            let mut paths: Vec<(String, Option<i64>)> =
                sqlx::query("SELECT ruta_relativa,tamano_bytes FROM evidencias")
                    .fetch_all(db.pool())
                    .await?
                    .iter()
                    .map(|r| (r.get(0), Some(r.get(1))))
                    .collect();
            let logo: Option<String> =
                sqlx::query_scalar("SELECT logo_ruta FROM configuracion_documentos WHERE id=1")
                    .fetch_optional(db.pool())
                    .await?
                    .flatten();
            if let Some(path) = logo.filter(|p| !p.is_empty()) {
                paths.push((format!("company/{path}"), None));
            }
            for (path, size) in paths {
                let name = format!("assets-3fn/{path}");
                if !crate::domain::backup::safe_path(&name) {
                    return Err(AppError::Validation(
                        "Hay una ruta de recurso inválida en el respaldo.".into(),
                    ));
                }
                let metadata = std::fs::symlink_metadata(root.join(name)).map_err(|_| {
                    AppError::Validation("Faltan fotografías o el logo del respaldo.".into())
                })?;
                #[cfg(windows)]
                let linked = {
                    use std::os::windows::fs::MetadataExt;
                    metadata.file_attributes() & 0x400 != 0
                };
                #[cfg(not(windows))]
                let linked = metadata.file_type().is_symlink();
                if !metadata.is_file()
                    || linked
                    || size.is_some_and(|n| n < 0 || n as u64 != metadata.len())
                {
                    return Err(AppError::Validation(
                        "Un recurso del respaldo está incompleto.".into(),
                    ));
                }
            }
            Ok::<(), AppError>(())
        }
        .await;
        if let Err(error) = result {
            db.close().await;
            return Err(error);
        }
        Ok(db)
    }
    pub async fn credentials(
        &self,
        pool: &SqlitePool,
    ) -> Result<Vec<crate::domain::security::DeviceCredential>, AppError> {
        sqlx::query("SELECT id_orden,id_clave,nonce,cifrado FROM credenciales_dispositivos")
            .fetch_all(pool)
            .await?
            .iter()
            .map(|r| {
                let nonce: Vec<u8> = r.try_get("nonce")?;
                crate::domain::security::DeviceCredential::from_encrypted(
                    r.try_get("id_orden")?,
                    r.try_get("id_clave")?,
                    nonce.as_slice().try_into().map_err(|_| AppError::Crypto)?,
                    r.try_get("cifrado")?,
                )
                .map_err(|_| AppError::Crypto)
            })
            .collect()
    }
}
