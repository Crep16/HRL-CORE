use super::{
    BackendServices,
    crypto::{CryptoService, KeyProvider},
};
use crate::{
    db::Database,
    domain::{contracts::*, security::DeviceCredential},
    error::AppError,
};
use sqlx::SqlitePool;
use std::sync::Arc;

struct TestKeys([u8; 32]);
impl KeyProvider for TestKeys {
    fn active_key_id(&self) -> &str {
        "test-only"
    }
    fn key(&self, id: &str) -> Result<&[u8; 32], AppError> {
        if id != "test-only" {
            return Err(AppError::Crypto);
        }
        Ok(&self.0)
    }
}
fn keys() -> Arc<dyn KeyProvider> {
    Arc::new(TestKeys([7; 32]))
}

#[test]
fn portable_backups_restore_books_assets_and_device_key_without_the_original_installation() {
    tauri::async_runtime::block_on(async {
        use crate::domain::{backup::*, evidence_contracts::NewEvidence};
        let workspace = tempfile::tempdir().unwrap();
        let source = workspace.path().join("origen");
        std::fs::create_dir(&source).unwrap();
        let db = Database::open_file(&source.join("hrl_core_3fn.db"))
            .await
            .unwrap();
        let pool = db.pool().clone();
        let services = BackendServices::with_os_key_store(db, &source.join("keys"))
            .await
            .unwrap();
        services.access.initialize(setup()).await.unwrap();
        let mut payload = order();
        let mut company = company_settings();
        company.logo="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aZ1sAAAAASUVORK5CYII=".into();
        company.logo_name = "logo-de-prueba.png".into();
        services
            .company
            .save(crate::domain::company_contracts::SaveCompanyPayload {
                value: company,
                expected_revision: 0,
            })
            .await
            .unwrap();
        payload.initial_payment = Some(InitialPaymentInput {
            request_id: "backup-receipt-test-001".into(),
            amount_minor: 25000,
            method: "Efectivo".into(),
        });
        let created = services.orders.create(payload).await.unwrap();
        let code = "11223344556677889900aabbccddeeff";
        services
            .evidence
            .add(NewEvidence {
                id: code.into(),
                order_id: created.id,
                original_name: "foto.png".into(),
                mime: "image/png".into(),
                source: "file".into(),
                note: "Prueba de respaldo".into(),
                bytes: b"\x89PNG\r\n\x1a\nprueba".to_vec(),
            })
            .await
            .unwrap();
        let auth = BackupAuth {
            master_password: "master-test-123".into(),
        };
        assert!(!services.backups.status(&auth).await.unwrap().automatic);
        assert!(
            services
                .backups
                .create(&BackupAuth {
                    master_password: "incorrecta".into()
                })
                .await
                .is_err()
        );
        let manual = services.backups.create(&auth).await.unwrap();
        services
            .backups
            .configure(&BackupConfiguration {
                master_password: "master-test-123".into(),
                automatic: true,
            })
            .await
            .unwrap();
        assert!(services.backups.status(&auth).await.unwrap().automatic);
        assert!(services.backups.automatic().await.unwrap().is_some());
        services
            .backups
            .configure(&BackupConfiguration {
                master_password: "master-test-123".into(),
                automatic: false,
            })
            .await
            .unwrap();
        assert!(!services.backups.status(&auth).await.unwrap().automatic);
        assert!(services.backups.automatic().await.unwrap().is_none());
        services
            .backups
            .configure(&BackupConfiguration {
                master_password: "master-test-123".into(),
                automatic: true,
            })
            .await
            .unwrap();
        assert_eq!(services.backups.status(&auth).await.unwrap().files.len(), 2);
        std::fs::write(
            source.join("keys/respaldo-automatico.dpapi"),
            b"configuracion-danada",
        )
        .unwrap();
        let damaged_status = services.backups.status(&auth).await.unwrap();
        assert!(!damaged_status.automatic);
        assert!(damaged_status.warning.is_some());
        assert!(services.backups.automatic().await.is_err());
        services
            .backups
            .configure(&BackupConfiguration {
                master_password: "master-test-123".into(),
                automatic: true,
            })
            .await
            .unwrap();
        let repaired_status = services.backups.status(&auth).await.unwrap();
        assert!(repaired_status.automatic);
        assert!(repaired_status.warning.is_none());
        let limited = add_user(&pool, "backup-limited", "limited").await;
        services
            .access
            .login(login("backup-limited"))
            .await
            .unwrap();
        assert!(matches!(
            services.backups.status(&auth).await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            services.backups.create(&auth).await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            services
                .backups
                .configure(&BackupConfiguration {
                    master_password: "master-test-123".into(),
                    automatic: false
                })
                .await,
            Err(AppError::Forbidden)
        ));
        assert!(limited > 0);
        assert!(matches!(
            services
                .backups
                .authorize_restore(&RestoreBackup {
                    current_master_password: "master-test-123".into(),
                    backup_master_password: "master-test-123".into()
                })
                .await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            services
                .backups
                .restore(
                    &workspace.path().join("inexistente.hrlbackup"),
                    &RestoreBackup {
                        current_master_password: "master-test-123".into(),
                        backup_master_password: "master-test-123".into()
                    }
                )
                .await,
            Err(AppError::Forbidden)
        ));
        add_user(&pool, "backup-custom", "custom").await;
        services.access.login(login("backup-custom")).await.unwrap();
        assert!(matches!(
            services.backups.status(&auth).await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            services
                .backups
                .authorize_restore(&RestoreBackup {
                    current_master_password: "master-test-123".into(),
                    backup_master_password: "master-test-123".into()
                })
                .await,
            Err(AppError::Forbidden)
        ));
        services.access.login(login("admin")).await.unwrap();
        let usb = workspace.path().join("usb.hrlbackup");
        std::fs::copy(source.join("Respaldos").join(manual), &usb).unwrap();
        let encrypted = std::fs::read(&usb).unwrap();
        assert!(
            !encrypted
                .windows(b"test-device-PIN-2048".len())
                .any(|part| part == b"test-device-PIN-2048")
        );
        assert!(
            !encrypted
                .windows(b"master-test-123".len())
                .any(|part| part == b"master-test-123")
        );
        let relative: String = sqlx::query_scalar("SELECT ruta_relativa FROM evidencias LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        let photo = source.join("assets-3fn").join(relative);
        assert!(photo.starts_with(&source));
        std::fs::remove_file(photo).unwrap();
        assert!(services.backups.create(&auth).await.is_err());
        assert_eq!(services.backups.status(&auth).await.unwrap().files.len(), 2);
        let original_hash: String =
            sqlx::query_scalar("SELECT hash FROM claves_maestras WHERE id=1")
                .fetch_one(&pool)
                .await
                .unwrap();
        sqlx::query("UPDATE claves_maestras SET hash='changed-for-test' WHERE id=1")
            .execute(&pool)
            .await
            .unwrap();
        assert!(services.backups.automatic().await.is_err());
        sqlx::query("UPDATE claves_maestras SET hash=? WHERE id=1")
            .bind(original_hash)
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;
        drop(services);
        assert!(
            source
                .canonicalize()
                .unwrap()
                .starts_with(workspace.path().canonicalize().unwrap())
        );
        std::fs::remove_dir_all(&source).unwrap();

        let target = workspace.path().join("destino");
        std::fs::create_dir(&target).unwrap();
        let db = Database::open_file(&target.join("hrl_core_3fn.db"))
            .await
            .unwrap();
        let target_pool = db.pool().clone();
        let destination = BackendServices::with_os_key_store(db, &target.join("keys"))
            .await
            .unwrap();
        let original_key = std::fs::read(target.join("keys/device-password-v1.dpapi")).unwrap();
        assert!(
            destination
                .backups
                .restore(
                    &usb,
                    &RestoreBackup {
                        current_master_password: "".into(),
                        backup_master_password: "incorrecta".into()
                    }
                )
                .await
                .is_err()
        );
        assert_eq!(count(&target_pool, "usuarios").await, 0);
        let corrupt = workspace.path().join("corrupto.hrlbackup");
        let mut changed = encrypted.clone();
        *changed.last_mut().unwrap() ^= 1;
        std::fs::write(&corrupt, changed).unwrap();
        assert!(
            destination
                .backups
                .restore(
                    &corrupt,
                    &RestoreBackup {
                        current_master_password: "".into(),
                        backup_master_password: "master-test-123".into()
                    }
                )
                .await
                .is_err()
        );
        assert_eq!(count(&target_pool, "usuarios").await, 0);
        assert_eq!(
            std::fs::read(target.join("keys/device-password-v1.dpapi")).unwrap(),
            original_key
        );
        destination
            .backups
            .restore(
                &usb,
                &RestoreBackup {
                    current_master_password: "".into(),
                    backup_master_password: "master-test-123".into(),
                },
            )
            .await
            .unwrap();
        assert!(destination.backups.needs_restart());
        drop(destination);
        let db = Database::open_file(&target.join("hrl_core_3fn.db"))
            .await
            .unwrap();
        let restored_pool = db.pool().clone();
        let restored = BackendServices::with_os_key_store(db, &target.join("keys"))
            .await
            .unwrap();
        restored.access.login(login("admin")).await.unwrap();
        let company = restored.company.load().await.unwrap().unwrap();
        assert_eq!(company.logo_name, "logo-de-prueba.png");
        assert!(company.logo.starts_with("data:image/png;base64,"));
        let read = restored
            .orders
            .get(GetOrderPayload {
                order_id: created.id,
                scope: AccessScope::Clients,
                include_device_password: true,
                include_deleted: false,
            })
            .await
            .unwrap();
        assert_eq!(read.net_minor, 25000);
        assert_eq!(read.pending_minor, 125000);
        assert_eq!(
            read.device_password.unwrap().0.as_str(),
            "test-device-PIN-2048"
        );
        assert_eq!(
            restored.evidence.read(created.id, code).await.unwrap(),
            b"\x89PNG\r\n\x1a\nprueba"
        );
        assert_eq!(count(&restored_pool, "abonos").await, 1);
        assert!(count(&restored_pool, "auditoria").await >= 3);
    });
}

fn company_settings() -> crate::domain::company_contracts::CompanySettings {
    serde_json::from_value(serde_json::json!({"name":"Taller de prueba","taxId":"","phone":"0000000000","whatsapp":"","email":"","website":"","address":"","hours":"","country":"México","locale":"es-MX","timezone":"America/Cancun","currency":"MXN","showTax":true,"taxRateThousandths":16000,"taxName":"IVA","laborDays":30,"partsDays":90,"logoName":"","logoPrint":true,"logoWeb":false,"entryFooter":"Entrada","exitFooter":"Salida","printAddress":true,"signature":true,"webReference":false,"clausePosition":"foot","numberClauses":true,"paper":"letter","orientation":"portrait","copies":2,"prefix":"PR-","logo":"","nextNumber":20,"clauses":[{"id":"new","title":"Condición de prueba","text":"Texto de prueba","entry":true,"exit":false}]})).unwrap()
}

#[test]
fn company_contact_validation_matches_optional_form_fields_and_rejects_without_writes() {
    tauri::async_runtime::block_on(async {
        use crate::domain::company_contracts::SaveCompanyPayload;
        let (pool, services) = fixture().await;
        let mut revision = 0;
        for (email, website) in [
            ("", ""),
            ("taller@example.mx", "negocio.mx"),
            ("taller+servicio@example.mx", "HTTPS://negocio.mx/contacto"),
            ("", "http://negocio.mx:8080"),
        ] {
            let mut value = company_settings();
            value.email = email.into();
            value.website = website.into();
            let saved = services
                .company
                .save(SaveCompanyPayload {
                    value,
                    expected_revision: revision,
                })
                .await
                .unwrap();
            revision = saved.revision;
            assert_eq!(saved.email, email);
            assert_eq!(saved.website, website);
        }
        let before = serde_json::to_value(services.company.load().await.unwrap()).unwrap();
        let audits = count(&pool, "auditoria").await;
        let details = count(&pool, "auditoria_detalle").await;
        for (email, website) in [
            ("sin-arroba.mx", ""),
            ("a@@negocio.mx", ""),
            ("a@dominio", ""),
            ("a b@negocio.mx", ""),
            (" a@negocio.mx", ""),
            ("", "localhost"),
            ("", "https://"),
            ("", "https://negocio .mx"),
            ("", "http://negocio.mx:abc"),
        ] {
            let mut value = company_settings();
            value.email = email.into();
            value.website = website.into();
            assert!(matches!(
                services
                    .company
                    .save(SaveCompanyPayload {
                        value,
                        expected_revision: revision,
                    })
                    .await,
                Err(AppError::Validation(_))
            ));
            assert_eq!(
                serde_json::to_value(services.company.load().await.unwrap()).unwrap(),
                before
            );
            assert_eq!(count(&pool, "auditoria").await, audits);
            assert_eq!(count(&pool, "auditoria_detalle").await, details);
        }
    });
}

#[test]
fn order_optional_changes_keep_old_and_new_values_and_audit_identity_without_secrets() {
    tauri::async_runtime::block_on(async {
        use crate::domain::report_contracts::LogbookFilters;
        let (_, services) = fixture().await;
        let created = services.orders.create(order()).await.unwrap();
        let mut revision = 1;
        for (email, delivery) in [
            (Some("nuevo@example.mx"), Some("2026-11-20")),
            (None, None),
            (Some("ultimo@example.mx"), Some("2026-11-25")),
        ] {
            let previous = services.orders.get(get(created.id, false)).await.unwrap();
            let mut value = order();
            value.client.email = email.map(str::to_owned);
            value.estimated_delivery_at = delivery.map(str::to_owned);
            value.device_password = Some("edited-secret-9347".into());
            let saved = services
                .orders
                .edit(EditOrderPayload {
                    order_id: created.id,
                    revision,
                    value,
                    delivered: false,
                    change_password: true,
                })
                .await
                .unwrap();
            revision = saved.revision;
            let page = services
                .reports
                .logbook(LogbookFilters {
                    query: String::new(),
                    operation: "orders.edit".into(),
                    author_id: String::new(),
                    page: 0,
                    order_id: created.folio.clone(),
                    from_utc: String::new(),
                    to_utc: String::new(),
                })
                .await
                .unwrap();
            let entry = &page.entries[0];
            assert_eq!(entry.entity, "ordenes");
            assert_eq!(entry.entity_id, Some(created.id.to_string()));
            for (field, old_value, new_value) in [
                ("email", previous.client.email, email.map(str::to_owned)),
                (
                    "estimatedDelivery",
                    previous.estimated_delivery_at,
                    delivery.map(str::to_owned),
                ),
            ] {
                let change = entry.details.iter().find(|d| d.field == field).unwrap();
                assert_eq!(change.old_value, old_value);
                assert_eq!(change.new_value, new_value);
            }
            let public = serde_json::to_string(&page).unwrap();
            assert!(!public.contains("test-device-PIN-2048"));
            assert!(!public.contains("edited-secret-9347"));
            assert!(!public.contains("devicePassword"));
            let history = services
                .orders
                .history(OrderHistoryPayload {
                    order_id: created.id,
                    limit: 50,
                    offset: 0,
                })
                .await
                .unwrap();
            let public_history = serde_json::to_string(&history).unwrap();
            assert!(!public_history.contains("edited-secret-9347"));
            assert!(!public_history.contains("test-device-PIN-2048"));
        }
    });
}

struct RejectAuditDetail;
impl crate::repositories::audit::AuditWriter for RejectAuditDetail {
    fn insert<'a>(
        &'a self,
        db: &'a mut sqlx::SqliteConnection,
        record: &'a crate::repositories::audit::AuditRecord<'_>,
    ) -> crate::repositories::RepositoryFuture<'a, i64> {
        Box::pin(async move {
            crate::repositories::audit::AuditRepository
                .insert(db, record)
                .await
        })
    }

    fn detail<'a>(
        &'a self,
        _db: &'a mut sqlx::SqliteConnection,
        _audit_id: i64,
        _field: &'a str,
        _old: Option<&'a str>,
        _new: Option<&'a str>,
    ) -> crate::repositories::RepositoryFuture<'a, ()> {
        Box::pin(async { Err(AppError::Validation("Fallo de auditoría de prueba.".into())) })
    }
}

