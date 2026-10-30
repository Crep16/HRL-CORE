use super::*;
use crate::{
    orders,
    users::{self, UserInput},
};

async fn migrate(pool: &SqlitePool) {
    for sql in [
        include_str!("../migrations/001_access.sql"),
        include_str!("../migrations/002_workshop.sql"),
        include_str!("../migrations/003_authentication.sql"),
        include_str!("../migrations/004_order_records.sql"),
        include_str!("../migrations/005_order_visibility.sql"),
        include_str!("../migrations/006_repair_records.sql"),
        include_str!("../migrations/007_logbook.sql"),
        include_str!("../migrations/008_financial_logbook.sql"),
        include_str!("../migrations/010_payment_adjustments.sql"),
    ] {
        sqlx::raw_sql(sql).execute(pool).await.unwrap();
    }
}
async fn seed(pool: &SqlitePool) -> AccessState {
    let state = AccessState::test_admin(pool).await;
    let order=serde_json::from_value(serde_json::json!({"id":"TEST-1","client":"Test client","phone":"123","received":"2026-10-27","status":"Diagnóstico","serial":"","item":"Laptop","brand":"","model":"","otherShop":false,"otherShopReason":"","dropped":false,"dropReason":"","password":"","work":"","cost":"1500","deposit":"500","depositMethod":"Efectivo de prueba","notes":"","paid":false,"delivered":false})).unwrap();
    orders::save(pool, &state, order, None).await.unwrap();
    users::save(
        pool,
        &state,
        UserInput {
            id: None,
            name: "Limited".into(),
            password: "test-user-123".into(),
            confirmation: "test-user-123".into(),
            access_level: "limited".into(),
            active: true,
            permissions: vec![],
            master_password: "test-master-123".into(),
        },
    )
    .await
    .unwrap();
    state
}
async fn fixture() -> (SqlitePool, AccessState) {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    migrate(&pool).await;
    let state = seed(&pool).await;
    (pool, state)
}
fn appended() -> Repair {
    let mut repair = Repair::default();
    repair.completed = 1;
    repair.public_note = "Public test note".into();
    repair.internal_note = "Private test note".into();
    repair.events.push(Event {
        id: 1,
        date: "2026-10-27T12:00".into(),
        title: "Avance".into(),
        detail: String::new(),
        stage: "Recibido".into(),
        public: true,
    });
    repair.warranties.push(Warranty {
        name: "Panel".into(),
        days: 30,
    });
    repair.parts.push(Part {
        name: "Panel".into(),
        price: 150.25,
    });
    repair
}
#[test]
fn dynamic_stages_are_saved_reusable_and_preserve_existing_progress() {
    tauri::async_runtime::block_on(async {
        let (pool, state) = fixture().await;
        state.test_login(&pool, "Limited").await;
        let fresh = load(&pool, &state, "TEST-1", None).await.unwrap();
        assert_eq!(fresh.repair.stages, ["Recibido", "Entregado"]);
        let mut next = appended();
        next.stages.insert(1, "Limpieza de placa".into());
        next.events[0].stage = "Limpieza de placa".into();
        save(&pool, &state, "TEST-1", next.clone(), 0).await.unwrap();
        assert_eq!(load(&pool, &state, "TEST-1", None).await.unwrap().repair, next);
        assert!(stage_options(&pool, &state).await.unwrap().contains(&"Limpieza de placa".into()));
        let mut invalid = next.clone();
        invalid.stages.swap(0, 1);
        assert!(save(&pool, &state, "TEST-1", invalid, 1).await.is_err());
        let mut invalid = next.clone();
        invalid.stages = vec!["Otro".into(), "Entregado".into()];
        assert!(save(&pool, &state, "TEST-1", invalid, 1).await.is_err());
        assert_eq!(load(&pool, &state, "TEST-1", None).await.unwrap().repair, next);
        let mut legacy = serde_json::to_value(&next).unwrap();
        legacy.as_object_mut().unwrap().remove("stages");
        legacy["completed"] = serde_json::json!(5);
        let legacy: Repair = serde_json::from_value(legacy).unwrap();
        assert_eq!(legacy.stages, legacy_stages());
        assert_eq!(legacy.completed, 5);
    });
}
#[test]
fn limited_can_read_and_append_technical_work_but_cannot_edit_or_collect_money() {
    tauri::async_runtime::block_on(async {
        let (pool, state) = fixture().await;
        state.test_login(&pool, "Limited").await;
        assert_eq!(
            load(&pool, &state, "TEST-1", None).await.unwrap().revision,
            0
        );
        let snapshot = save(&pool, &state, "TEST-1", appended(), 0).await.unwrap();
        assert_eq!(snapshot.revision, 1);
        let mut attempts = Vec::new();
        let mut next = snapshot.repair.clone();
        next.public_note = "Replacement".into();
        attempts.push(next);
        let mut next = snapshot.repair.clone();
        next.technician = "Changed".into();
        attempts.push(next);
        let mut next = snapshot.repair.clone();
        next.completed = 0;
        attempts.push(next);
        let mut next = snapshot.repair.clone();
        next.events.clear();
        attempts.push(next);
        let mut next = snapshot.repair.clone();
        next.parts[0].price = 1.;
        attempts.push(next);
        let mut next = snapshot.repair.clone();
        next.warranties.clear();
        attempts.push(next);
        let mut next = snapshot.repair.clone();
        next.payments.push(Payment {
            method: Some("Efectivo de prueba".into()),
            date: String::new(),
            amount: 100.,
        });
        attempts.push(next);
        let mut next = snapshot.repair.clone();
        next.visibility[0].enabled = false;
        attempts.push(next);
        let mut next = snapshot.repair.clone();
        next.sync_mode = "auto".into();
        attempts.push(next);
        for next in attempts {
            assert!(save(&pool, &state, "TEST-1", next, 1).await.is_err());
        }
        let mut next = snapshot.repair.clone();
        next.internal_note.push_str("\nAnother note");
        assert!(
            save(&pool, &state, "TEST-1", next.clone(), 0)
                .await
                .is_err()
        );
        save(&pool, &state, "TEST-1", next, 1).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM payments")
                .fetch_one(&pool)
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM repair_notes")
                .fetch_one(&pool)
                .await
                .unwrap(),
            3
        );
        let authors:i64=sqlx::query_scalar("SELECT count(*) FROM repair_changes JOIN users ON users.id=author_id WHERE users.name='Limited'").fetch_one(&pool).await.unwrap();
        assert!(authors >= 5);
        let public =
            serde_json::to_string(&public_summary(&pool, &state, "TEST-1").await.unwrap()).unwrap();
        assert!(public.contains("Public test note"));
        assert!(!public.contains("Private test note"));
        assert!(!public.contains("internalNote"));
        sqlx::query("UPDATE users SET active=0 WHERE name='Limited'")
            .execute(&pool)
            .await
            .unwrap();
        assert!(load(&pool, &state, "TEST-1", None).await.is_err());
        assert!(
            load(&pool, &AccessState::default(), "TEST-1", None)
                .await
                .is_err()
        );
        pool.close().await;
    });
}
#[test]
fn legacy_import_requires_admin_and_does_not_duplicate_payments_or_notes() {
    tauri::async_runtime::block_on(async {
        let (pool, state) = fixture().await;
        let mut legacy = appended();
        legacy.payments.push(Payment {
            method: None,
            date: "2026-10-26T11:00".into(),
            amount: 100.,
        });
        state.test_login(&pool, "Limited").await;
        assert!(
            load(&pool, &state, "TEST-1", Some(legacy.clone()))
                .await
                .is_err()
        );
        state.test_login(&pool, "Test admin").await;
        load(&pool, &state, "TEST-1", Some(legacy.clone()))
            .await
            .unwrap();
        load(&pool, &state, "TEST-1", Some(legacy.clone()))
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT sum(amount_minor) FROM payments")
                .fetch_one(&pool)
                .await
                .unwrap(),
            60000
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM repair_notes")
                .fetch_one(&pool)
                .await
                .unwrap(),
            2
        );
        state.test_login(&pool, "Limited").await;
        assert_eq!(
            load(&pool, &state, "TEST-1", None).await.unwrap().repair,
            legacy
        );
        pool.close().await;
    });
}
#[test]
fn repair_history_failure_rolls_back_every_related_record() {
    tauri::async_runtime::block_on(async {
        let (pool, state) = fixture().await;
        sqlx::raw_sql("CREATE TRIGGER reject_repair_change BEFORE INSERT ON repair_changes BEGIN SELECT RAISE(ABORT,'Test failure'); END").execute(&pool).await.unwrap();
        assert!(save(&pool, &state, "TEST-1", appended(), 0).await.is_err());
        for table in [
            "repair_records",
            "repair_notes",
            "repair_events",
            "warranties",
            "order_parts",
            "repair_changes",
        ] {
            assert_eq!(
                sqlx::query_scalar::<_, i64>(&format!("SELECT count(*) FROM {table}"))
                    .fetch_one(&pool)
                    .await
                    .unwrap(),
                0,
                "{table}"
            );
        }
        assert_eq!(
            load(&pool, &state, "TEST-1", None).await.unwrap().revision,
            0
        );
        pool.close().await;
    });
}
#[test]
fn full_access_payments_use_the_same_ledger_and_respect_the_remaining_balance() {
    tauri::async_runtime::block_on(async {
        let (pool, state) = fixture().await;
        let mut repair = Repair::default();
        repair.payments.push(Payment {
            method: Some("Efectivo de prueba".into()),
            date: "2026-10-27T12:00".into(),
            amount: 1000.,
        });
        save(&pool, &state, "TEST-1", repair.clone(), 0)
            .await
            .unwrap();
        repair.payments.push(Payment {
            method: Some("Efectivo de prueba".into()),
            date: String::new(),
            amount: 0.01,
        });
        assert!(save(&pool, &state, "TEST-1", repair, 1).await.is_err());
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT sum(amount_minor) FROM payments")
                .fetch_one(&pool)
                .await
                .unwrap(),
            150000
        );
        pool.close().await;
    });
}
#[test]
fn custom_editor_can_update_existing_notes_without_permission_to_create_events() {
    tauri::async_runtime::block_on(async {
        let (pool, state) = fixture().await;
        save(&pool, &state, "TEST-1", appended(), 0).await.unwrap();
        users::save(
            &pool,
            &state,
            UserInput {
                id: None,
                name: "Editor".into(),
                password: "test-user-123".into(),
                confirmation: "test-user-123".into(),
                access_level: "custom".into(),
                active: true,
                permissions: vec![
                    crate::users::Grant {
                        section: "repairs".into(),
                        action: "view".into(),
                    },
                    crate::users::Grant {
                        section: "repairs".into(),
                        action: "edit".into(),
                    },
                ],
                master_password: "test-master-123".into(),
            },
        )
        .await
        .unwrap();
        state.test_login(&pool, "Editor").await;
        let mut next = appended();
        next.public_note.push_str(" edited");
        save(&pool, &state, "TEST-1", next.clone(), 1)
            .await
            .unwrap();
        next.events.insert(
            0,
            Event {
                id: 2,
                date: String::new(),
                title: "New event".into(),
                detail: String::new(),
                stage: "Diagnóstico".into(),
                public: false,
            },
        );
        assert!(save(&pool, &state, "TEST-1", next, 2).await.is_err());
        pool.close().await;
    });
}

#[test]
fn technical_tracking_survives_database_reopen() {
    tauri::async_runtime::block_on(async {
        let directory = std::env::temp_dir().join(format!("hrl-repair-{}", access::random_id()));
        std::fs::create_dir_all(&directory).unwrap();
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(directory.join("test.db"))
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options.clone())
            .await
            .unwrap();
        migrate(&pool).await;
        let state = seed(&pool).await;
        state.test_login(&pool, "Limited").await;
        save(&pool, &state, "TEST-1", appended(), 0).await.unwrap();
        pool.close().await;
        drop(pool);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        let state = AccessState::default();
        state.test_login(&pool, "Limited").await;
        assert_eq!(
            load(&pool, &state, "TEST-1", None).await.unwrap().repair,
            appended()
        );
        pool.close().await;
        drop(pool);
        std::fs::remove_dir_all(directory).unwrap();
    });
}
