use crate::{access::AccessState, orders::*};
use sqlx::SqlitePool;

async fn migrate(pool: &SqlitePool) {
    for schema in [
        include_str!("../migrations/001_access.sql"),
        include_str!("../migrations/002_workshop.sql"),
        include_str!("../migrations/003_authentication.sql"),
        include_str!("../migrations/004_order_records.sql"),
    ] {
        sqlx::raw_sql(schema).execute(pool).await.unwrap();
    }
}
fn order() -> Order {
    serde_json::from_value(serde_json::json!({
        "id":"ORD-001", "client":"Cliente", "phone":"9980000000", "received":"27/10/2026", "status":"Diagnóstico",
        "serial":"SERIE", "item":"Laptop", "brand":"HP", "model":"14", "otherShop":false, "otherShopReason":"", "dropped":false, "dropReason":"",
        "password":"equipment-secret-test-only", "work":"Diagnóstico", "cost":"$1,500.00", "deposit":"$500.00", "notes":"Nota", "paid":false, "delivered":false
    })).unwrap()
}

#[test]
fn order_and_audit_survive_reopen_and_reject_stale_updates() {
    tauri::async_runtime::block_on(async {
        let directory =
            std::env::temp_dir().join(format!("hrl-orders-{}", crate::access::random_id()));
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
        let state = AccessState::test_admin(&pool).await;
        let mut original = order();
        original.currency = "USD".into();
        save(&pool, &state, original.clone(), None).await.unwrap();
        assert!(save(&pool, &state, original.clone(), None).await.is_err());
        pool.close().await;
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        let state = AccessState::default();
        assert!(list(&pool, &state).await.is_err());
        state.test_login(&pool, "Test admin").await;
        assert_eq!(
            list(&pool, &state).await.unwrap().orders,
            vec![original.clone()]
        );
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT currency FROM orders")
                .fetch_one(&pool)
                .await
                .unwrap(),
            "USD"
        );
        let mut other_currency = original.clone();
        other_currency.currency = "EUR".into();
        assert!(
            save(&pool, &state, other_currency, Some(original.clone()))
                .await
                .is_err()
        );
        let mut changed = original.clone();
        changed.work = "Reparación".into();
        save(&pool, &state, changed.clone(), Some(original.clone()))
            .await
            .unwrap();
        assert!(
            save(&pool, &state, original.clone(), Some(original.clone()))
                .await
                .is_err()
        );
        assert_eq!(list(&pool, &state).await.unwrap().orders, vec![changed]);
        let history: String = sqlx::query_scalar(
            "SELECT group_concat(coalesce(old_value,'') || new_value) FROM audit_events",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(history.contains("Diagnóstico"));
        assert!(history.contains("Reparación"));
        assert!(!history.contains("equipment-secret"));
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT amount_minor FROM payments")
                .fetch_one(&pool)
                .await
                .unwrap(),
            50000
        );
        pool.close().await;
        std::fs::remove_dir_all(directory).unwrap();
    });
}

#[test]
fn failed_audit_or_duplicate_import_leaves_no_partial_records() {
    tauri::async_runtime::block_on(async {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        migrate(&pool).await;
        let state = AccessState::test_admin(&pool).await;
        sqlx::raw_sql("CREATE TRIGGER reject_audit BEFORE INSERT ON audit_events BEGIN SELECT RAISE(ABORT,'Test failure'); END").execute(&pool).await.unwrap();
        assert!(save(&pool, &state, order(), None).await.is_err());
        for table in ["orders", "clients", "equipment", "payments", "audit_events"] {
            assert_eq!(
                sqlx::query_scalar::<_, i64>(&format!("SELECT count(*) FROM {table}"))
                    .fetch_one(&pool)
                    .await
                    .unwrap(),
                0,
                "{table}"
            );
        }
        sqlx::raw_sql("DROP TRIGGER reject_audit")
            .execute(&pool)
            .await
            .unwrap();
        assert!(import(&pool, &state, vec![order(), order()]).await.is_err());
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM orders")
                .fetch_one(&pool)
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM legacy_imports")
                .fetch_one(&pool)
                .await
                .unwrap(),
            0
        );
        let mut legacy = order();
        legacy.delivered = true;
        import(&pool, &state, vec![legacy.clone()]).await.unwrap();
        assert_eq!(list(&pool, &state).await.unwrap().orders, vec![legacy]);
        assert!(list(&pool, &state).await.unwrap().legacy_imported);
        assert!(import(&pool, &state, vec![order()]).await.is_err());
        assert_eq!(
            sqlx::query_scalar::<_, Option<String>>("SELECT delivered_at FROM orders")
                .fetch_one(&pool)
                .await
                .unwrap(),
            None,
            "Do not invent historical delivery dates"
        );
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT occurred_at FROM payments")
                .fetch_one(&pool)
                .await
                .unwrap(),
            "",
            "Do not invent historical payment dates"
        );
        pool.close().await;
    });
}

#[test]
fn limited_access_can_create_without_payment_but_cannot_edit_or_import() {
    tauri::async_runtime::block_on(async {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        migrate(&pool).await;
        let state = AccessState::test_admin(&pool).await;
        crate::users::save(
            &pool,
            &state,
            crate::users::UserInput {
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
        state.test_login(&pool, "Limited").await;
        assert!(save(&pool, &state, order(), None).await.is_err());
        let mut clean = order();
        clean.deposit = "$0.00".into();
        save(&pool, &state, clean.clone(), None).await.unwrap();
        assert_eq!(list(&pool, &state).await.unwrap().orders.len(), 1);
        assert!(
            save(&pool, &state, clean.clone(), Some(clean))
                .await
                .is_err()
        );
        assert!(import(&pool, &state, vec![]).await.is_err());
        pool.close().await;
    });
}

#[test]
fn integer_money_parser_preserves_decimal_formats() {
    for (text, expected) in [
        ("$1,500.25", 150025),
        ("1.500,25", 150025),
        ("0", 0),
        (".50", 50),
        ("1,500", 150000),
        ("10,5", 1050),
    ] {
        assert_eq!(money_minor(text).unwrap(), expected);
    }
    for text in ["", "invalid", "-1", "9999999999999999999999999999"] {
        assert!(money_minor(text).is_err());
    }
}