#[test]
fn injected_audit_failure_rolls_back_repairs_expenses_and_evidence_including_its_file() {
    tauri::async_runtime::block_on(async {
        use crate::{
            domain::{
                evidence_contracts::NewEvidence, repair_contracts::SaveRepairPayload,
                report_contracts::ExpensePayload,
            },
            repositories::{
                evidence::EvidenceRepository, ledger::LedgerRepository, orders::OrderRepository,
                repairs::RepairRepository, reports::ReportRepository,
            },
        };
        let (pool, services) = fixture().await;
        let created = services.orders.create(order()).await.unwrap();
        let audits = count(&pool, "auditoria").await;
        let details = count(&pool, "auditoria_detalle").await;
        let audit = Arc::new(super::audit::AuditService::new(Arc::new(RejectAuditDetail)));
        let repairs = super::repairs::RepairService::new(
            pool.clone(),
            Arc::new(RepairRepository),
            Arc::new(LedgerRepository),
            services.access.clone(),
            audit.clone(),
        );
        let mut snapshot = repairs.load(created.id).await.unwrap();
        snapshot.repair.public_note = "Cambio que se debe revertir".into();
        assert!(matches!(
            repairs
                .save(SaveRepairPayload {
                    order_id: created.id,
                    revision: snapshot.revision,
                    repair: snapshot.repair,
                })
                .await,
            Err(AppError::Validation(_))
        ));
        let restored = repairs.load(created.id).await.unwrap();
        assert_eq!(restored.revision, snapshot.revision);
        assert!(restored.repair.public_note.is_empty());

        let reports = super::reports::ReportService::new(
            pool.clone(),
            Arc::new(ReportRepository),
            Arc::new(OrderRepository),
            services.access.clone(),
            audit.clone(),
        );
        assert!(matches!(
            reports
                .expense(ExpensePayload {
                    order_id: created.id,
                    description: "Refacción de prueba".into(),
                    amount_minor: 2000,
                    reason: "Prueba de rollback".into(),
                })
                .await,
            Err(AppError::Validation(_))
        ));
        assert_eq!(count(&pool, "gastos_directos").await, 0);

        let directory = tempfile::tempdir().unwrap();
        let evidence = super::evidence::EvidenceService::new(
            pool.clone(),
            Arc::new(EvidenceRepository),
            Arc::new(OrderRepository),
            Arc::new(LedgerRepository),
            services.access.clone(),
            audit,
            Some(directory.path().to_owned()),
        );
        let photo_id = "1234567890abcdef1234567890abcdef";
        assert!(matches!(
            evidence
                .add(NewEvidence {
                    id: photo_id.into(),
                    order_id: created.id,
                    original_name: "prueba.png".into(),
                    mime: "image/png".into(),
                    source: "file".into(),
                    note: "Prueba".into(),
                    bytes: b"\x89PNG\r\n\x1a\nprueba".to_vec(),
                })
                .await,
            Err(AppError::Validation(_))
        ));
        assert!(evidence.list(created.id, false).await.unwrap().is_empty());
        assert!(
            !directory
                .path()
                .join(format!("evidencias/{}/{photo_id}.png", created.id))
                .exists()
        );
        assert_eq!(count(&pool, "auditoria").await, audits);
        assert_eq!(count(&pool, "auditoria_detalle").await, details);
    });
}

struct RejectBalance;
impl crate::repositories::ledger::LedgerReader for RejectBalance {
    fn active<'a>(
        &'a self,
        db: &'a mut sqlx::SqliteConnection,
        id: i64,
    ) -> crate::repositories::RepositoryFuture<'a, ()> {
        Box::pin(async move {
            crate::repositories::ledger::LedgerRepository
                .active(db, id)
                .await
        })
    }
    fn state<'a>(
        &'a self,
        _db: &'a mut sqlx::SqliteConnection,
        _id: i64,
    ) -> crate::repositories::RepositoryFuture<'a, crate::domain::ledger_contracts::PaymentState>
    {
        Box::pin(async { Err(AppError::Validation("Fallo de saldo de prueba.".into())) })
    }
}

#[test]
fn injected_balance_failure_after_repair_write_reverts_repair_and_audit() {
    tauri::async_runtime::block_on(async {
        use crate::{
            domain::repair_contracts::SaveRepairPayload,
            repositories::{audit::AuditRepository, repairs::RepairRepository},
        };
        let (pool, services) = fixture().await;
        let created = services.orders.create(order()).await.unwrap();
        let before = services.repairs.load(created.id).await.unwrap();
        let audits = count(&pool, "auditoria").await;
        let repairs = super::repairs::RepairService::new(
            pool.clone(),
            Arc::new(RepairRepository),
            Arc::new(RejectBalance),
            services.access.clone(),
            Arc::new(super::audit::AuditService::new(Arc::new(AuditRepository))),
        );
        let mut repair = before.repair;
        repair.public_note = "Se revierte al fallar el saldo".into();
        assert!(matches!(
            repairs
                .save(SaveRepairPayload {
                    order_id: created.id,
                    revision: before.revision,
                    repair,
                })
                .await,
            Err(AppError::Validation(_))
        ));
        let after = services.repairs.load(created.id).await.unwrap();
        assert_eq!(after.revision, before.revision);
        assert!(after.repair.public_note.is_empty());
        assert_eq!(count(&pool, "auditoria").await, audits);
    });
}

struct MissingOrder;
impl crate::repositories::orders::OrderReader for MissingOrder {
    fn load_any<'a>(
        &'a self,
        _db: &'a mut sqlx::SqliteConnection,
        _id: i64,
        _include_deleted: bool,
    ) -> crate::repositories::RepositoryFuture<'a, Option<OrderResponse>> {
        Box::pin(async { Ok(None) })
    }
}

