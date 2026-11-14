use super::{
    tests::{migrate, order},
    *,
};
use crate::{
    access::AccessState,
    users::{self, Grant, UserInput},
};
use sqlx::SqlitePool;

async fn fixture() -> (SqlitePool, AccessState, Order) {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    migrate(&pool).await;
    let state = AccessState::test_admin(&pool).await;
    let order = order();
    save(&pool, &state, order.clone(), None).await.unwrap();
    (pool, state, order)
}

#[test]
fn complete_access_can_delete_and_restore_without_losing_records_or_reusing_folios() {
    tauri::async_runtime::block_on(async {
        let (pool, state, order) = fixture().await;
        set_deleted(&pool, &state, order.clone(), true)
            .await
            .unwrap();
        set_deleted(&pool, &state, order.clone(), true)
            .await
            .unwrap();
        assert!(list(&pool, &state).await.unwrap().orders.is_empty());
        assert_eq!(
            list_view(&pool, &state, true).await.unwrap().orders,
            vec![order.clone()]
        );
        assert!(
            save(&pool, &state, order.clone(), Some(order.clone()))
                .await
                .is_err()
        );
        assert!(save(&pool, &state, order.clone(), None).await.is_err());
        for table in [
            "Ordenes",
            "Equipo",
            "Clientes",
            "Abonos",
            "Historial_ordenes",
            "Historial_papelera",
        ] {
            assert_eq!(
                sqlx::query_scalar::<_, i64>(&format!("SELECT count(*) FROM {table}"))
                    .fetch_one(&pool)
                    .await
                    .unwrap(),
                1,
                "{table}"
            );
        }
        let author: String = sqlx::query_scalar("SELECT id_autor FROM Historial_papelera")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(!author.is_empty());
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT importe_centavos FROM Abonos")
                .fetch_one(&pool)
                .await
                .unwrap(),
            50000
        );
        set_deleted(&pool, &state, order.clone(), false)
            .await
            .unwrap();
        assert_eq!(list(&pool, &state).await.unwrap().orders, vec![order]);
        assert!(
            list_view(&pool, &state, true)
                .await
                .unwrap()
                .orders
                .is_empty()
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM Historial_papelera")
                .fetch_one(&pool)
                .await
                .unwrap(),
            2
        );
        assert!(
            sqlx::query("DELETE FROM Historial_papelera")
                .execute(&pool)
                .await
                .is_err()
        );
        assert!(
            sqlx::query("UPDATE Historial_papelera SET accion='deleted'")
                .execute(&pool)
                .await
                .is_err()
        );
        pool.close().await;
    });
}

#[test]
fn limited_and_revoked_custom_permissions_cannot_delete_or_restore() {
    tauri::async_runtime::block_on(async {
        let (pool, state, order) = fixture().await;
        let account = |name: &str, role: &str, permissions| UserInput {
            id: None,
            name: name.into(),
            password: "test-user-123".into(),
            confirmation: "test-user-123".into(),
            access_level: role.into(),
            active: true,
            permissions,
            master_password: "test-master-123".into(),
        };
        users::save(&pool, &state, account("Limited", "limited", vec![]))
            .await
            .unwrap();
        users::save(
            &pool,
            &state,
            account(
                "Custom",
                "custom",
                vec![
                    Grant {
                        section: "clients".into(),
                        action: "view".into(),
                    },
                    Grant {
                        section: "clients".into(),
                        action: "delete".into(),
                    },
                ],
            ),
        )
        .await
        .unwrap();
        state.test_login(&pool, "Limited").await;
        assert!(
            set_deleted(&pool, &state, order.clone(), true)
                .await
                .is_err()
        );
        assert!(
            set_deleted(&pool, &state, order.clone(), false)
                .await
                .is_err()
        );
        assert!(list_view(&pool, &state, true).await.is_err());
        assert_eq!(list(&pool, &state).await.unwrap().orders.len(), 1);
        state.test_login(&pool, "Custom").await;
        set_deleted(&pool, &state, order.clone(), true)
            .await
            .unwrap();
        sqlx::query("DELETE FROM Permisos_usuarios WHERE accion='delete'")
            .execute(&pool)
            .await
            .unwrap();
        assert!(
            set_deleted(&pool, &state, order.clone(), false)
                .await
                .is_err()
        );
        assert!(list_view(&pool, &state, true).await.is_err());
        state.test_login(&pool, "Test admin").await;
        set_deleted(&pool, &state, order, false).await.unwrap();
        pool.close().await;
    });
}

#[test]
fn stale_order_or_failed_history_cannot_change_visibility() {
    tauri::async_runtime::block_on(async {
        let (pool, state, original) = fixture().await;
        let mut changed = original.clone();
        changed.work = "Trabajo actualizado".into();
        save(&pool, &state, changed.clone(), Some(original.clone()))
            .await
            .unwrap();
        assert!(set_deleted(&pool, &state, original, true).await.is_err());
        sqlx::raw_sql("CREATE TRIGGER reject_visibility BEFORE INSERT ON Historial_papelera BEGIN SELECT RAISE(ABORT,'Test failure'); END").execute(&pool).await.unwrap();
        assert!(
            set_deleted(&pool, &state, changed.clone(), true)
                .await
                .is_err()
        );
        assert_eq!(
            list(&pool, &state).await.unwrap().orders,
            vec![changed.clone()]
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM Historial_papelera")
                .fetch_one(&pool)
                .await
                .unwrap(),
            0
        );
        assert!(
            set_deleted(&pool, &AccessState::default(), changed, true)
                .await
                .is_err()
        );
        pool.close().await;
    });
}

#[test]
fn deleted_orders_remain_recoverable_after_database_reopen() {
    tauri::async_runtime::block_on(async {
        let directory =
            std::env::temp_dir().join(format!("hrl-trash-{}", crate::access::random_id()));
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
        let order = order();
        save(&pool, &state, order.clone(), None).await.unwrap();
        set_deleted(&pool, &state, order.clone(), true)
            .await
            .unwrap();
        pool.close().await;
        drop(pool);
        let reopened = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        let state = AccessState::default();
        state.test_login(&reopened, "Test admin").await;
        assert!(list(&reopened, &state).await.unwrap().orders.is_empty());
        assert_eq!(
            list_view(&reopened, &state, true).await.unwrap().orders,
            vec![order.clone()]
        );
        set_deleted(&reopened, &state, order.clone(), false)
            .await
            .unwrap();
        assert_eq!(list(&reopened, &state).await.unwrap().orders, vec![order]);
        reopened.close().await;
        drop(reopened);
        std::fs::remove_dir_all(directory).unwrap();
    });
}
