use super::{fixture, order};
use crate::{
    domain::{contracts::EditOrderPayload, portal::PortalConfig},
    error::AppError,
    repositories::portal::PortalRepository,
    services::{portal_publisher::PublisherLogin, portal_transport::*},
};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

#[derive(Default)]
struct Vault(Mutex<Option<StoredIdentity>>);
impl TokenVault for Vault {
    fn load(&self) -> Result<Option<StoredIdentity>, AppError> {
        Ok(self.0.lock().unwrap().as_ref().map(|v| StoredIdentity {
            project_id: v.project_id.clone(),
            uid: v.uid.clone(),
            refresh_token: v.refresh_token.clone(),
        }))
    }
    fn save(&self, v: &StoredIdentity) -> Result<(), AppError> {
        *self.0.lock().unwrap() = Some(StoredIdentity {
            project_id: v.project_id.clone(),
            uid: v.uid.clone(),
            refresh_token: v.refresh_token.clone(),
        });
        Ok(())
    }
    fn clear(&self) -> Result<(), AppError> {
        *self.0.lock().unwrap() = None;
        Ok(())
    }
}
#[derive(Default)]
struct Scenario {
    remote: Option<Value>,
    commits: Vec<Value>,
    lose_response: bool,
    refresh_fails: bool,
    wrong_uid: bool,
}
#[derive(Default)]
struct Gateway(Mutex<Scenario>);
fn credentials(uid: &str) -> AuthTokens {
    AuthTokens {
        id_token: Zeroizing::new("test-id-token".into()),
        refresh_token: Zeroizing::new("test-refresh-token".into()),
        uid: uid.into(),
        expires_in: 3600,
    }
}
impl FirebaseGateway for Gateway {
    fn login<'a>(
        &'a self,
        _: &'a PortalConfig,
        _: &'a str,
        _: &'a str,
    ) -> RemoteFuture<'a, AuthTokens> {
        Box::pin(async move {
            Ok(credentials(if self.0.lock().unwrap().wrong_uid {
                "another-publisher"
            } else {
                "publisher"
            }))
        })
    }
    fn refresh<'a>(&'a self, _: &'a PortalConfig, _: &'a str) -> RemoteFuture<'a, AuthTokens> {
        Box::pin(async move {
            if self.0.lock().unwrap().refresh_fails {
                Err(RemoteError::Authentication)
            } else {
                Ok(credentials("publisher"))
            }
        })
    }
    fn commit<'a>(
        &'a self,
        _: &'a PortalConfig,
        _: &'a str,
        d: &'a Value,
    ) -> RemoteFuture<'a, String> {
        Box::pin(async move {
            let mut s = self.0.lock().unwrap();
            s.commits.push(d.clone());
            if s.remote
                .as_ref()
                .is_some_and(|r| r["revision"].as_i64() >= d["revision"].as_i64())
            {
                return Err(RemoteError::Denied);
            }
            s.remote = Some(d.clone());
            if s.lose_response {
                s.lose_response = false;
                return Err(RemoteError::Transient);
            }
            Ok("2026-11-19T17:30:00.123456Z".into())
        })
    }
    fn read<'a>(
        &'a self,
        _: &'a PortalConfig,
        _: &'a str,
        _: &'a str,
    ) -> RemoteFuture<'a, RemoteDocument> {
        Box::pin(async move {
            let s = self.0.lock().unwrap();
            let r = s.remote.as_ref().ok_or(RemoteError::Missing)?;
            Ok(RemoteDocument {
                revision: r["revision"].as_i64().unwrap(),
                fingerprint: r["huella"].as_str().unwrap().into(),
                updated_at: "2026-11-19T17:30:00.123456Z".into(),
            })
        })
    }
}
async fn connected() -> (
    sqlx::SqlitePool,
    crate::services::BackendServices,
    Arc<Gateway>,
    i64,
) {
    let (pool, mut services) = fixture().await;
    let gateway = Arc::new(Gateway::default());
    Arc::get_mut(&mut services.portal).unwrap().runtime =
        PublisherRuntime::with(gateway.clone(), Arc::new(Vault::default()));
    let id = services.orders.create(order()).await.unwrap().id;
    services
        .portal
        .configure(PortalConfig {
            project_id: "hrl-web".into(),
            api_key: "test-key".into(),
            publisher_uid: "publisher".into(),
            portal_url: "https://portal.example.invalid".into(),
            paused: true,
        })
        .await
        .unwrap();
    services
        .portal
        .connect(PublisherLogin {
            email: "test@example.invalid".into(),
            password: "test-only-password".into(),
        })
        .await
        .unwrap();
    (pool, services, gateway, id)
}