#[test]
fn evidence_and_reports_use_the_injected_order_reader() {
    tauri::async_runtime::block_on(async {
        use crate::{
            domain::report_contracts::ExpensePayload,
            repositories::{
                audit::AuditRepository, evidence::EvidenceRepository, ledger::LedgerRepository,
                reports::ReportRepository,
            },
        };
        let (pool, services) = fixture().await;
        let created = services.orders.create(order()).await.unwrap();
        let audits = count(&pool, "auditoria").await;
        let audit = Arc::new(super::audit::AuditService::new(Arc::new(AuditRepository)));
        let orders = Arc::new(MissingOrder);
        let evidence = super::evidence::EvidenceService::new(
            pool.clone(),
            Arc::new(EvidenceRepository),
            orders.clone(),
            Arc::new(LedgerRepository),
            services.access.clone(),
            audit.clone(),
            None,
        );
        assert!(matches!(
            evidence.list(created.id, false).await,
            Err(AppError::NotFound)
        ));
        let reports = super::reports::ReportService::new(
            pool.clone(),
            Arc::new(ReportRepository),
            orders,
            services.access.clone(),
            audit,
        );
        assert!(matches!(
            reports
                .expense(ExpensePayload {
                    order_id: created.id,
                    description: "Prueba".into(),
                    amount_minor: 1000,
                    reason: "Prueba de consulta".into(),
                })
                .await,
            Err(AppError::NotFound)
        ));
        assert_eq!(count(&pool, "gastos_directos").await, 0);
        assert_eq!(count(&pool, "auditoria").await, audits);
    });
}

#[test]
fn restored_company_and_users_persist_with_permissions_and_revisions() {
    tauri::async_runtime::block_on(async {
        use crate::domain::{
            company_contracts::SaveCompanyPayload, user_contracts::SaveUserPayload,
        };
        let (pool, services) = fixture().await;
        assert!(services.company.load().await.unwrap().is_none());
        let saved = services
            .company
            .save(SaveCompanyPayload {
                value: company_settings(),
                expected_revision: 0,
            })
            .await
            .unwrap();
        assert_eq!(saved.revision, 1);
        assert_eq!(saved.next_number, 20);
        assert_eq!(saved.clauses.len(), 1);
        assert!(matches!(
            services
                .company
                .save(SaveCompanyPayload {
                    value: company_settings(),
                    expected_revision: 0
                })
                .await,
            Err(AppError::Conflict(_))
        ));
        services
            .users
            .save(SaveUserPayload {
                id: None,
                name: "tecnico-restaurado".into(),
                role: "technician".into(),
                access_level: "limited".into(),
                active: true,
                permissions: vec![],
                password: "account-test-123".into(),
                confirmation: "account-test-123".into(),
                master_password: "master-test-123".into(),
            })
            .await
            .unwrap();
        let users = services.users.list().await.unwrap();
        assert!(users.iter().any(|u| u.name == "admin"));
        assert!(users.iter().any(|u| u.name == "tecnico-restaurado"));
        assert_eq!(count(&pool, "clausulas").await, 1);
        services.access.logout().await.unwrap();
        services
            .access
            .login(login("tecnico-restaurado"))
            .await
            .unwrap();
        assert!(matches!(
            services.users.list().await,
            Err(AppError::Forbidden)
        ));
        assert!(services.company.load().await.unwrap().is_some());
        assert!(matches!(
            services
                .company
                .save(SaveCompanyPayload {
                    value: company_settings(),
                    expected_revision: 1
                })
                .await,
            Err(AppError::Forbidden)
        ));
    });
}

#[test]
fn restored_edit_trash_finances_and_refund_preserve_original_books() {
    tauri::async_runtime::block_on(async {
        use crate::domain::{ledger_contracts::MoneyRequest, report_contracts::*};
        let (pool, services) = fixture().await;
        let mut first = order();
        first.initial_payment = Some(InitialPaymentInput {
            request_id: "restored-initial-001".into(),
            amount_minor: 150000,
            method: "Efectivo".into(),
        });
        let created = services.orders.create(first).await.unwrap();
        let other = services.orders.create(order()).await.unwrap();
        let mut changed = order();
        changed.price_minor = 100000;
        changed.client.full_name = "Nombre corregido".into();
        let edited = services
            .orders
            .edit(EditOrderPayload {
                order_id: created.id,
                revision: 1,
                value: changed,
                delivered: true,
                change_password: false,
            })
            .await
            .unwrap();
        assert_eq!(edited.credit_minor, 50000);
        assert_eq!(edited.net_minor, 150000);
        assert!(edited.has_device_password);
        assert_eq!(
            services
                .orders
                .get(get(other.id, false))
                .await
                .unwrap()
                .client
                .full_name,
            "Cliente de prueba"
        );
        let refund = |amount| MoneyRequest {
            request_id: "restored-refund-001".into(),
            order_id: created.id,
            kind: "refund".into(),
            payment_id: None,
            amount_minor: amount,
            expected_minor: 150000,
            method: "Transferencia".into(),
            reason: "Cambio de precio".into(),
        };
        assert!(matches!(
            services.ledger.record(refund(10000)).await,
            Err(AppError::Validation(_))
        ));
        services.ledger.record(refund(50000)).await.unwrap();
        services.ledger.record(refund(50000)).await.unwrap();
        assert_eq!(count(&pool, "devoluciones").await, 1);
        services
            .reports
            .expense(ExpensePayload {
                order_id: created.id,
                description: "Envío".into(),
                amount_minor: 5000,
                reason: "Refacción solicitada".into(),
            })
            .await
            .unwrap();
        let financial = services.reports.finances().await.unwrap();
        let row = financial
            .iter()
            .find(|r| r.database_id == created.id)
            .unwrap();
        assert_eq!(row.collected_minor, 100000);
        assert_eq!(row.profit_minor, 95000);
        assert_eq!(row.classification, "completed");
        services
            .orders
            .trash(TrashOrderPayload {
                order_id: created.id,
                revision: edited.revision,
                deleted: true,
            })
            .await
            .unwrap();
        assert!(matches!(
            services.orders.get(get(created.id, false)).await,
            Err(AppError::NotFound)
        ));
        assert!(
            services
                .reports
                .finances()
                .await
                .unwrap()
                .iter()
                .find(|r| r.database_id == created.id)
                .unwrap()
                .excluded
        );
        let mut query = get(created.id, false);
        query.include_deleted = true;
        let trashed = services.orders.get(query).await.unwrap();
        services
            .orders
            .trash(TrashOrderPayload {
                order_id: created.id,
                revision: trashed.revision,
                deleted: false,
            })
            .await
            .unwrap();
        assert_eq!(
            services
                .orders
                .get(get(created.id, false))
                .await
                .unwrap()
                .net_minor,
            100000
        );
        assert_eq!(count(&pool, "abonos").await, 1);
        let history = services
            .reports
            .logbook(LogbookFilters {
                query: created.folio,
                operation: "payments.refund".into(),
                author_id: "".into(),
                order_id: "".into(),
                from_utc: "".into(),
                to_utc: "".into(),
                page: 0,
            })
            .await
            .unwrap();
        assert_eq!(history.total, 1);
        assert_eq!(history.entries[0].operation, "payments.refund");
    });
}

#[test]
fn normalized_finances_classify_payment_and_delivery_independently_in_each_currency() {
    tauri::async_runtime::block_on(async {
        use crate::domain::ledger_contracts::MoneyRequest;
        let (_, services) = fixture().await;
        let cases = [
            (150000, true, "completed"),
            (150000, false, "paid_uncollected"),
            (25000, true, "delivered_unpaid"),
            (0, false, "pending"),
        ];
        let mut expected = Vec::new();
        for currency in ["MXN", "USD"] {
            for (index, (collected, delivered, classification)) in cases.iter().enumerate() {
                let mut payload = order();
                payload.currency = currency.into();
                let created = services.orders.create(payload).await.unwrap();
                if *collected > 0 {
                    services
                        .ledger
                        .record(MoneyRequest {
                            request_id: format!("classification-{currency}-{index}"),
                            order_id: created.id,
                            kind: "receipt".into(),
                            payment_id: None,
                            amount_minor: *collected,
                            expected_minor: 0,
                            method: "Efectivo".into(),
                            reason: "Prueba de clasificación financiera".into(),
                        })
                        .await
                        .unwrap();
                }
                if *delivered {
                    let mut edited = order();
                    edited.currency = currency.into();
                    services
                        .orders
                        .edit(EditOrderPayload {
                            order_id: created.id,
                            revision: created.revision,
                            value: edited,
                            delivered: true,
                            change_password: false,
                        })
                        .await
                        .unwrap();
                }
                expected.push((
                    created.id,
                    currency,
                    *collected,
                    *delivered,
                    *classification,
                ));
            }
        }
        let report = services.reports.finances().await.unwrap();
        assert_eq!(report.len(), expected.len());
        for (id, currency, collected, delivered, classification) in expected {
            let row = report.iter().find(|row| row.database_id == id).unwrap();
            assert_eq!(row.classification, classification, "Orden {id}");
            assert_eq!(row.currency, currency);
            assert_eq!(row.total_minor, 150000);
            assert_eq!(row.collected_minor, collected);
            assert_eq!(row.balance_minor, 150000 - collected);
            assert_eq!(row.paid, collected == 150000);
            assert_eq!(row.delivered, delivered);
            assert_eq!(row.status, "En diagnóstico");
            assert!(!row.excluded);
        }
    });
}

#[test]
fn restored_evidence_and_company_reopen_without_business_json() {
    tauri::async_runtime::block_on(async {
        use crate::domain::{company_contracts::SaveCompanyPayload, evidence_contracts::*};
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("restored.db");
        let db = Database::open_file(&path).await.unwrap();
        let services = BackendServices::new(db, keys()).await.unwrap();
        services.access.initialize(setup()).await.unwrap();
        services
            .company
            .save(SaveCompanyPayload {
                value: company_settings(),
                expected_revision: 0,
            })
            .await
            .unwrap();
        let created = services.orders.create(order()).await.unwrap();
        let code = "1234567890abcdef1234567890abcdef";
        let photo = || NewEvidence {
            id: code.into(),
            order_id: created.id,
            original_name: "prueba.png".into(),
            mime: "image/png".into(),
            source: "file".into(),
            note: "Recepción".into(),
            bytes: b"\x89PNG\r\n\x1a\nprueba".to_vec(),
        };
        services.evidence.add(photo()).await.unwrap();
        services.evidence.add(photo()).await.unwrap();
        assert_eq!(
            services
                .evidence
                .list(created.id, false)
                .await
                .unwrap()
                .len(),
            1
        );
        drop(services);
        let db = Database::open_file(&path).await.unwrap();
        let services = BackendServices::new(db, keys()).await.unwrap();
        services.access.login(login("admin")).await.unwrap();
        assert_eq!(
            services.company.load().await.unwrap().unwrap().prefix,
            "PR-"
        );
        assert_eq!(
            services.evidence.read(created.id, code).await.unwrap(),
            photo().bytes
        );
        let update = || UpdateEvidence {
            order_id: created.id,
            id: code.into(),
            note: "Recepción".into(),
            deleted: true,
            expected_revision: 1,
        };
        services.evidence.update(update()).await.unwrap();
        assert!(matches!(
            services.evidence.update(update()).await,
            Err(AppError::Conflict(_))
        ));
        assert!(
            services
                .evidence
                .list(created.id, false)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            services
                .evidence
                .list(created.id, true)
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            services.evidence.read(created.id, code).await.unwrap(),
            photo().bytes
        );
    });
}

