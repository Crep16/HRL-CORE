use crate::{domain::portal::PublicTimestamp, error::AppError, repositories::portal::PortalConfig};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteError {
    Transient,
    Authentication,
    Denied,
    Invalid,
    Missing,
}
pub type RemoteFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, RemoteError>> + Send + 'a>>;
pub struct AuthTokens {
    pub id_token: Zeroizing<String>,
    pub refresh_token: Zeroizing<String>,
    pub uid: String,
    pub expires_in: u64,
}
pub struct RemoteDocument {
    pub revision: i64,
    pub fingerprint: String,
    pub updated_at: String,
}
pub trait FirebaseGateway: Send + Sync {
    fn login<'a>(
        &'a self,
        c: &'a PortalConfig,
        email: &'a str,
        password: &'a str,
    ) -> RemoteFuture<'a, AuthTokens>;
    fn refresh<'a>(&'a self, c: &'a PortalConfig, refresh: &'a str)
    -> RemoteFuture<'a, AuthTokens>;
    fn commit<'a>(
        &'a self,
        c: &'a PortalConfig,
        token: &'a str,
        document: &'a Value,
    ) -> RemoteFuture<'a, String>;
    fn read<'a>(
        &'a self,
        c: &'a PortalConfig,
        token: &'a str,
        code: &'a str,
    ) -> RemoteFuture<'a, RemoteDocument>;
}

pub struct RestFirebase {
    client: reqwest::Client,
}
impl RestFirebase {
    pub fn new() -> Result<Self, AppError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(8))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| {
                AppError::Validation("No se pudo iniciar la conexión con Firebase.".into())
            })?;
        Ok(Self { client })
    }
}

async fn response(request: reqwest::RequestBuilder) -> Result<Value, RemoteError> {
    let result = request.send().await.map_err(|_| RemoteError::Transient)?;
    let status = result.status().as_u16();
    if !(200..300).contains(&status) {
        return Err(match status {
            401 => RemoteError::Authentication,
            403 => RemoteError::Denied,
            404 => RemoteError::Missing,
            408 | 429 | 500..=599 => RemoteError::Transient,
            _ => RemoteError::Invalid,
        });
    }
    let bytes = result.bytes().await.map_err(|_| RemoteError::Transient)?;
    if bytes.len() > 2_000_000 {
        return Err(RemoteError::Invalid);
    }
    serde_json::from_slice(&bytes).map_err(|_| RemoteError::Transient)
}
fn tokens(value: Value, renewal: bool) -> Result<AuthTokens, RemoteError> {
    let key = |login, refresh| if renewal { refresh } else { login };
    let text = |name: &str| {
        value[name]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .ok_or(RemoteError::Invalid)
    };
    let expires = text(key("expiresIn", "expires_in"))?
        .parse::<u64>()
        .map_err(|_| RemoteError::Invalid)?;
    if !(60..=86400).contains(&expires) {
        return Err(RemoteError::Invalid);
    }
    Ok(AuthTokens {
        id_token: Zeroizing::new(text(key("idToken", "id_token"))?),
        refresh_token: Zeroizing::new(text(key("refreshToken", "refresh_token"))?),
        uid: text(key("localId", "user_id"))?,
        expires_in: expires,
    })
}
impl FirebaseGateway for RestFirebase {
    fn login<'a>(
        &'a self,
        c: &'a PortalConfig,
        email: &'a str,
        password: &'a str,
    ) -> RemoteFuture<'a, AuthTokens> {
        Box::pin(async move {
            let body = Zeroizing::new(
                serde_json::to_string(
                    &json!({"email":email,"password":password,"returnSecureToken":true}),
                )
                .map_err(|_| RemoteError::Invalid)?,
            );
            let r=response(self.client.post(format!("https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",c.api_key)).header("Content-Type","application/json").body(body.as_bytes().to_vec())).await?;
            tokens(r, false)
        })
    }
    fn refresh<'a>(
        &'a self,
        c: &'a PortalConfig,
        refresh: &'a str,
    ) -> RemoteFuture<'a, AuthTokens> {
        Box::pin(async move {
            let r = response(
                self.client
                    .post(format!(
                        "https://securetoken.googleapis.com/v1/token?key={}",
                        c.api_key
                    ))
                    .form(&[("grant_type", "refresh_token"), ("refresh_token", refresh)]),
            )
            .await
            .map_err(|e| {
                if e == RemoteError::Invalid {
                    RemoteError::Authentication
                } else {
                    e
                }
            })?;
            tokens(r, true)
        })
    }
    fn commit<'a>(
        &'a self,
        c: &'a PortalConfig,
        token: &'a str,
        document: &'a Value,
    ) -> RemoteFuture<'a, String> {
        Box::pin(async move {
            let body = commit_body(&c.project_id, document)?;
            let r=response(self.client.post(format!("https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents:commit",c.project_id)).bearer_auth(token).json(&body)).await?;
            let at = r
                .pointer("/writeResults/0/updateTime")
                .and_then(Value::as_str)
                .ok_or(RemoteError::Transient)?;
            chrono::DateTime::parse_from_rfc3339(at).map_err(|_| RemoteError::Transient)?;
            Ok(at.into())
        })
    }
    fn read<'a>(
        &'a self,
        c: &'a PortalConfig,
        token: &'a str,
        code: &'a str,
    ) -> RemoteFuture<'a, RemoteDocument> {
        Box::pin(async move {
            let r=response(self.client.get(format!("https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/seguimiento/{code}",c.project_id)).bearer_auth(token)).await?;
            let revision = r
                .pointer("/fields/revision/integerValue")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<i64>().ok())
                .filter(|v| *v > 0)
                .ok_or(RemoteError::Invalid)?;
            let fingerprint = r
                .pointer("/fields/huella/stringValue")
                .and_then(Value::as_str)
                .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
                .ok_or(RemoteError::Invalid)?
                .into();
            let updated_at = r["updateTime"].as_str().ok_or(RemoteError::Invalid)?.into();
            Ok(RemoteDocument {
                revision,
                fingerprint,
                updated_at,
            })
        })
    }
}