#[test]
fn portal_sender_allows_correcting_configuration_before_connecting_but_blocks_unplanned_transfers()
{
    tauri::async_runtime::block_on(async {
        let (_pool, mut services) = fixture().await;
        Arc::get_mut(&mut services.portal).unwrap().runtime =
            PublisherRuntime::with(Arc::new(Gateway::default()), Arc::new(Vault::default()));
        let mut config = PortalConfig {
            project_id: "hrl-typo".into(),
            api_key: "test-key".into(),
            publisher_uid: "mistyped".into(),
            portal_url: String::new(),
            paused: true,
        };
        services.portal.configure(config.clone()).await.unwrap();
        config.project_id = "hrl-web".into();
        config.publisher_uid = "publisher".into();
        services.portal.configure(config.clone()).await.unwrap();
        services
            .portal
            .connect(PublisherLogin {
                email: "test@example.invalid".into(),
                password: "test-only-password".into(),
            })
            .await
            .unwrap();
        config.project_id = "another-project".into();
        assert!(services.portal.configure(config).await.is_err());
    });
}

#[test]
fn portal_sender_confirms_a_lost_response_by_revision_and_fingerprint() {
    tauri::async_runtime::block_on(async {
        let (pool, services, gateway, id) = connected().await;
        assert!(!services.portal.pump().await.unwrap());
        assert!(gateway.0.lock().unwrap().commits.is_empty());
        services.portal.resume().await.unwrap();
        gateway.0.lock().unwrap().lose_response = true;
        assert!(services.portal.pump().await.unwrap());
        assert_eq!(
            services.portal.status(id).await.unwrap()["state"],
            "pending"
        );
        sqlx::query("UPDATE portal_envios SET siguiente_intento=0")
            .execute(&pool)
            .await
            .unwrap();
        assert!(services.portal.pump().await.unwrap());
        assert_eq!(
            services.portal.status(id).await.unwrap()["state"],
            "confirmed"
        );
        let calls = &gateway.0.lock().unwrap().commits;
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], calls[1]);
        assert!(
            !serde_json::to_string(calls)
                .unwrap()
                .contains("test-device-PIN")
        );
    });
}

#[test]
fn portal_sender_keeps_conflicts_paused_until_explicit_order_reconciliation() {
    tauri::async_runtime::block_on(async {
        let (pool, services, gateway, id) = connected().await;
        let snapshot: i64 = sqlx::query_scalar("SELECT max(id) FROM portal_publicaciones")
            .fetch_one(&pool)
            .await
            .unwrap();
        let mut remote = PortalRepository
            .load(&mut *pool.acquire().await.unwrap(), snapshot)
            .await
            .unwrap()
            .document;
        remote["huella"] = Value::String("f".repeat(64));
        gateway.0.lock().unwrap().remote = Some(remote);
        assert!(services.portal.resume().await.is_err());
        assert_eq!(
            services.portal.status(id).await.unwrap()["state"],
            "conflict"
        );
        assert_eq!(
            services.portal.settings().await.unwrap()["config"]["paused"],
            true
        );
        services.portal.reconcile_order(id).await.unwrap();
        services.portal.resume().await.unwrap();
        services.portal.pump().await.unwrap();
        assert_eq!(
            services.portal.status(id).await.unwrap()["state"],
            "confirmed"
        );
    });
}