#[test]
fn restored_repair_uses_relational_records_and_rejects_stale_changes() {
    tauri::async_runtime::block_on(async {
        use crate::domain::repair_contracts::*;
        let (pool, services) = fixture().await;
        let created = services.orders.create(order()).await.unwrap();
        let mut snapshot = services.repairs.load(created.id).await.unwrap();
        assert_eq!(snapshot.revision, 0);
        snapshot.repair.stages.insert(1, "Diagnóstico".into());
        snapshot.repair.events.push(Event {
            id: 123,
            date: "2026-11-05T10:00".into(),
            title: "Revisión inicial".into(),
            detail: "Prueba".into(),
            stage: "Diagnóstico".into(),
            public: true,
        });
        snapshot.repair.parts.push(Part {
            name: "Conector".into(),
            price: 12550,
        });
        snapshot.repair.warranties.push(Warranty {
            name: "Conector".into(),
            days: 30,
        });
        let saved = services
            .repairs
            .save(SaveRepairPayload {
                order_id: created.id,
                repair: snapshot.repair,
                revision: 0,
            })
            .await
            .unwrap();
        assert_eq!(saved.repair.parts[0].price, 12550);
        assert_eq!(count(&pool, "avances").await, 1);
        assert_eq!(count(&pool, "refacciones").await, 1);
        assert_eq!(
            saved.repair.stages,
            vec!["Recibido", "Diagnóstico", "Entregado"]
        );
        assert!(matches!(
            services
                .repairs
                .save(SaveRepairPayload {
                    order_id: created.id,
                    repair: saved.repair.clone(),
                    revision: 0
                })
                .await,
            Err(AppError::Conflict(_))
        ));
        let mut next = saved.repair;
        next.completed = 1;
        sqlx::query("UPDATE ordenes SET entregado=1 WHERE id=?")
            .bind(created.id)
            .execute(&pool)
            .await
            .unwrap();
        services
            .repairs
            .save(SaveRepairPayload {
                order_id: created.id,
                repair: next,
                revision: 1,
            })
            .await
            .unwrap();
        assert_eq!(count(&pool, "avances").await, 1);
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT entregado FROM ordenes WHERE id=?")
                .bind(created.id)
                .fetch_one(&pool)
                .await
                .unwrap(),
            1
        );
    });
}

#[test]
fn restored_ledger_retries_exact_request_and_rolls_back_audit_failure() {
    tauri::async_runtime::block_on(async {
        use crate::domain::ledger_contracts::MoneyRequest;
        let (pool, services) = fixture().await;
        let created = services.orders.create(order()).await.unwrap();
        let request = |id: &str, amount: i64| MoneyRequest {
            request_id: id.into(),
            order_id: created.id,
            kind: "receipt".into(),
            payment_id: None,
            amount_minor: amount,
            expected_minor: 0,
            method: "Transferencia".into(),
            reason: "".into(),
        };
        let first = services
            .ledger
            .record(request("restored-payment-001", 25000))
            .await
            .unwrap();
        assert_eq!(first.collected_minor, 25000);
        services
            .ledger
            .record(request("restored-payment-001", 25000))
            .await
            .unwrap();
        assert_eq!(count(&pool, "abonos").await, 1);
        assert!(matches!(
            services
                .ledger
                .record(request("restored-payment-001", 25001))
                .await,
            Err(AppError::Conflict(_))
        ));
        let mut next = request("restored-payment-002", 100);
        next.expected_minor = 25000;
        sqlx::query("CREATE TRIGGER test_audit_failure BEFORE INSERT ON auditoria BEGIN SELECT RAISE(ABORT,'test'); END").execute(&pool).await.unwrap();
        assert!(services.ledger.record(next).await.is_err());
        assert_eq!(count(&pool, "abonos").await, 1);
        assert_eq!(count(&pool, "solicitudes_financieras").await, 1);
    });
}
fn setup() -> SetupPayload {
    SetupPayload {
        name: "Administrador de prueba".into(),
        username: "admin".into(),
        password: "account-test-123".into(),
        confirmation: "account-test-123".into(),
        master_password: "master-test-123".into(),
        master_confirmation: "master-test-123".into(),
    }
}
fn login(username: &str) -> LoginPayload {
    LoginPayload {
        username: username.into(),
        password: "account-test-123".into(),
    }
}
fn order() -> CreateOrderPayload {
    CreateOrderPayload {
        client: ClientInput {
            id: None,
            full_name: "Cliente de prueba".into(),
            phone: "0000000000".into(),
            email: Some("prueba@example.invalid".into()),
        },
        equipment: EquipmentInput {
            id: None,
            item: "Laptop".into(),
            brand: "Marca".into(),
            model: "Modelo".into(),
            serial: "TEST-SERIAL".into(),
        },
        history: EquipmentHistory::default(),
        folio_prefix: "ORD-".into(),
        sequence_start: 1,
        status: "diagnosis".into(),
        currency: "MXN".into(),
        received_at: "2026-11-04".into(),
        estimated_delivery_at: Some("2026-11-10".into()),
        reported_failure: "No enciende".into(),
        planned_work: "Diagnóstico".into(),
        reception_notes: "Recepción de prueba".into(),
        price_minor: 150000,
        device_password: Some("test-device-PIN-2048".into()),
        initial_payment: None,
    }
}

#[test]
fn equipment_history_text_preserves_old_fields_audits_changes_and_rolls_back() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        let mut original = order();
        original.history.previous_workshop = true;
        original.history.previous_workshop_detail = "Revisión en otro taller".into();
        let created = services.orders.create(original).await.unwrap();
        assert_eq!(created.history.notes, None);

        let mut value = order();
        value.history = created.history;
        value.history.notes = Some("Otro taller; golpe lateral y humedad.".into());
        let saved = services
            .orders
            .edit(EditOrderPayload {
                order_id: created.id,
                revision: created.revision,
                value,
                delivered: false,
                change_password: false,
            })
            .await
            .unwrap();
        assert_eq!(
            saved.history.notes.as_deref(),
            Some("Otro taller; golpe lateral y humedad.")
        );
        assert!(saved.history.previous_workshop);
        assert_eq!(
            saved.history.previous_workshop_detail,
            "Revisión en otro taller"
        );
        let audit: (Option<String>, Option<String>) = sqlx::query_as(
            "SELECT valor_anterior,valor_nuevo FROM auditoria_detalle WHERE campo='equipmentHistory' AND valor_nuevo IS NOT NULL ORDER BY id DESC LIMIT 1"
        ).fetch_one(&pool).await.unwrap();
        assert_eq!(audit, (None, saved.history.notes.clone()));

        let mut empty = order();
        empty.history = saved.history;
        empty.history.notes = Some(String::new());
        let cleared = services
            .orders
            .edit(EditOrderPayload {
                order_id: saved.id,
                revision: saved.revision,
                value: empty,
                delivered: false,
                change_password: false,
            })
            .await
            .unwrap();
        assert_eq!(cleared.history.notes.as_deref(), Some(""));
        assert!(cleared.history.previous_workshop);

        let mut invalid = order();
        invalid.history.notes = Some("a".repeat(16001));
        assert!(matches!(
            services.orders.create(invalid).await,
            Err(AppError::Validation(_))
        ));
        let audits = count(&pool, "auditoria").await;
        sqlx::query("CREATE TRIGGER history_audit_failure BEFORE INSERT ON auditoria BEGIN SELECT RAISE(ABORT,'test'); END").execute(&pool).await.unwrap();
        let mut rejected = order();
        rejected.history.notes = Some("No debe confirmarse".into());
        assert!(
            services
                .orders
                .edit(EditOrderPayload {
                    order_id: cleared.id,
                    revision: cleared.revision,
                    value: rejected,
                    delivered: false,
                    change_password: false,
                })
                .await
                .is_err()
        );
        let stored: (String, i64) =
            sqlx::query_as("SELECT antecedentes_equipo,revision FROM ordenes WHERE id=?")
                .bind(cleared.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stored, (String::new(), cleared.revision));
        assert_eq!(count(&pool, "auditoria").await, audits);
    });
}

#[test]
fn integrated_notes_warranties_parts_receipts_adjustments_refunds_and_currencies() {
    tauri::async_runtime::block_on(async {
        use crate::domain::{ledger_contracts::MoneyRequest, repair_contracts::*};
        let (pool, services) = fixture().await;
        for currency in ["MXN", "USD"] {
            let mut payload = order();
            payload.currency = currency.into();
            let created = services.orders.create(payload).await.unwrap();
            let snapshot = services.repairs.load(created.id).await.unwrap();
            let mut repair = snapshot.repair;
            repair.public_note = "Cambio de conector terminado".into();
            repair.internal_note = "Nota privada de laboratorio".into();
            repair.technician = "Técnico de prueba".into();
            repair.parts.push(Part {
                name: "Conector".into(),
                price: 12550,
            });
            repair.warranties.push(Warranty {
                name: "Conector".into(),
                days: 30,
            });
            services
                .repairs
                .save(SaveRepairPayload {
                    order_id: created.id,
                    revision: snapshot.revision,
                    repair,
                })
                .await
                .unwrap();
            let request = |kind: &str, suffix: &str, amount, expected, payment_id| MoneyRequest {
                request_id: format!("integrated-{currency}-{suffix}"),
                order_id: created.id,
                kind: kind.into(),
                payment_id,
                amount_minor: amount,
                expected_minor: expected,
                method: if kind == "adjustment" { "" } else { "Efectivo" }.into(),
                reason: "Validación del flujo integrado".into(),
            };
            services
                .ledger
                .record(request("receipt", "receipt", 100000, 0, None))
                .await
                .unwrap();
            let ledger = services.ledger.list(created.id).await.unwrap();
            let receipt_id: i64 = ledger
                .movements
                .iter()
                .find(|row| row.kind == "receipt")
                .unwrap()
                .id
                .parse()
                .unwrap();
            services
                .ledger
                .record(request(
                    "adjustment",
                    "adjustment",
                    -10000,
                    100000,
                    Some(receipt_id),
                ))
                .await
                .unwrap();
            let mut reduced = order();
            reduced.currency = currency.into();
            reduced.price_minor = 60000;
            let edited = services
                .orders
                .edit(EditOrderPayload {
                    order_id: created.id,
                    revision: created.revision,
                    value: reduced,
                    delivered: true,
                    change_password: false,
                })
                .await
                .unwrap();
            assert_eq!(edited.credit_minor, 30000);
            services
                .ledger
                .record(request("refund", "refund", 30000, 90000, None))
                .await
                .unwrap();
            let summary =
                serde_json::to_value(services.repairs.summary(created.id).await.unwrap()).unwrap();
            assert_eq!(summary["publicNote"], "Cambio de conector terminado");
            assert_eq!(summary["warranties"][0]["days"], 30);
            assert!(!summary.to_string().contains("Nota privada"));
            assert!(!summary.to_string().contains("test-device-PIN"));
        }
        let finances = services.reports.finances().await.unwrap();
        assert_eq!(finances.len(), 2);
        for currency in ["MXN", "USD"] {
            let row = finances
                .iter()
                .find(|row| row.currency == currency)
                .unwrap();
            assert_eq!(
                (row.received_minor, row.adjusted_minor, row.refunded_minor),
                (100000, -10000, 30000)
            );
            assert_eq!(
                (
                    row.total_minor,
                    row.collected_minor,
                    row.credit_minor,
                    row.balance_minor
                ),
                (60000, 60000, 0, 0)
            );
            assert_eq!((row.costs_minor, row.profit_minor), (12550, 47450));
            assert_eq!(row.classification, "completed");
            assert_eq!(row.payments.len(), 3);
        }
        assert_eq!(count(&pool, "abonos").await, 2);
        assert_eq!(count(&pool, "ajustes_abono").await, 2);
        assert_eq!(count(&pool, "devoluciones").await, 2);
    });
}

