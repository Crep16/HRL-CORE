use super::{access::AccessService, audit::AuditService};
use crate::{
    domain::{company_contracts::*, contracts::AccessScope},
    error::AppError,
    repositories::{audit::AuditRecord, company::CompanyRepository},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::{io::Write, path::PathBuf, sync::Arc};

pub struct CompanyService {
    pool: SqlitePool,
    repository: Arc<CompanyRepository>,
    access: Arc<AccessService>,
    audit: Arc<AuditService>,
    assets: Option<PathBuf>,
}
fn invalid(text: &str) -> AppError {
    AppError::Validation(text.into())
}
impl CompanyService {
    pub fn new(
        pool: SqlitePool,
        repository: Arc<CompanyRepository>,
        access: Arc<AccessService>,
        audit: Arc<AuditService>,
        assets: Option<PathBuf>,
    ) -> Self {
        Self {
            pool,
            repository,
            access,
            audit,
            assets,
        }
    }
    fn logo_path(&self, name: &str) -> Result<PathBuf, AppError> {
        let valid = name
            .strip_suffix(".png")
            .or_else(|| name.strip_suffix(".jpg"))
            .is_some_and(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()));
        if !valid {
            return Err(invalid("La ruta del logo requiere revisión."));
        }
        Ok(self
            .assets
            .as_ref()
            .ok_or_else(|| invalid("El almacenamiento del logo no está disponible."))?
            .join("company")
            .join(name))
    }
    fn read_logo(&self, name: Option<&str>) -> Result<String, AppError> {
        let Some(name) = name else {
            return Ok(String::new());
        };
        let bytes = std::fs::read(self.logo_path(name)?)
            .map_err(|_| invalid("No se pudo recuperar el logo guardado."))?;
        let mime = if name.ends_with(".png") {
            "image/png"
        } else {
            "image/jpeg"
        };
        Ok(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
    }
    fn save_logo(&self, logo: &str) -> Result<Option<String>, AppError> {
        if logo.is_empty() {
            return Ok(None);
        }
        let (encoded, extension, signature) =
            if let Some(encoded) = logo.strip_prefix("data:image/png;base64,") {
                (encoded, "png", b"\x89PNG\r\n\x1a\n".as_slice())
            } else if let Some(encoded) = logo.strip_prefix("data:image/jpeg;base64,") {
                (encoded, "jpg", b"\xff\xd8\xff".as_slice())
            } else {
                return Err(invalid("El logo debe ser una imagen PNG o JPG."));
            };
        let bytes = STANDARD
            .decode(encoded)
            .map_err(|_| invalid("El logo no tiene una codificación válida."))?;
        if bytes.len() > 1024 * 1024 || !bytes.starts_with(signature) {
            return Err(invalid("El logo debe ser PNG o JPG de hasta 1 MB."));
        }
        let name = format!("{:x}.{extension}", Sha256::digest(&bytes));
        let path = self.logo_path(&name)?;
        std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|_| invalid("No se pudo preparar el almacenamiento del logo."))?;
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                if file
                    .write_all(&bytes)
                    .and_then(|_| file.sync_all())
                    .is_err()
                {
                    drop(file);
                    let _ = std::fs::remove_file(&path);
                    return Err(invalid("No se pudo guardar el logo completo."));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if std::fs::read(&path).map_err(|_| invalid("No se pudo comprobar el logo."))?
                    != bytes
                {
                    return Err(invalid("El archivo del logo requiere revisión."));
                }
            }
            Err(_) => return Err(invalid("No se pudo guardar el logo.")),
        }
        Ok(Some(name))
    }
    pub async fn load(&self) -> Result<Option<CompanySettings>, AppError> {
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        let mut allowed = false;
        for (operation, scope) in [
            ("company.view", AccessScope::Company),
            ("clients.view", AccessScope::Clients),
            ("repairs.view", AccessScope::Repairs),
            ("logbook.view", AccessScope::Logbook),
        ] {
            match self
                .access
                .require(&mut tx, operation, scope, None, false)
                .await
            {
                Ok(_) => {
                    allowed = true;
                    break;
                }
                Err(AppError::Forbidden) => {}
                Err(error) => return Err(error),
            }
        }
        if !allowed {
            return Err(AppError::Forbidden);
        }
        let result = self.repository.load(&mut tx).await?;
        tx.commit().await?;
        result
            .map(|(mut value, logo)| {
                value.logo = self.read_logo(logo.as_deref())?;
                Ok(value)
            })
            .transpose()
    }
    pub async fn save(&self, payload: SaveCompanyPayload) -> Result<CompanySettings, AppError> {
        let value = &payload.value;
        validate_contact(&value.email, &value.website)?;
        if value.name.trim().is_empty()
            || value.prefix.is_empty()
            || value.prefix.chars().count() > 12
            || !value
                .prefix
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            || !(1..=999999999).contains(&value.next_number)
            || !(1..=5).contains(&value.copies)
            || !(0..=3650).contains(&value.labor_days)
            || !(0..=3650).contains(&value.parts_days)
            || !(0..=100000).contains(&value.tax_rate_thousandths)
            || value.tax_name.trim().is_empty()
            || value.tax_name.chars().count() > 12
        {
            return Err(invalid(
                "Revisa los datos, importes y numeración de Empresa.",
            ));
        }
        if !["letter", "ticket", "ticket60"].contains(&value.paper.as_str())
            || !["portrait", "landscape"].contains(&value.orientation.as_str())
            || !["foot", "back"].contains(&value.clause_position.as_str())
            || !["es-MX", "en-US", "es-ES"].contains(&value.locale.as_str())
            || !["MXN", "USD", "EUR"].contains(&value.currency.as_str())
            || ![
                "America/Cancun",
                "America/Mexico_City",
                "America/Tijuana",
                "America/Hermosillo",
                "UTC",
            ]
            .contains(&value.timezone.as_str())
        {
            return Err(invalid("Selecciona las opciones disponibles de Empresa."));
        }
        if value.entry_footer.chars().count() > 200
            || value.exit_footer.chars().count() > 200
            || value.clauses.len() > 200
            || value.clauses.iter().any(|c| {
                c.title.trim().is_empty()
                    || c.title.chars().count() > 80
                    || c.text.trim().is_empty()
                    || c.text.chars().count() > 400
            })
        {
            return Err(invalid("Revisa los mensajes y cláusulas de las notas."));
        }
        let current_value = serde_json::to_value(value).map_err(|_| invalid("Revisa Empresa."))?;
        if current_value
            .as_object()
            .unwrap()
            .iter()
            .any(|(key, v)| key != "logo" && v.as_str().is_some_and(|s| s.len() > 16000))
            || value.logo.len() > 1_500_000
        {
            return Err(invalid("Un campo de Empresa supera el tamaño permitido."));
        }
        let _gate = self.access.lock().await;
        let mut tx = self.pool.begin().await?;
        let author = self
            .access
            .require(&mut tx, "company.edit", AccessScope::Company, None, true)
            .await?;
        let old = self.repository.load(&mut tx).await?;
        if old.as_ref().map_or(0, |(v, _)| v.revision) != payload.expected_revision {
            return Err(AppError::Conflict(
                "Empresa cambió. Vuelve a consultar antes de guardar.".into(),
            ));
        }
        let logo = self.save_logo(&value.logo)?;
        let old_value = old
            .as_ref()
            .map(|(v, _)| serde_json::to_value(v).unwrap())
            .unwrap_or(serde_json::Value::Null);
        let mut changes = Vec::<(String, Option<String>, Option<String>)>::new();
        for (key, new) in current_value.as_object().unwrap() {
            if ["revision", "logo", "clauses"].contains(&key.as_str()) {
                continue;
            }
            if old_value[key] != *new {
                changes.push((
                    key.clone(),
                    (!old_value[key].is_null()).then(|| scalar(&old_value[key])),
                    Some(scalar(new)),
                ))
            }
        }
        if old.as_ref().and_then(|(_, path)| path.as_ref()) != logo.as_ref() {
            changes.push((
                "logo".into(),
                old.as_ref()
                    .and_then(|(_, path)| path.as_ref())
                    .map(|_| "Imagen anterior".into()),
                logo.as_ref().map(|_| "Imagen actualizada".into()),
            ))
        }
        let old_clauses = old
            .as_ref()
            .map(|(v, _)| v.clauses.as_slice())
            .unwrap_or(&[]);
        for index in 0..old_clauses.len().max(value.clauses.len()) {
            for field in ["title", "text", "entry", "exit"] {
                let old = old_clauses
                    .get(index)
                    .map(|c| serde_json::to_value(c).unwrap()[field].clone());
                let new = value
                    .clauses
                    .get(index)
                    .map(|c| serde_json::to_value(c).unwrap()[field].clone());
                if old != new {
                    changes.push((
                        format!("clause.{}.{}", index + 1, field),
                        old.as_ref().map(scalar),
                        new.as_ref().map(scalar),
                    ))
                }
            }
        }
        self.repository
            .save(&mut tx, value, logo.as_deref())
            .await?;
        self.audit
            .record(
                &mut tx,
                AuditRecord {
                    author_id: author.id,
                    operation: "company.edit",
                    entity: "empresa",
                    entity_id: Some(1),
                    order_id: None,
                    reason: None,
                },
                &changes
                    .iter()
                    .map(|(k, o, n)| (k.as_str(), o.clone(), n.clone()))
                    .collect::<Vec<_>>(),
            )
            .await?;
        let (mut saved, _) = self
            .repository
            .load(&mut tx)
            .await?
            .ok_or(AppError::NotFound)?;
        tx.commit().await.map_err(AppError::Commit)?;
        saved.logo = value.logo.clone();
        Ok(saved)
    }
}
fn validate_contact(email: &str, website: &str) -> Result<(), AppError> {
    if !email.is_empty() {
        let valid = email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty()
                && !domain.contains('@')
                && domain.split_once('.').is_some_and(|(left, right)| {
                    !left.is_empty() && !right.is_empty()
                })
                && !email.chars().any(char::is_whitespace)
        });
        if !valid {
            return Err(invalid("Revisa el formato del correo."));
        }
    }
    if !website.is_empty() {
        let lower = website.to_ascii_lowercase();
        let address = if lower.starts_with("https://") || lower.starts_with("http://") {
            website.to_owned()
        } else {
            format!("https://{website}")
        };
        let valid = tauri::Url::parse(&address).is_ok_and(|url| {
            matches!(url.scheme(), "http" | "https")
                && url.host_str().is_some_and(|host| host.contains('.'))
        });
        if !valid {
            return Err(invalid("Escribe una página válida, por ejemplo negocio.mx."));
        }
    }
    Ok(())
}
fn scalar(value: &serde_json::Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
}