#[test]
fn portal_sender_pauses_on_revoked_authentication_without_losing_local_changes() {
    tauri::async_runtime::block_on(async {
        let (_pool, services, gateway, id) = connected().await;
        services.portal.resume().await.unwrap();
        gateway.0.lock().unwrap().refresh_fails = true;
        *services.portal.runtime.session.lock().await = None;
        services.portal.pump().await.unwrap();
        assert_eq!(
            services.portal.settings().await.unwrap()["config"]["paused"],
            true
        );
        assert_eq!(
            services.portal.status(id).await.unwrap()["state"],
            "pending"
        );
        assert!(gateway.0.lock().unwrap().commits.is_empty());
        let old = services
            .orders
            .get(crate::domain::contracts::GetOrderPayload {
                order_id: id,
                scope: crate::domain::contracts::AccessScope::Clients,
                include_deleted: false,
                include_device_password: false,
            })
            .await
            .unwrap();
        let mut changed = order();
        changed.status = "repair".into();
        services
            .orders
            .edit(EditOrderPayload {
                order_id: id,
                revision: old.revision,
                value: changed,
                delivered: false,
                change_password: false,
            })
            .await
            .unwrap();
        assert!(gateway.0.lock().unwrap().commits.is_empty());
    });
}

#[test]
fn portal_sender_recovers_when_remote_commit_succeeds_and_local_audit_fails() {
    tauri::async_runtime::block_on(async {
        let (pool, services, gateway, id) = connected().await;
        services.portal.resume().await.unwrap();
        sqlx::raw_sql("CREATE TRIGGER reject_send_confirmation BEFORE INSERT ON auditoria WHEN NEW.entidad='portal_envios' AND EXISTS(SELECT 1 FROM portal_envios WHERE id_publicacion=CAST(NEW.id_entidad AS INTEGER) AND estado='confirmed') BEGIN SELECT RAISE(ABORT,'prueba'); END;").execute(&pool).await.unwrap();
        assert!(services.portal.pump().await.is_err());
        assert!(gateway.0.lock().unwrap().remote.is_some());
        assert_eq!(
            services.portal.status(id).await.unwrap()["state"],
            "sending"
        );
        sqlx::raw_sql("DROP TRIGGER reject_send_confirmation")
            .execute(&pool)
            .await
            .unwrap();
        services.portal.pump().await.unwrap();
        assert_eq!(
            services.portal.status(id).await.unwrap()["state"],
            "confirmed"
        );
    });
}

#[test]
fn portal_sender_delay_is_bounded_and_a_different_uid_is_rejected() {
    tauri::async_runtime::block_on(async {
        use crate::services::portal_publisher::retry_delay;
        assert_eq!(retry_delay(1, 0), 2);
        assert_eq!(retry_delay(2, 0), 4);
        assert_eq!(retry_delay(100, 12345), 300);
        let (_pool, services, gateway, _id) = connected().await;
        services.portal.disconnect().await.unwrap();
        gateway.0.lock().unwrap().wrong_uid = true;
        assert!(
            services
                .portal
                .connect(PublisherLogin {
                    email: "test@example.invalid".into(),
                    password: "test-only-password".into()
                })
                .await
                .is_err()
        );
        assert_eq!(
            services.portal.settings().await.unwrap()["connected"],
            false
        );
    });
}