#[test]
fn integrated_evidence_revision_restore_and_revoked_authorization_preserve_original() {
    tauri::async_runtime::block_on(async {
        use crate::domain::evidence_contracts::{NewEvidence, UpdateEvidence};
        let directory = tempfile::tempdir().unwrap();
        let db = Database::open_file(&directory.path().join("evidence-matrix.db"))
            .await
            .unwrap();
        let pool = db.pool().clone();
        let services = BackendServices::new(db, keys()).await.unwrap();
        services.access.initialize(setup()).await.unwrap();
        let created = services.orders.create(order()).await.unwrap();
        let code = "11112222333344445555666677778888";
        let bytes = b"\x89PNG\r\n\x1a\nprueba".to_vec();
        let original = services
            .evidence
            .add(NewEvidence {
                id: code.into(),
                order_id: created.id,
                original_name: "original.png".into(),
                mime: "image/png".into(),
                source: "file".into(),
                note: "Recepción".into(),
                bytes: bytes.clone(),
            })
            .await
            .unwrap();
        let update = |revision, deleted, note: &str| UpdateEvidence {
            id: code.into(),
            order_id: created.id,
            expected_revision: revision,
            deleted,
            note: note.into(),
        };
        let edited = services
            .evidence
            .update(update(original.revision, false, "Golpe lateral"))
            .await
            .unwrap();
        assert!(matches!(
            services
                .evidence
                .update(update(original.revision, false, "Revisión antigua"))
                .await,
            Err(AppError::Conflict(_))
        ));
        let removed = services
            .evidence
            .update(update(edited.revision, true, &edited.note))
            .await
            .unwrap();
        assert!(
            services
                .evidence
                .list(created.id, false)
                .await
                .unwrap()
                .is_empty()
        );
        let restored = services
            .evidence
            .update(update(removed.revision, false, &removed.note))
            .await
            .unwrap();
        assert_eq!(restored.created_at, original.created_at);
        assert_eq!(restored.author_name, original.author_name);
        assert_eq!(restored.original_name, original.original_name);
        assert_eq!(
            services.evidence.read(created.id, code).await.unwrap(),
            bytes
        );

        let reader = add_user(&pool, "evidence-reader", "custom").await;
        let approver = add_user(&pool, "evidence-approver", "custom").await;
        for (user_id, permissions) in [(reader, vec!["view"]), (approver, vec!["view", "edit"])] {
            for permission in permissions {
                sqlx::query("INSERT INTO usuario_permiso(id_usuario,id_permiso) SELECT ?,p.id FROM permisos p JOIN apartados a ON a.id=p.id_apartado JOIN acciones c ON c.id=p.id_accion WHERE a.codigo='clients' AND c.codigo=?")
                    .bind(user_id).bind(permission).execute(&pool).await.unwrap();
            }
        }
        services
            .access
            .login(login("evidence-reader"))
            .await
            .unwrap();
        assert_eq!(
            services
                .evidence
                .list(created.id, false)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(matches!(
            services
                .evidence
                .update(update(restored.revision, false, "Sin permiso"))
                .await,
            Err(AppError::Forbidden)
        ));
        assert!(
            services
                .access
                .temporary(loan(
                    "evidence-approver",
                    AccessScope::Repairs,
                    Some(created.id),
                    "evidence.edit"
                ))
                .await
                .is_err()
        );
        services
            .access
            .temporary(loan(
                "evidence-approver",
                AccessScope::Clients,
                None,
                "evidence.edit",
            ))
            .await
            .unwrap();
        sqlx::query("DELETE FROM usuario_permiso WHERE id_usuario=? AND id_permiso=(SELECT p.id FROM permisos p JOIN apartados a ON a.id=p.id_apartado JOIN acciones c ON c.id=p.id_accion WHERE a.codigo='clients' AND c.codigo='edit')")
            .bind(approver).execute(&pool).await.unwrap();
        let audits = count(&pool, "auditoria").await;
        assert!(matches!(
            services
                .evidence
                .update(update(restored.revision, false, "Autorización revocada"))
                .await,
            Err(AppError::Forbidden)
        ));
        assert_eq!(
            services.evidence.list(created.id, false).await.unwrap()[0].note,
            "Golpe lateral"
        );
        assert_eq!(
            services.evidence.read(created.id, code).await.unwrap(),
            bytes
        );
        assert_eq!(count(&pool, "evidencias").await, 1);
        assert_eq!(count(&pool, "auditoria").await, audits);
    });
}
fn loan(
    username: &str,
    scope: AccessScope,
    order_id: Option<i64>,
    operation: &str,
) -> TemporaryAccessPayload {
    TemporaryAccessPayload {
        username: username.into(),
        password: "account-test-123".into(),
        scope,
        order_id,
        operation: operation.into(),
        reason: "Autorización de prueba".into(),
    }
}
fn get(id: i64, secret: bool) -> GetOrderPayload {
    GetOrderPayload {
        include_deleted: false,
        order_id: id,
        scope: AccessScope::Clients,
        include_device_password: secret,
    }
}
async fn empty() -> (SqlitePool, BackendServices) {
    let db = Database::in_memory().await.unwrap();
    let pool = db.pool().clone();
    let services = BackendServices::new(db, keys()).await.unwrap();
    (pool, services)
}
async fn fixture() -> (SqlitePool, BackendServices) {
    let (pool, services) = empty().await;
    services.access.initialize(setup()).await.unwrap();
    (pool, services)
}
async fn count(pool: &SqlitePool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT count(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .unwrap()
}
async fn add_user(pool: &SqlitePool, name: &str, level: &str) -> i64 {
    let encoded = super::access::hash("account-test-123").await.unwrap();
    sqlx::query("INSERT INTO usuarios(nombre,nombre_acceso,hash_contrasena,id_nivel,id_rol) SELECT ?,?,?,n.id,r.id FROM niveles_acceso n JOIN roles r WHERE n.codigo=? AND r.codigo='technician'")
        .bind(name).bind(name).bind(encoded).bind(level).execute(pool).await.unwrap().last_insert_rowid()
}

#[test]
fn reception_list_search_reference_history_and_balances_survive_reopen() {
    tauri::async_runtime::block_on(async {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("reception.db");
        let database = Database::open_file(&file).await.unwrap();
        let services = BackendServices::new(database, keys()).await.unwrap();
        services.access.initialize(setup()).await.unwrap();
        let mut payload = order();
        payload.client.full_name = "HÉCTOR %_ ' Cliente".into();
        payload.equipment.serial = "SERIE-UNICÁ".into();
        payload.device_password = Some("password-only-for-test".into());
        payload.price_minor = 15050;
        payload.initial_payment = Some(InitialPaymentInput {
            request_id: "reception-persistent-payment".into(),
            amount_minor: 2005,
            method: "Efectivo".into(),
        });
        let first = services.orders.create(payload).await.unwrap();
        services.orders.create(order()).await.unwrap();
        drop(services);
        let database = Database::open_file(&file).await.unwrap();
        let services = BackendServices::new(database, keys()).await.unwrap();
        let page = |query: &str, offset, limit| ListOrdersPayload {
            trashed: false,
            query: query.into(),
            field: OrderSearchField::All,
            oldest_first: false,
            offset,
            limit,
        };
        assert!(matches!(
            services.orders.list(page("", 0, 5)).await,
            Err(AppError::Unauthorized)
        ));
        services.access.login(login("admin")).await.unwrap();
        let result = services.orders.list(page("", 0, 1)).await.unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.items.len(), 1);
        assert_ne!(result.items[0].id, first.id);
        assert_eq!(
            services.orders.list(page("", 1, 1)).await.unwrap().items[0].id,
            first.id
        );
        assert!(
            services
                .orders
                .list(page("", 2, 1))
                .await
                .unwrap()
                .items
                .is_empty()
        );
        for query in ["héctor", "%_", "'", "serie-unicá", first.folio.as_str()] {
            let result = services.orders.list(page(query, 0, 5)).await.unwrap();
            assert_eq!(result.total, 1, "{query}");
            assert_eq!(result.items[0].id, first.id);
            assert_eq!(result.items[0].pending_minor, 13045);
            assert_eq!(result.items[0].net_minor, 2005);
            let text = serde_json::to_string(&result).unwrap();
            assert!(!text.contains("password-only-for-test") && !text.contains("devicePassword"));
        }
        for (field, query, expected) in [
            (OrderSearchField::Client, "héctor", 1),
            (OrderSearchField::Serial, "héctor", 0),
            (OrderSearchField::Serial, "serie-unicá", 1),
            (OrderSearchField::Order, first.folio.as_str(), 1),
            (OrderSearchField::Client, first.folio.as_str(), 0),
        ] {
            let mut request = page(query, 0, 1);
            request.field = field;
            assert_eq!(services.orders.list(request).await.unwrap().total, expected);
        }
        let mut request = page("", 0, 1);
        request.oldest_first = true;
        assert_eq!(
            services.orders.list(request).await.unwrap().items[0].id,
            first.id
        );
        assert!(
            serde_json::from_str::<ListOrdersPayload>(
                r#"{"query":"","limit":5,"field":"folio; DROP TABLE ordenes"}"#
            )
            .is_err()
        );
        assert_eq!(
            services
                .orders
                .list(page("' OR 1=1 --", 0, 5))
                .await
                .unwrap()
                .total,
            0
        );
        for reference in [first.folio.clone(), first.id.to_string()] {
            let found = services
                .orders
                .find(FindOrderPayload { reference })
                .await
                .unwrap();
            assert_eq!(found.id, first.id);
            assert!(found.has_device_password && found.device_password.is_none());
            assert_eq!(found.reported_failure, first.reported_failure);
        }
        let history = services
            .orders
            .history(OrderHistoryPayload {
                order_id: first.id,
                offset: 0,
                limit: 50,
            })
            .await
            .unwrap();
        assert_eq!(history.total, 2);
        assert!(
            history
                .items
                .iter()
                .any(|item| item.operation == "orders.create")
        );
        assert!(
            history
                .items
                .iter()
                .any(|item| item.operation == "payments.add")
        );
        assert!(
            !serde_json::to_string(&history)
                .unwrap()
                .contains("password-only-for-test")
        );
        assert_eq!(
            services
                .orders
                .history(OrderHistoryPayload {
                    order_id: first.id,
                    offset: 1,
                    limit: 1
                })
                .await
                .unwrap()
                .items
                .len(),
            1
        );
        assert!(matches!(
            services.orders.list(page("", -1, 5)).await,
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            services.orders.list(page("", 0, 101)).await,
            Err(AppError::Validation(_))
        ));
    });
}

#[test]
fn reception_history_requires_logbook_permission_and_list_rechecks_active_users() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        let saved = services.orders.create(order()).await.unwrap();
        let limited = add_user(&pool, "limited-reader", "limited").await;
        services.access.logout().await.unwrap();
        services
            .access
            .login(login("limited-reader"))
            .await
            .unwrap();
        assert_eq!(
            services
                .orders
                .list(ListOrdersPayload {
                    trashed: false,
                    query: "".into(),
                    field: OrderSearchField::All,
                    oldest_first: false,
                    offset: 0,
                    limit: 5
                })
                .await
                .unwrap()
                .total,
            1
        );
        assert!(matches!(
            services
                .orders
                .history(OrderHistoryPayload {
                    order_id: saved.id,
                    offset: 0,
                    limit: 50
                })
                .await,
            Err(AppError::Forbidden)
        ));
        sqlx::query("UPDATE usuarios SET activo=0 WHERE id=?")
            .bind(limited)
            .execute(&pool)
            .await
            .unwrap();
        assert!(matches!(
            services
                .orders
                .list(ListOrdersPayload {
                    trashed: false,
                    query: "".into(),
                    field: OrderSearchField::All,
                    oldest_first: false,
                    offset: 0,
                    limit: 5
                })
                .await,
            Err(AppError::Unauthorized)
        ));
        services.access.login(login("admin")).await.unwrap();
        sqlx::query("UPDATE ordenes SET eliminada_en='2026-11-05' WHERE id=?")
            .bind(saved.id)
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            services
                .orders
                .list(ListOrdersPayload {
                    trashed: false,
                    query: "".into(),
                    field: OrderSearchField::All,
                    oldest_first: false,
                    offset: 0,
                    limit: 5
                })
                .await
                .unwrap()
                .total,
            0
        );
        assert!(matches!(
            services
                .orders
                .find(FindOrderPayload {
                    reference: saved.folio
                })
                .await,
            Err(AppError::NotFound)
        ));
    });
}