pub fn commit_body(project: &str, document: &Value) -> Result<Value, RemoteError> {
    let code = document["idPublico"].as_str().ok_or(RemoteError::Invalid)?;
    let object = document.as_object().ok_or(RemoteError::Invalid)?;
    let mut fields = serde_json::Map::new();
    for (key, value) in object {
        if key != "actualizadoEn" {
            fields.insert(key.clone(), firestore_value(key, value)?);
        }
    }
    Ok(
        json!({"writes":[{"update":{"name":format!("projects/{project}/databases/(default)/documents/seguimiento/{code}"),"fields":fields},"updateTransforms":[{"fieldPath":"actualizadoEn","setToServerValue":"REQUEST_TIME"}]}]}),
    )
}
fn firestore_value(key: &str, value: &Value) -> Result<Value, RemoteError> {
    Ok(match value {
        Value::Bool(v) => json!({"booleanValue":v}),
        Value::Number(v) => {
            json!({"integerValue":v.as_i64().ok_or(RemoteError::Invalid)?.to_string()})
        }
        Value::String(v) => {
            if [
                "recibidoEn",
                "entregaEstimada",
                "entregadoEn",
                "accesoHasta",
                "completadaEn",
                "fecha",
            ]
            .contains(&key)
            {
                PublicTimestamp::parse(v).map_err(|_| RemoteError::Invalid)?;
                json!({"timestampValue":v})
            } else {
                json!({"stringValue":v})
            }
        }
        Value::Array(v) => {
            json!({"arrayValue":{"values":v.iter().map(|i|firestore_value("",i)).collect::<Result<Vec<_>,_>>()?}})
        }
        Value::Object(v) => {
            let mut fields = serde_json::Map::new();
            for (k, v) in v {
                fields.insert(k.clone(), firestore_value(k, v)?);
            }
            json!({"mapValue":{"fields":fields}})
        }
        Value::Null => return Err(RemoteError::Invalid),
    })
}

