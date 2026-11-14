use crate::{access::AccessState, orders::*};
use sqlx::SqlitePool;

pub(super) async fn migrate(pool: &SqlitePool) {
    crate::test_support::migrate_legacy(&pool).await;
}
pub(super) fn order() -> Order {
    serde_json::from_value(serde_json::json!({
        "id":"ORD-001", "client":"Cliente", "phone":"9980000000", "received":"27/10/2026", "status":"Diagnóstico",
        "serial":"SERIE", "item":"Laptop", "brand":"HP", "model":"14", "otherShop":false, "otherShopReason":"", "dropped":false, "dropReason":"",
        "failure":"Falla de prueba", "password":"equipment-secret-test-only", "work":"Diagnóstico", "cost":"$1,500.00", "deposit":"$500.00", "depositMethod":"Efectivo de prueba", "notes":"Nota", "paid":false, "delivered":false
    })).unwrap()
}

#[test]
fn final_repair_payment_updates_clients_and_price_changes_recalculate_paid_without_touching_receipts() {
    tauri::async_runtime::block_on(async {
        let pool=sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        migrate(&pool).await;
        let state=AccessState::test_admin(&pool).await;
        let mut captured=order();captured.cost="750".into();captured.deposit="300".into();
        let before=save(&pool,&state,captured,None).await.unwrap();
        assert!(!before.paid);
        let snapshot=crate::repairs::load(&pool,&state,&before.id,None).await.unwrap();
        let mut repair=serde_json::to_value(snapshot).unwrap()["repair"].clone();
        repair["payments"]=serde_json::json!([{"date":"2026-10-28T13:15","amount":450,"method":"Efectivo de prueba"}]);
        let saved=crate::repairs::save(&pool,&state,&before.id,serde_json::from_value(repair).unwrap(),0).await.unwrap();
        let payment=serde_json::to_value(saved).unwrap()["payment"].clone();
        assert_eq!(payment["collectedMinor"],75000);assert_eq!(payment["totalMinor"],75000);assert_eq!(payment["paid"],true);
        let current=list(&pool,&state).await.unwrap().orders.remove(0);
        assert!(current.paid);assert_eq!(current.deposit,"300");
        assert!(save(&pool,&state,before.clone(),Some(before)).await.is_err());
        let mut edited=current.clone();edited.delivered=true;
        let delivered=save(&pool,&state,edited,Some(current)).await.unwrap();
        assert!(delivered.paid&&delivered.delivered);
        let mut higher=delivered.clone();higher.cost="900".into();
        let higher=save(&pool,&state,higher,Some(delivered)).await.unwrap();
        assert!(!higher.paid);assert!(higher.delivered);
        assert_eq!(list(&pool,&state).await.unwrap().orders[0],higher);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT sum(importe_centavos) FROM Abonos").fetch_one(&pool).await.unwrap(),75000);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM Abonos").fetch_one(&pool).await.unwrap(),2);
        set_deleted(&pool,&state,higher.clone(),true).await.unwrap();
        set_deleted(&pool,&state,higher.clone(),false).await.unwrap();
        assert_eq!(list(&pool,&state).await.unwrap().orders[0],higher);
    });
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
        drop(pool);
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
            sqlx::query_scalar::<_, String>("SELECT moneda FROM Ordenes")
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
            "SELECT group_concat(coalesce(valor_anterior,'') || valor_nuevo) FROM Historial_ordenes",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(history.contains("Diagnóstico"));
        assert!(history.contains("Reparación"));
        assert!(!history.contains("equipment-secret"));
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT importe_centavos FROM Abonos")
                .fetch_one(&pool)
                .await
                .unwrap(),
            50000
        );
        pool.close().await;
        drop(pool);
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
        sqlx::raw_sql("CREATE TRIGGER reject_audit BEFORE INSERT ON Historial_ordenes BEGIN SELECT RAISE(ABORT,'Test failure'); END").execute(&pool).await.unwrap();
        assert!(save(&pool, &state, order(), None).await.is_err());
        for table in ["Ordenes", "Clientes", "Equipo", "Abonos", "Historial_ordenes"] {
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
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM Ordenes")
                .fetch_one(&pool)
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM Importaciones_anteriores")
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
            sqlx::query_scalar::<_, Option<String>>("SELECT fecha_entrega FROM Ordenes")
                .fetch_one(&pool)
                .await
                .unwrap(),
            None,
            "Do not invent historical delivery dates"
        );
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT fecha_evento FROM Abonos")
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