#[test]
fn normalized_window_profile_tracks_setup_session_logout_and_native_failure() {
    tauri::async_runtime::block_on(async {
        let (_, services) = empty().await;
        let states = std::sync::Mutex::new(Vec::new());
        let apply = |authenticated, configured| {
            states.lock().unwrap().push((authenticated, configured));
            Ok(())
        };
        services.access.synchronize_window(apply).await.unwrap();
        services.access.initialize(setup()).await.unwrap();
        services.access.synchronize_window(apply).await.unwrap();
        services.access.logout().await.unwrap();
        services.access.synchronize_window(apply).await.unwrap();
        services.access.login(login("admin")).await.unwrap();
        services.access.synchronize_window(apply).await.unwrap();
        assert_eq!(
            *states.lock().unwrap(),
            [(false, false), (true, true), (false, true), (true, true)]
        );
        assert!(matches!(
            services
                .access
                .synchronize_window(|_, _| Err("fallo nativo de prueba".into()))
                .await,
            Err(AppError::Window)
        ));
        services.access.synchronize_window(apply).await.unwrap();
        assert_eq!(states.lock().unwrap().last(), Some(&(true, true)));
    });
}

#[test]
fn crypto_authenticates_ciphertext_nonce_order_and_key_and_uses_fresh_nonces() {
    let service = CryptoService::new(keys());
    let first = service.encrypt(1, "PIN-test-only").unwrap();
    let second = service.encrypt(1, "PIN-test-only").unwrap();
    assert_ne!(first.nonce(), second.nonce());
    assert_ne!(first.ciphertext(), b"PIN-test-only");
    assert_eq!(service.decrypt(&first).unwrap().0.as_str(), "PIN-test-only");
    let mut changed = first.ciphertext().to_vec();
    changed[0] ^= 1;
    let altered =
        DeviceCredential::from_encrypted(1, "test-only".into(), *first.nonce(), changed).unwrap();
    assert!(service.decrypt(&altered).is_err());
    let swapped = DeviceCredential::from_encrypted(
        2,
        "test-only".into(),
        *first.nonce(),
        first.ciphertext().to_vec(),
    )
    .unwrap();
    assert!(service.decrypt(&swapped).is_err());
    let mut nonce = *first.nonce();
    nonce[0] ^= 1;
    let altered =
        DeviceCredential::from_encrypted(1, "test-only".into(), nonce, first.ciphertext().to_vec())
            .unwrap();
    assert!(service.decrypt(&altered).is_err());
    assert!(
        CryptoService::new(Arc::new(TestKeys([8; 32])))
            .decrypt(&first)
            .is_err()
    );
}

#[test]
fn errors_and_input_contracts_do_not_expose_storage_or_accept_float_money() {
    let error = AppError::Database(sqlx::Error::Protocol("SQL con información privada".into()));
    let value = serde_json::to_value(error).unwrap();
    assert_eq!(value["code"], "STORAGE_ERROR");
    assert!(!value.to_string().contains("privada"));
    let mut request = serde_json::to_value(order()).unwrap();
    request["priceMinor"] = serde_json::json!(100.25);
    assert!(serde_json::from_value::<CreateOrderPayload>(request).is_err());
    assert!(
        serde_json::from_value::<LoginPayload>(
            serde_json::json!({"username":"admin","password":"test","userId":1})
        )
        .is_err()
    );
}

#[test]
fn login_picker_lists_only_active_names_without_authenticating_or_exposing_account_details() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = empty().await;
        assert!(services.access.login_names().await.unwrap().is_empty());
        assert!(matches!(
            services.access.refresh_session().await,
            Err(AppError::Unauthorized)
        ));
        services.access.initialize(setup()).await.unwrap();
        let active = add_user(&pool, "ana", "limited").await;
        let inactive = add_user(&pool, "usuario-inactivo", "custom").await;
        sqlx::query("UPDATE usuarios SET activo=0 WHERE id=?")
            .bind(inactive)
            .execute(&pool)
            .await
            .unwrap();
        services.access.logout().await.unwrap();
        let audits = count(&pool, "auditoria").await;
        let names = services.access.login_names().await.unwrap();
        assert_eq!(names, vec!["admin", "ana"]);
        assert_eq!(
            serde_json::to_value(&names).unwrap(),
            serde_json::json!(["admin", "ana"])
        );
        assert_eq!(count(&pool, "auditoria").await, audits);
        assert!(matches!(
            services.access.refresh_session().await,
            Err(AppError::Unauthorized)
        ));
        assert!(matches!(services.access.login(login("ana")).await, Ok(_)));
        services.access.logout().await.unwrap();
        sqlx::query("UPDATE usuarios SET activo=0 WHERE id=?")
            .bind(active)
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(services.access.login_names().await.unwrap(), vec!["admin"]);
        assert!(matches!(
            services.access.login(login("ana")).await,
            Err(AppError::InvalidCredentials)
        ));
    });
}

#[test]
fn initial_setup_and_login_return_only_public_identity_and_permanent_permissions() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = empty().await;
        assert!(!services.access.setup_status().await.unwrap().setup_complete);
        let response = services.access.initialize(setup()).await.unwrap();
        assert!(response.permissions.contains(&"users.manage".into()));
        assert_eq!(count(&pool, "auditoria").await, 1);
        let value = serde_json::to_value(response).unwrap();
        assert!(!value.to_string().contains("hash"));
        assert!(!value.to_string().contains("account-test"));
        assert!(services.access.setup_status().await.unwrap().setup_complete);
        assert!(services.access.initialize(setup()).await.is_err());
        services.access.logout().await.unwrap();
        let response = services.access.login(login("  ADMIN  ")).await.unwrap();
        assert_eq!(response.user.login_name, "admin");
        let hash: String = sqlx::query_scalar("SELECT hash_contrasena FROM usuarios WHERE id=1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(hash.starts_with("$argon2id$"));
    });
}

#[test]
fn setup_and_login_audit_failures_do_not_leave_partial_state_or_sessions() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = empty().await;
        sqlx::raw_sql("CREATE TRIGGER fail_audit BEFORE INSERT ON auditoria BEGIN SELECT RAISE(ABORT,'prueba'); END;").execute(&pool).await.unwrap();
        assert!(services.access.initialize(setup()).await.is_err());
        assert_eq!(count(&pool, "usuarios").await, 0);
        assert_eq!(count(&pool, "claves_maestras").await, 0);
        sqlx::raw_sql("DROP TRIGGER fail_audit")
            .execute(&pool)
            .await
            .unwrap();
        services.access.initialize(setup()).await.unwrap();
        services.access.logout().await.unwrap();
        sqlx::raw_sql("CREATE TRIGGER fail_audit BEFORE INSERT ON auditoria BEGIN SELECT RAISE(ABORT,'prueba'); END;").execute(&pool).await.unwrap();
        assert!(services.access.login(login("admin")).await.is_err());
        assert!(matches!(
            services.orders.create(order()).await,
            Err(AppError::Unauthorized)
        ));
    });
}

#[test]
fn receipt_creation_encrypts_the_password_and_reuses_exact_customer_and_equipment() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        let response = services.orders.create(order()).await.unwrap();
        assert_eq!(response.folio, "ORD-001");
        assert_eq!(response.pending_minor, 150000);
        assert!(response.has_device_password);
        assert!(response.device_password.is_none());
        let public = serde_json::to_string(&response).unwrap();
        assert!(!public.contains("test-device-PIN"));
        assert!(!public.contains("\"devicePassword\""));
        let raw: Vec<u8> =
            sqlx::query_scalar("SELECT cifrado FROM credenciales_dispositivos WHERE id_orden=?")
                .bind(response.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(!raw.windows(4).any(|part| part == b"2048"));
        let restored = services.orders.get(get(response.id, true)).await.unwrap();
        assert_eq!(
            restored.device_password.unwrap().0.as_str(),
            "test-device-PIN-2048"
        );
        let again = services.orders.create(order()).await.unwrap();
        assert_eq!(again.folio, "ORD-002");
        assert_eq!(count(&pool, "clientes").await, 1);
        assert_eq!(count(&pool, "equipos").await, 1);
        let audits: Vec<String> =
            sqlx::query_scalar("SELECT coalesce(valor_nuevo,'') FROM auditoria_detalle")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert!(!audits.join("").contains("2048"));
        services.access.logout().await.unwrap();
        assert!(matches!(
            services.orders.get(get(response.id, true)).await,
            Err(AppError::Unauthorized)
        ));
    });
}

#[test]
fn failed_order_or_payment_audit_rolls_back_all_rows_and_the_folio_reservation() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        sqlx::raw_sql("CREATE TRIGGER fail_order_audit BEFORE INSERT ON auditoria WHEN NEW.entidad='ordenes' BEGIN SELECT RAISE(ABORT,'prueba'); END;").execute(&pool).await.unwrap();
        let mut payload = order();
        payload.initial_payment = Some(InitialPaymentInput {
            request_id: "payment-rollback-test".into(),
            amount_minor: 50000,
            method: "Método libre".into(),
        });
        assert!(services.orders.create(payload).await.is_err());
        for table in [
            "clientes",
            "equipos",
            "ordenes",
            "abonos",
            "solicitudes_financieras",
            "credenciales_dispositivos",
            "secuencias_folio",
        ] {
            assert_eq!(count(&pool, table).await, 0, "{table}");
        }
        assert_eq!(count(&pool, "auditoria").await, 1);
        sqlx::raw_sql("DROP TRIGGER fail_order_audit; CREATE TRIGGER fail_payment_audit BEFORE INSERT ON auditoria WHEN NEW.entidad='abonos' BEGIN SELECT RAISE(ABORT,'prueba'); END;").execute(&pool).await.unwrap();
        let mut payload = order();
        payload.initial_payment = Some(InitialPaymentInput {
            request_id: "payment-rollback-test".into(),
            amount_minor: 50000,
            method: "Libre".into(),
        });
        assert!(services.orders.create(payload).await.is_err());
        assert_eq!(count(&pool, "ordenes").await, 0);
        sqlx::raw_sql("DROP TRIGGER fail_payment_audit")
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            services.orders.create(order()).await.unwrap().folio,
            "ORD-001"
        );
    });
}