#[derive(Serialize, Deserialize)]
pub struct StoredIdentity {
    pub project_id: String,
    pub uid: String,
    pub refresh_token: String,
}
impl Drop for StoredIdentity {
    fn drop(&mut self) {
        self.refresh_token.zeroize();
    }
}
pub trait TokenVault: Send + Sync {
    fn load(&self) -> Result<Option<StoredIdentity>, AppError>;
    fn save(&self, value: &StoredIdentity) -> Result<(), AppError>;
    fn clear(&self) -> Result<(), AppError>;
}
pub struct DpapiVault {
    path: Option<PathBuf>,
}
impl DpapiVault {
    pub fn new(root: Option<PathBuf>) -> Self {
        Self {
            path: root.map(|p| p.join("keys/portal-publicador.dpapi")),
        }
    }
}
impl TokenVault for DpapiVault {
    fn load(&self) -> Result<Option<StoredIdentity>, AppError> {
        let Some(path) = &self.path else {
            return Ok(None);
        };
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(path).map_err(|_| AppError::Crypto)?;
        if bytes.len() > 32768 {
            return Err(AppError::Crypto);
        }
        let clear = Zeroizing::new(super::key_store::protect(&bytes, true)?);
        let value = serde_json::from_slice(&clear).map_err(|_| AppError::Crypto)?;
        Ok(Some(value))
    }
    fn save(&self, value: &StoredIdentity) -> Result<(), AppError> {
        let path = self.path.as_ref().ok_or(AppError::Crypto)?;
        let directory = path.parent().ok_or(AppError::Crypto)?;
        std::fs::create_dir_all(directory).map_err(|_| AppError::Crypto)?;
        let clear = Zeroizing::new(serde_json::to_vec(value).map_err(|_| AppError::Crypto)?);
        let encrypted = super::key_store::protect(&clear, false)?;
        let temporary = path.with_extension("pending");
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temporary)
            .map_err(|_| AppError::Crypto)?;
        file.write_all(&encrypted)
            .and_then(|_| file.sync_all())
            .map_err(|_| AppError::Crypto)?;
        drop(file);
        std::fs::rename(&temporary, path).map_err(|_| AppError::Crypto)?;
        Ok(())
    }
    fn clear(&self) -> Result<(), AppError> {
        if let Some(path) = &self.path {
            if path.exists() {
                std::fs::remove_file(path).map_err(|_| AppError::Crypto)?;
            }
        }
        Ok(())
    }
}
pub struct Session {
    pub tokens: AuthTokens,
    pub expires_at: Instant,
}
pub struct PublisherRuntime {
    pub gateway: Arc<dyn FirebaseGateway>,
    pub vault: Arc<dyn TokenVault>,
    pub gate: Mutex<()>,
    pub session: Mutex<Option<Session>>,
    pub error: Mutex<Option<String>>,
    pub started: std::sync::atomic::AtomicBool,
    lock_path: Option<PathBuf>,
}
pub struct PublisherLease {
    _file: Option<std::fs::File>,
}
impl PublisherRuntime {
    pub fn new(root: Option<PathBuf>) -> Result<Self, AppError> {
        let lock_path = root.as_ref().map(|p| p.join("keys/portal-publicador.lock"));
        let mut runtime = Self::with(
            Arc::new(RestFirebase::new()?),
            Arc::new(DpapiVault::new(root)),
        );
        runtime.lock_path = lock_path;
        Ok(runtime)
    }
    pub fn with(gateway: Arc<dyn FirebaseGateway>, vault: Arc<dyn TokenVault>) -> Self {
        Self {
            gateway,
            vault,
            gate: Mutex::new(()),
            session: Mutex::new(None),
            error: Mutex::new(None),
            started: std::sync::atomic::AtomicBool::new(false),
            lock_path: None,
        }
    }
    pub fn try_lease(&self) -> Result<PublisherLease, AppError> {
        let Some(path) = &self.lock_path else {
            return Ok(PublisherLease { _file: None });
        };
        std::fs::create_dir_all(path.parent().ok_or(AppError::Crypto)?)
            .map_err(|_| AppError::Crypto)?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            let file=std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).share_mode(0).open(path)
                .map_err(|_|AppError::Conflict("Otra instancia está utilizando el publicador. Espera a que termine el envío.".into()))?;
            Ok(PublisherLease { _file: Some(file) })
        }
        #[cfg(not(windows))]
        {
            Err(AppError::Validation(
                "La custodia del publicador requiere Windows.".into(),
            ))
        }
    }
    pub async fn store(&self, c: &PortalConfig, tokens: AuthTokens) -> Result<(), AppError> {
        if tokens.uid != c.publisher_uid {
            return Err(AppError::Validation(
                "La cuenta no coincide con el UID autorizado para publicar.".into(),
            ));
        }
        self.vault.save(&StoredIdentity {
            project_id: c.project_id.clone(),
            uid: tokens.uid.clone(),
            refresh_token: tokens.refresh_token.to_string(),
        })?;
        let expires_at = Instant::now() + Duration::from_secs(tokens.expires_in.saturating_sub(30));
        *self.session.lock().await = Some(Session { tokens, expires_at });
        *self.error.lock().await = None;
        Ok(())
    }
    pub async fn token(
        &self,
        c: &PortalConfig,
        force: bool,
    ) -> Result<Zeroizing<String>, RemoteError> {
        if !force {
            if let Some(session) = self.session.lock().await.as_ref() {
                if session.expires_at > Instant::now() {
                    return Ok(Zeroizing::new(session.tokens.id_token.to_string()));
                }
            }
        }
        let identity = self
            .vault
            .load()
            .map_err(|_| RemoteError::Authentication)?
            .ok_or(RemoteError::Authentication)?;
        if identity.project_id != c.project_id || identity.uid != c.publisher_uid {
            return Err(RemoteError::Authentication);
        }
        let tokens = self.gateway.refresh(c, &identity.refresh_token).await?;
        let token = Zeroizing::new(tokens.id_token.to_string());
        self.store(c, tokens)
            .await
            .map_err(|_| RemoteError::Authentication)?;
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    fn publisher_lease_serializes_instances_and_is_released_when_the_handle_closes() {
        let directory = tempfile::tempdir().unwrap();
        let first = PublisherRuntime::new(Some(directory.path().to_path_buf())).unwrap();
        let second = PublisherRuntime::new(Some(directory.path().to_path_buf())).unwrap();
        let lease = first.try_lease().unwrap();
        assert!(second.try_lease().is_err());
        drop(lease);
        assert!(second.try_lease().is_ok());
    }
    #[cfg(windows)]
    #[test]
    fn dpapi_vault_keeps_only_the_encrypted_refresh_token_and_supports_rotation() {
        let directory = tempfile::tempdir().unwrap();
        let vault = DpapiVault::new(Some(directory.path().to_path_buf()));
        vault
            .save(&StoredIdentity {
                project_id: "test-project".into(),
                uid: "publisher".into(),
                refresh_token: "secret-refresh-token-1".into(),
            })
            .unwrap();
        let encrypted =
            std::fs::read(directory.path().join("keys/portal-publicador.dpapi")).unwrap();
        assert!(!encrypted.windows(14).any(|v| v == b"secret-refresh"));
        assert_eq!(
            vault.load().unwrap().unwrap().refresh_token,
            "secret-refresh-token-1"
        );
        vault
            .save(&StoredIdentity {
                project_id: "test-project".into(),
                uid: "publisher".into(),
                refresh_token: "secret-refresh-token-2".into(),
            })
            .unwrap();
        assert_eq!(
            vault.load().unwrap().unwrap().refresh_token,
            "secret-refresh-token-2"
        );
        vault.clear().unwrap();
        assert!(vault.load().unwrap().is_none());
    }
    #[test]
    fn rest_body_replaces_everything_and_uses_server_time_and_integer_strings() {
        let body=commit_body("hrl-web",&json!({"idPublico":"ABCD-EFGH-JKMN","revision":4,"publicado":true,"huella":"a".repeat(64),"equipo":{"tipo":"Laptop"},"pago":{"totalCentavos":60000},"eventos":[{"fecha":"2026-11-19T17:30:00.000Z"}]})).unwrap();
        let w = &body["writes"][0];
        assert!(w.get("updateMask").is_none());
        assert!(w["update"]["fields"].get("actualizadoEn").is_none());
        assert_eq!(w["update"]["fields"]["revision"]["integerValue"], "4");
        assert_eq!(w["updateTransforms"][0]["setToServerValue"], "REQUEST_TIME");
        assert_eq!(
            w.pointer(
                "/update/fields/eventos/arrayValue/values/0/mapValue/fields/fecha/timestampValue"
            )
            .unwrap(),
            "2026-11-19T17:30:00.000Z"
        );
        assert!(commit_body("hrl-web", &json!({"idPublico":"A","importe":1.5})).is_err());
    }
}