#[test]
fn portal_status_does_not_confirm_invalid_content_or_reuse_an_old_code_confirmation() {
    tauri::async_runtime::block_on(async {
        let (_pool, services, _gateway, id) = connected().await;
        services.portal.resume().await.unwrap();
        services.portal.pump().await.unwrap();
        let confirmed = services.portal.status(id).await.unwrap();
        assert_eq!(confirmed["state"], "confirmed");
        assert!(confirmed["confirmedAt"].is_string());

        let mut value = order();
        value.equipment.model = "x".repeat(101);
        services
            .orders
            .edit(EditOrderPayload {
                order_id: id,
                revision: 1,
                value,
                delivered: false,
                change_password: false,
            })
            .await
            .unwrap();
        let invalid = services.portal.status(id).await.unwrap();
        assert_eq!(invalid["state"], "error");
        assert!(invalid["error"].is_string());
        assert!(invalid["confirmedAt"].is_null());

        let rotated = services.portal.action(id, "rotate").await.unwrap();
        assert_ne!(rotated["code"], confirmed["code"]);
        services.portal.pump().await.unwrap();
        let current = services.portal.status(id).await.unwrap();
        assert_eq!(current["code"], rotated["code"]);
        assert_eq!(current["state"], "error");
        assert!(current["confirmedAt"].is_null());

        services
            .orders
            .edit(EditOrderPayload {
                order_id: id,
                revision: 2,
                value: order(),
                delivered: false,
                change_password: false,
            })
            .await
            .unwrap();
        let pending = services.portal.status(id).await.unwrap();
        assert_eq!(pending["state"], "pending");
        assert!(pending["error"].is_null());
        assert!(pending["confirmedAt"].is_null());
        services.portal.pump().await.unwrap();
        assert_eq!(
            services.portal.status(id).await.unwrap()["state"],
            "confirmed"
        );
    });
}

#[test]
fn portal_status_only_exposes_the_confirmation_of_the_latest_publication() {
    tauri::async_runtime::block_on(async {
        let (_pool, services, _gateway, id) = connected().await;
        services.portal.resume().await.unwrap();
        services.portal.pump().await.unwrap();
        let withdrawn = services.portal.action(id, "withdraw").await.unwrap();
        assert_eq!(withdrawn["state"], "pending");
        assert!(withdrawn["confirmedAt"].is_null());
        services.portal.pump().await.unwrap();
        let confirmed = services.portal.status(id).await.unwrap();
        assert_eq!(confirmed["state"], "confirmed");
        assert_eq!(confirmed["enabled"], false);
        assert!(confirmed["confirmedAt"].is_string());
    });
}

#[test]
fn portal_publishes_local_repair_events_as_utc_without_rewriting_the_history() {
    tauri::async_runtime::block_on(async {
        use crate::domain::{
            company_contracts::SaveCompanyPayload,
            repair_contracts::{Event, SaveRepairPayload},
        };
        let (pool, services, gateway, id) = connected().await;
        let mut snapshot = services.repairs.load(id).await.unwrap();
        snapshot.repair.events.push(Event {
            id: 1,
            date: "2026-11-19T17:30".into(),
            title: "Avance local".into(),
            detail: "Diagnóstico completado".into(),
            stage: snapshot.repair.stages[0].clone(),
            public: true,
        });
        services
            .repairs
            .save(SaveRepairPayload {
                order_id: id,
                revision: snapshot.revision,
                repair: snapshot.repair,
            })
            .await
            .unwrap();
        assert!(services.portal.status(id).await.unwrap()["error"].is_null());
        services.portal.resume().await.unwrap();
        services.portal.pump().await.unwrap();
        assert_eq!(
            gateway.0.lock().unwrap().remote.as_ref().unwrap()["eventos"][0]["fecha"],
            "2026-11-19T22:30:00.000Z"
        );
        assert_eq!(
            services.repairs.load(id).await.unwrap().repair.events[0].date,
            "2026-11-19T17:30"
        );

        let publication: i64 = sqlx::query_scalar("SELECT max(id) FROM portal_publicaciones")
            .fetch_one(&pool)
            .await
            .unwrap();
        let before = PortalRepository
            .load(&mut *pool.acquire().await.unwrap(), publication)
            .await
            .unwrap()
            .document;
        let mut company = super::company_settings();
        company.timezone = "America/Tijuana".into();
        services
            .company
            .save(SaveCompanyPayload {
                value: company,
                expected_revision: 0,
            })
            .await
            .unwrap();
        let after = PortalRepository
            .load(&mut *pool.acquire().await.unwrap(), publication)
            .await
            .unwrap()
            .document;
        assert_eq!(before, after);
    });
}