#[test]
fn same_financial_request_is_idempotent_and_changed_content_cannot_reuse_it() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        let payload = || {
            let mut p = order();
            p.initial_payment = Some(InitialPaymentInput {
                request_id: "initial-receipt-test-1".into(),
                amount_minor: 50000,
                method: "Método escrito".into(),
            });
            p
        };
        let first = services.orders.create(payload()).await.unwrap();
        let audit_count = count(&pool, "auditoria").await;
        let retry = services.orders.create(payload()).await.unwrap();
        assert_eq!(first.id, retry.id);
        assert_eq!(retry.pending_minor, 100000);
        assert_eq!(retry.net_minor, 50000);
        assert_eq!(count(&pool, "ordenes").await, 1);
        assert_eq!(count(&pool, "abonos").await, 1);
        assert_eq!(count(&pool, "auditoria").await, audit_count);
        let mut changed = payload();
        changed.device_password = Some("changed-test-only".into());
        assert!(matches!(
            services.orders.create(changed).await,
            Err(AppError::Conflict(_))
        ));
        assert_eq!(count(&pool, "ordenes").await, 1);
    });
}

#[test]
fn creation_rejects_unowned_equipment_ambiguous_clients_and_invalid_amounts() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        let original = services.orders.create(order()).await.unwrap();
        let mut wrong = order();
        wrong.client.full_name = "Otro cliente".into();
        wrong.equipment.id = Some(original.equipment_id);
        assert!(matches!(
            services.orders.create(wrong).await,
            Err(AppError::Conflict(_))
        ));
        assert_eq!(count(&pool, "clientes").await, 1);
        sqlx::query("INSERT INTO clientes(nombre,telefono,correo) VALUES ('Cliente de prueba','0000000000','prueba@example.invalid')").execute(&pool).await.unwrap();
        assert!(matches!(
            services.orders.create(order()).await,
            Err(AppError::Conflict(_))
        ));
        for amount in [-1, 9_000_000_000_001] {
            let mut payload = order();
            payload.price_minor = amount;
            assert!(matches!(
                services.orders.create(payload).await,
                Err(AppError::Validation(_))
            ));
        }
        let mut payload = order();
        payload.initial_payment = Some(InitialPaymentInput {
            request_id: "too-large-initial-receipt".into(),
            amount_minor: 150001,
            method: "Libre".into(),
        });
        assert!(matches!(
            services.orders.create(payload).await,
            Err(AppError::Validation(_))
        ));
    });
}

#[test]
fn limited_accounts_require_independent_payment_permission_and_temporary_use_is_audited() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        let limited = add_user(&pool, "limited", "limited").await;
        services.access.login(login("limited")).await.unwrap();
        let with_payment = || {
            let mut p = order();
            p.initial_payment = Some(InitialPaymentInput {
                request_id: "limited-initial-payment".into(),
                amount_minor: 50000,
                method: "Método libre".into(),
            });
            p
        };
        assert!(matches!(
            services.orders.create(with_payment()).await,
            Err(AppError::Forbidden)
        ));
        assert_eq!(count(&pool, "ordenes").await, 0);
        let grant = services
            .access
            .temporary(loan("admin", AccessScope::Clients, None, "payments.add"))
            .await
            .unwrap();
        assert_eq!(grant.recipient_id, limited);
        let order = services.orders.create(with_payment()).await.unwrap();
        assert_eq!(order.received_by, limited);
        let uses:i64=sqlx::query_scalar("SELECT count(*) FROM auditoria a JOIN operaciones_auditoria o ON o.id=a.id_operacion WHERE o.codigo='access.temporary.use' AND a.id_autor=?").bind(limited).fetch_one(&pool).await.unwrap();
        assert_eq!(uses, 1);
        services.orders.create(with_payment()).await.unwrap();
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM auditoria a JOIN operaciones_auditoria o ON o.id=a.id_operacion WHERE o.codigo='access.temporary.use'").fetch_one(&pool).await.unwrap(),1);
    });
}

#[test]
fn temporary_grants_revalidate_approver_and_cannot_escape_their_order_scope() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        let first = services.orders.create(order()).await.unwrap();
        let second = services.orders.create(order()).await.unwrap();
        let custom = add_user(&pool, "custom", "custom").await;
        services.access.login(login("custom")).await.unwrap();
        let grant = services
            .access
            .temporary(loan(
                "admin",
                AccessScope::Repairs,
                Some(first.id),
                "repairs.view",
            ))
            .await
            .unwrap();
        assert_eq!(grant.recipient_id, custom);
        assert!(
            services
                .orders
                .get(GetOrderPayload {
                    include_deleted: false,
                    order_id: first.id,
                    scope: AccessScope::Repairs,
                    include_device_password: true
                })
                .await
                .is_ok()
        );
        assert!(matches!(
            services
                .orders
                .get(GetOrderPayload {
                    include_deleted: false,
                    order_id: second.id,
                    scope: AccessScope::Repairs,
                    include_device_password: true
                })
                .await,
            Err(AppError::Forbidden)
        ));
        sqlx::query("INSERT INTO usuarios(nombre,nombre_acceso,hash_contrasena,id_nivel,id_rol) SELECT 'Otro admin','other',hash_contrasena,id_nivel,id_rol FROM usuarios WHERE id=1").execute(&pool).await.unwrap();
        sqlx::query("UPDATE usuarios SET activo=0 WHERE id=1")
            .execute(&pool)
            .await
            .unwrap();
        assert!(matches!(
            services
                .orders
                .get(GetOrderPayload {
                    include_deleted: false,
                    order_id: first.id,
                    scope: AccessScope::Repairs,
                    include_device_password: false
                })
                .await,
            Err(AppError::Forbidden)
        ));
    });
}

#[test]
fn grant_and_use_audit_failures_do_not_elevate_or_save_an_order() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        add_user(&pool, "custom", "custom").await;
        services.access.login(login("custom")).await.unwrap();
        sqlx::raw_sql("CREATE TRIGGER fail_access_audit BEFORE INSERT ON auditoria WHEN NEW.entidad='accesos_temporales' BEGIN SELECT RAISE(ABORT,'prueba'); END;").execute(&pool).await.unwrap();
        assert!(
            services
                .access
                .temporary(loan("admin", AccessScope::Clients, None, "orders.create"))
                .await
                .is_err()
        );
        assert_eq!(count(&pool, "accesos_temporales").await, 0);
        assert!(matches!(
            services.orders.create(order()).await,
            Err(AppError::Forbidden)
        ));
        sqlx::raw_sql("DROP TRIGGER fail_access_audit")
            .execute(&pool)
            .await
            .unwrap();
        services
            .access
            .temporary(loan("admin", AccessScope::Clients, None, "orders.create"))
            .await
            .unwrap();
        sqlx::raw_sql("CREATE TRIGGER fail_use_audit BEFORE INSERT ON auditoria WHEN NEW.id_operacion=(SELECT id FROM operaciones_auditoria WHERE codigo='access.temporary.use') BEGIN SELECT RAISE(ABORT,'prueba'); END;").execute(&pool).await.unwrap();
        assert!(services.orders.create(order()).await.is_err());
        assert_eq!(count(&pool, "ordenes").await, 0);
    });
}

#[test]
fn failed_end_audit_removes_memory_access_and_restart_never_restores_a_loan() {
    tauri::async_runtime::block_on(async {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("restart.sqlite");
        let db = Database::open_file(&file).await.unwrap();
        let pool = db.pool().clone();
        let services = BackendServices::new(db, keys()).await.unwrap();
        services.access.initialize(setup()).await.unwrap();
        add_user(&pool, "custom", "custom").await;
        services.access.login(login("custom")).await.unwrap();
        services
            .access
            .temporary(loan("admin", AccessScope::Clients, None, "orders.create"))
            .await
            .unwrap();
        sqlx::raw_sql("CREATE TRIGGER fail_end BEFORE INSERT ON auditoria WHEN NEW.id_operacion=(SELECT id FROM operaciones_auditoria WHERE codigo='access.temporary.end') BEGIN SELECT RAISE(ABORT,'prueba'); END;").execute(&pool).await.unwrap();
        assert!(services.access.finish_temporary().await.is_err());
        assert!(matches!(
            services.orders.create(order()).await,
            Err(AppError::Forbidden)
        ));
        sqlx::raw_sql("DROP TRIGGER fail_end")
            .execute(&pool)
            .await
            .unwrap();
        drop(services);
        pool.close().await;
        let db = Database::open_file(&file).await.unwrap();
        let pool = db.pool().clone();
        let restarted = BackendServices::new(db, keys()).await.unwrap();
        assert!(matches!(
            restarted.orders.create(order()).await,
            Err(AppError::Unauthorized)
        ));
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM accesos_temporales WHERE finalizado_en IS NULL"
            )
            .fetch_one(&pool)
            .await
            .unwrap(),
            0
        );
        drop(restarted);
        pool.close().await;
    });
}

#[test]
fn wrong_approval_preserves_operator_and_five_failures_are_rate_limited() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        let limited = add_user(&pool, "limited", "limited").await;
        services.access.login(login("limited")).await.unwrap();
        for _ in 0..5 {
            let mut request = loan("admin", AccessScope::Clients, None, "orders.edit");
            request.password = "wrong-test-password".into();
            assert!(matches!(
                services.access.temporary(request).await,
                Err(AppError::InvalidCredentials)
            ));
        }
        assert!(matches!(
            services
                .access
                .temporary(loan("admin", AccessScope::Clients, None, "orders.edit"))
                .await,
            Err(AppError::RateLimited)
        ));
        let order = services.orders.create(order()).await.unwrap();
        assert_eq!(order.received_by, limited);
    });
}

#[test]
fn concurrent_receipts_use_distinct_folios_and_audit_authorship() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        let services = Arc::new(services);
        let first = services.clone();
        let second = services.clone();
        let one = tauri::async_runtime::spawn(async move { first.orders.create(order()).await });
        let two = tauri::async_runtime::spawn(async move { second.orders.create(order()).await });
        let one = one.await.unwrap().unwrap();
        let two = two.await.unwrap().unwrap();
        assert_ne!(one.folio, two.folio);
        assert_eq!(count(&pool, "ordenes").await, 2);
        let creators:i64=sqlx::query_scalar("SELECT count(*) FROM auditoria a JOIN operaciones_auditoria o ON o.id=a.id_operacion WHERE o.codigo='orders.create' AND a.id_autor=1").fetch_one(&pool).await.unwrap();
        assert_eq!(creators, 2);
    });
}

#[test]
fn crypto_and_financial_retry_survive_backend_reopen_with_the_same_external_key() {
    tauri::async_runtime::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("key-reopen.sqlite");
        let db = Database::open_file(&file).await.unwrap();
        let pool = db.pool().clone();
        let backend = BackendServices::new(db, keys()).await.unwrap();
        backend.access.initialize(setup()).await.unwrap();
        let payload = || {
            let mut p = order();
            p.initial_payment = Some(InitialPaymentInput {
                request_id: "reopen-financial-retry".into(),
                amount_minor: 30000,
                method: "Libre".into(),
            });
            p
        };
        let created = backend.orders.create(payload()).await.unwrap();
        drop(backend);
        pool.close().await;
        let db = Database::open_file(&file).await.unwrap();
        let pool = db.pool().clone();
        let backend = BackendServices::new(db, keys()).await.unwrap();
        backend.access.login(login("admin")).await.unwrap();
        let loaded = backend.orders.get(get(created.id, true)).await.unwrap();
        assert_eq!(
            loaded.device_password.unwrap().0.as_str(),
            "test-device-PIN-2048"
        );
        assert_eq!(
            backend.orders.create(payload()).await.unwrap().id,
            created.id
        );
        assert_eq!(count(&pool, "abonos").await, 1);
        drop(backend);
        pool.close().await;
    });
}

#[cfg(windows)]
#[test]
fn windows_key_custody_reopens_dpapi_key_without_storing_it_in_sqlite() {
    use super::key_store::OsKeyStore;
    let directory = tempfile::tempdir().unwrap();
    let provider = OsKeyStore::open(directory.path()).unwrap();
    let crypto = CryptoService::new(Arc::new(provider));
    let encrypted = crypto.encrypt(12, "os-key-test-only").unwrap();
    drop(crypto);
    let provider = OsKeyStore::open(directory.path()).unwrap();
    let crypto = CryptoService::new(Arc::new(provider));
    assert_eq!(
        crypto.decrypt(&encrypted).unwrap().0.as_str(),
        "os-key-test-only"
    );
}

#[cfg(windows)]
#[test]
fn windows_startup_requires_the_original_key_when_encrypted_credentials_exist() {
    tauri::async_runtime::block_on(async {
        let directory = tempfile::tempdir().unwrap();
        let database_path = directory.path().join("hrl_core_3fn.db");
        let key_directory = directory.path().join("keys");
        let key_path = key_directory.join("device-password-v1.dpapi");
        let db = Database::open_file(&database_path).await.unwrap();
        let pool = db.pool().clone();
        let services = BackendServices::with_os_key_store(db, &key_directory)
            .await
            .unwrap();
        services.access.initialize(setup()).await.unwrap();
        let created = services.orders.create(order()).await.unwrap();
        let audits = count(&pool, "auditoria").await;
        let protected_key = std::fs::read(&key_path).unwrap();
        drop(services);
        pool.close().await;

        std::fs::remove_file(&key_path).unwrap();
        let db = Database::open_file(&database_path).await.unwrap();
        assert!(matches!(
            BackendServices::with_os_key_store(db, &key_directory).await,
            Err(AppError::Crypto)
        ));
        assert!(!key_path.exists());

        let corrupt_key = b"custodia-corrupta-de-prueba";
        std::fs::write(&key_path, corrupt_key).unwrap();
        let db = Database::open_file(&database_path).await.unwrap();
        assert!(matches!(
            BackendServices::with_os_key_store(db, &key_directory).await,
            Err(AppError::Crypto)
        ));
        assert_eq!(std::fs::read(&key_path).unwrap(), corrupt_key);

        std::fs::write(&key_path, &protected_key).unwrap();
        let db = Database::open_file(&database_path).await.unwrap();
        let pool = db.pool().clone();
        let services = BackendServices::with_os_key_store(db, &key_directory)
            .await
            .unwrap();
        assert_eq!(count(&pool, "auditoria").await, audits);
        services.access.login(login("admin")).await.unwrap();
        let restored = services.orders.get(get(created.id, true)).await.unwrap();
        assert_eq!(
            restored.device_password.unwrap().0.as_str(),
            "test-device-PIN-2048"
        );
        assert_eq!(std::fs::read(&key_path).unwrap(), protected_key);
    });
}

fn recovery_payload(token: &str) -> VerifyRecoveryPayload {
    VerifyRecoveryPayload {
        challenge: token.into(),
        signature: super::recovery::sign_test_challenge(token),
        new_master_password: "new-master-test-456".into(),
        new_admin_password: "new-admin-test-456".into(),
    }
}

#[test]
fn recovery_resets_both_hashes_revokes_access_and_keeps_device_encryption_key() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        let created = services.orders.create(order()).await.unwrap();
        let recipient = add_user(&pool, "recipient", "custom").await;
        services.access.login(login("recipient")).await.unwrap();
        let grant = services
            .access
            .temporary(loan("admin", AccessScope::Clients, None, "orders.create"))
            .await
            .unwrap();
        let challenge = services.access.generate_recovery_challenge().await.unwrap();
        assert_eq!(challenge.admin_id, 1);
        assert_eq!(challenge.admin_username, "admin");
        assert_eq!(challenge.expires_in_seconds, 900);
        let response = services
            .access
            .verify_recovery_signature(recovery_payload(&challenge.challenge))
            .await
            .unwrap();
        assert!(response.requires_login);
        assert_eq!(response.admin_id, 1);
        assert!(matches!(
            services.orders.get(get(created.id, false)).await,
            Err(AppError::Unauthorized)
        ));
        assert!(
            services
                .access
                .verify_recovery_signature(recovery_payload(&challenge.challenge))
                .await
                .is_err()
        );
        let closed: bool = sqlx::query_scalar(
            "SELECT finalizado_en IS NOT NULL FROM accesos_temporales WHERE id=?",
        )
        .bind(grant.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(closed);
        assert!(matches!(
            services.access.login(login("admin")).await,
            Err(AppError::InvalidCredentials)
        ));
        services
            .access
            .login(LoginPayload {
                username: "admin".into(),
                password: "new-admin-test-456".into(),
            })
            .await
            .unwrap();
        let encoded: String = sqlx::query_scalar("SELECT hash FROM claves_maestras WHERE id=1")
            .fetch_one(&pool)
            .await
            .unwrap();
        use argon2::{Argon2, PasswordVerifier, password_hash::PasswordHash};
        let parsed = PasswordHash::new(&encoded).unwrap();
        assert!(
            Argon2::default()
                .verify_password(b"new-master-test-456", &parsed)
                .is_ok()
        );
        assert!(
            Argon2::default()
                .verify_password(b"master-test-123", &parsed)
                .is_err()
        );
        let loaded = services.orders.get(get(created.id, true)).await.unwrap();
        assert_eq!(
            loaded.device_password.unwrap().0.as_str(),
            "test-device-PIN-2048"
        );
        let audit_count: i64 = sqlx::query_scalar("SELECT count(*) FROM auditoria WHERE motivo LIKE 'Recuperación autorizada%' AND id_autor=1").fetch_one(&pool).await.unwrap();
        assert_eq!(audit_count, 2);
        let details: Vec<String> =
            sqlx::query_scalar("SELECT coalesce(valor_nuevo,'') FROM auditoria_detalle")
                .fetch_all(&pool)
                .await
                .unwrap();
        let text = details.join(" ");
        for secret in [
            "new-master-test-456",
            "new-admin-test-456",
            "test-device-PIN-2048",
            &encoded,
        ] {
            assert!(!text.contains(secret));
        }
        let role: String = sqlx::query_scalar(
            "SELECT n.codigo FROM usuarios u JOIN niveles_acceso n ON n.id=u.id_nivel WHERE u.id=?",
        )
        .bind(recipient)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(role, "custom");
    });
}

#[test]
fn recovery_audit_failure_rolls_back_credentials_and_loan_closure_and_consumes_signature() {
    tauri::async_runtime::block_on(async {
        for table in ["auditoria", "auditoria_detalle"] {
            let (pool, services) = fixture().await;
            add_user(&pool, "recipient", "custom").await;
            services.access.login(login("recipient")).await.unwrap();
            let grant = services
                .access
                .temporary(loan("admin", AccessScope::Clients, None, "orders.create"))
                .await
                .unwrap();
            let challenge = services.access.generate_recovery_challenge().await.unwrap();
            let before: (String, String) = sqlx::query_as(
                "SELECT m.hash,u.hash_contrasena FROM claves_maestras m JOIN usuarios u ON u.id=1",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let audits_before = count(&pool, "auditoria").await;
            sqlx::raw_sql(&format!("CREATE TRIGGER fail_recovery BEFORE INSERT ON {table} BEGIN SELECT RAISE(ABORT,'prueba rollback recuperación'); END;")).execute(&pool).await.unwrap();
            assert!(
                services
                    .access
                    .verify_recovery_signature(recovery_payload(&challenge.challenge))
                    .await
                    .is_err()
            );
            let after: (String, String) = sqlx::query_as(
                "SELECT m.hash,u.hash_contrasena FROM claves_maestras m JOIN usuarios u ON u.id=1",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(before, after);
            assert_eq!(count(&pool, "auditoria").await, audits_before);
            let open: bool = sqlx::query_scalar(
                "SELECT finalizado_en IS NULL FROM accesos_temporales WHERE id=?",
            )
            .bind(grant.id)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(open);
            assert!(matches!(
                services
                    .access
                    .verify_recovery_signature(recovery_payload(&challenge.challenge))
                    .await,
                Err(AppError::RecoveryRejected)
            ));
        }
    });
}

#[test]
fn recovery_is_available_without_login_but_only_after_setup_and_not_after_restart() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = empty().await;
        assert!(matches!(
            services.access.generate_recovery_challenge().await,
            Err(AppError::RecoveryUnavailable)
        ));
        services.access.initialize(setup()).await.unwrap();
        services.access.logout().await.unwrap();
        let challenge = services.access.generate_recovery_challenge().await.unwrap();
        let db = Database::in_memory().await.unwrap();
        let restarted = BackendServices::new(db, keys()).await.unwrap();
        restarted.access.initialize(setup()).await.unwrap();
        assert!(matches!(
            restarted
                .access
                .verify_recovery_signature(recovery_payload(&challenge.challenge))
                .await,
            Err(AppError::RecoveryRejected)
        ));
        let before: String = sqlx::query_scalar("SELECT hash FROM claves_maestras")
            .fetch_one(&pool)
            .await
            .unwrap();
        let mut invalid = recovery_payload(&challenge.challenge);
        invalid.signature = "00".repeat(64);
        assert!(matches!(
            services.access.verify_recovery_signature(invalid).await,
            Err(AppError::RecoveryRejected)
        ));
        let after: String = sqlx::query_scalar("SELECT hash FROM claves_maestras")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(before, after);
        services
            .access
            .verify_recovery_signature(recovery_payload(&challenge.challenge))
            .await
            .unwrap();
    });
}

#[test]
fn recovery_rejects_changed_accounts_and_weak_replacement_passwords() {
    tauri::async_runtime::block_on(async {
        let (pool, services) = fixture().await;
        let challenge = services.access.generate_recovery_challenge().await.unwrap();
        let mut invalid = recovery_payload(&challenge.challenge);
        invalid.new_master_password = "short".into();
        assert!(matches!(
            services.access.verify_recovery_signature(invalid).await,
            Err(AppError::Validation(_))
        ));
        let before: String = sqlx::query_scalar("SELECT hash FROM claves_maestras")
            .fetch_one(&pool)
            .await
            .unwrap();
        sqlx::query("UPDATE usuarios SET nombre_acceso='renamed-admin' WHERE id=1")
            .execute(&pool)
            .await
            .unwrap();
        assert!(matches!(
            services
                .access
                .verify_recovery_signature(recovery_payload(&challenge.challenge))
                .await,
            Err(AppError::RecoveryRejected)
        ));
        let after: String = sqlx::query_scalar("SELECT hash FROM claves_maestras")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(before, after);
    });
}
