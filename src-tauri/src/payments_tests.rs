use super::*;
use crate::{orders, users};

async fn fixture() -> (SqlitePool, AccessState, orders::Order) {
    fixture_with(sqlx::sqlite::SqliteConnectOptions::new().in_memory(true).foreign_keys(true)).await
}
async fn fixture_with(options:sqlx::sqlite::SqliteConnectOptions) -> (SqlitePool, AccessState, orders::Order) {
    let pool=sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    for sql in [include_str!("../migrations/001_access.sql"),include_str!("../migrations/002_workshop.sql"),include_str!("../migrations/003_authentication.sql"),include_str!("../migrations/004_order_records.sql"),include_str!("../migrations/005_order_visibility.sql"),include_str!("../migrations/006_repair_records.sql"),include_str!("../migrations/007_logbook.sql"),include_str!("../migrations/008_financial_logbook.sql"),include_str!("../migrations/009_company_settings.sql"),include_str!("../migrations/010_payment_adjustments.sql")] {
        sqlx::raw_sql(sql).execute(&pool).await.unwrap();
    }
    let state=AccessState::test_admin(&pool).await;
    let order=serde_json::from_value(serde_json::json!({"id":"MONEY-1","client":"Prueba","phone":"123","received":"2026-10-28","status":"Recibido","serial":"","item":"Laptop","brand":"","model":"","otherShop":false,"otherShopReason":"","dropped":false,"dropReason":"","password":"","work":"Prueba","cost":"750","deposit":"0","notes":"","paid":false,"delivered":false})).unwrap();
    let order=orders::save(&pool,&state,order,None).await.unwrap();
    (pool,state,order)
}
fn request(kind:&str, amount:i64, expected:i64) -> MoneyRequest {
    MoneyRequest {request_id:access::random_id(),order_id:"MONEY-1".into(),kind:kind.into(),payment_id:None,amount_minor:amount,expected_minor:expected,method:"Transferencia de prueba".into(),reason:"Motivo de prueba".into()}
}

#[test]
fn credit_refund_and_retry_preserve_originals_and_reject_partial_or_stale_refunds() {
    tauri::async_runtime::block_on(async {
        let (pool,state,original)=fixture().await;
        let receipt=request("receipt",75000,0);
        record(&pool,&state,receipt.clone()).await.unwrap();
        assert_eq!(record(&pool,&state,receipt.clone()).await.unwrap().collected_minor,75000);
        let current:orders::Order=serde_json::from_value(serde_json::to_value(orders::list(&pool,&state).await.unwrap()).unwrap()["orders"][0].clone()).unwrap();
        let mut cheaper=serde_json::to_value(&current).unwrap();cheaper["cost"]=serde_json::json!("600");
        let cheaper=orders::save(&pool,&state,serde_json::from_value(cheaper).unwrap(),Some(current)).await.unwrap();
        let credit=orders::payment_state(&mut *pool.acquire().await.unwrap(),"MONEY-1").await.unwrap();
        assert_eq!(credit.credit_minor,15000);assert_eq!(credit.refunded_minor,0);
        assert!(record(&pool,&state,request("refund",5000,75000)).await.is_err());
        let refund=request("refund",15000,75000);
        let result=record(&pool,&state,refund.clone()).await.unwrap();
        assert_eq!(result.collected_minor,60000);assert_eq!(result.credit_minor,0);assert_eq!(result.refunded_minor,15000);assert!(result.paid);
        record(&pool,&state,refund.clone()).await.unwrap();
        assert!(record(&pool,&state,request("refund",15000,75000)).await.is_err());
        let mut conflicting=refund;conflicting.reason="Otra solicitud".into();assert!(record(&pool,&state,conflicting).await.is_err());
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM payment_operations").fetch_one(&pool).await.unwrap(),1);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT amount_minor FROM payments").fetch_one(&pool).await.unwrap(),75000);
        assert!(sqlx::query("UPDATE payments SET amount_minor=1").execute(&pool).await.is_err());
        assert!(sqlx::query("DELETE FROM payment_operations").execute(&pool).await.is_err());
        let finances=serde_json::to_value(crate::finances::list(&pool,&state).await.unwrap()).unwrap();
        assert_eq!(finances[0]["collectedMinor"],60000);assert_eq!(finances[0]["refundedMinor"],15000);
        let mut special=serde_json::to_value(&cheaper).unwrap();special["status"]=serde_json::json!("Cancelada");
        orders::save(&pool,&state,serde_json::from_value(special).unwrap(),Some(cheaper)).await.unwrap();
        assert_eq!(orders::payment_state(&mut *pool.acquire().await.unwrap(),"MONEY-1").await.unwrap().collected_minor,60000);
        let _=original;
    });
}

#[test]
fn corrections_are_linked_compensating_entries_with_admin_only_access_and_atomic_audit() {
    tauri::async_runtime::block_on(async {
        let (pool,state,_)=fixture().await;
        let receipt=request("receipt",50000,0);record(&pool,&state,receipt.clone()).await.unwrap();
        let mut adjustment=request("adjustment",-20000,50000);adjustment.payment_id=Some(receipt.request_id.clone());adjustment.method.clear();
        let result=record(&pool,&state,adjustment.clone()).await.unwrap();
        assert_eq!(result.collected_minor,30000);assert_eq!(result.adjusted_minor,-20000);assert_eq!(result.refunded_minor,0);
        record(&pool,&state,adjustment).await.unwrap();
        let mut stale=request("adjustment",-1000,50000);stale.payment_id=Some(receipt.request_id.clone());stale.method.clear();assert!(record(&pool,&state,stale).await.is_err());
        let mut excessive=request("adjustment",-30001,30000);excessive.payment_id=Some(receipt.request_id.clone());excessive.method.clear();assert!(record(&pool,&state,excessive).await.is_err());
        sqlx::raw_sql("CREATE TRIGGER fail_money_history BEFORE INSERT ON logbook_events WHEN NEW.source='money' BEGIN SELECT RAISE(ABORT,'test'); END;").execute(&pool).await.unwrap();
        let mut failed=request("adjustment",1000,30000);failed.payment_id=Some(receipt.request_id);failed.method.clear();assert!(record(&pool,&state,failed.clone()).await.is_err());
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM payment_requests WHERE id=?").bind(failed.request_id).fetch_one(&pool).await.unwrap(),0);
        assert_eq!(orders::payment_state(&mut *pool.acquire().await.unwrap(),"MONEY-1").await.unwrap().collected_minor,30000);
    });
}

#[test]
fn limited_and_custom_money_permissions_are_independent_and_revocation_is_immediate() {
    tauri::async_runtime::block_on(async {
        let (pool,state,_)=fixture().await;
        let receipt=request("receipt",75000,0);record(&pool,&state,receipt.clone()).await.unwrap();
        let current:orders::Order=serde_json::from_value(serde_json::to_value(orders::list(&pool,&state).await.unwrap()).unwrap()["orders"][0].clone()).unwrap();
        let mut reduced=serde_json::to_value(&current).unwrap();reduced["cost"]=serde_json::json!("600");orders::save(&pool,&state,serde_json::from_value(reduced).unwrap(),Some(current)).await.unwrap();
        for level in ["limited","custom"] {
            users::save(&pool,&state,users::UserInput{id:None,name:level.into(),password:"test-user-123".into(),confirmation:"test-user-123".into(),access_level:level.into(),active:true,permissions:vec![users::Grant{section:"payments".into(),action:"refund".into()}],master_password:"test-master-123".into()}).await.unwrap();
        }
        for name in ["limited","custom"] {
            state.test_login(&pool,name).await;
            let mut db=pool.acquire().await.unwrap();
            assert!(state.authorize(&mut db,"payments.refund").await.is_ok());
            assert!(state.authorize(&mut db,"payments.add").await.is_err());
            assert!(state.authorize(&mut db,"payments.adjust").await.is_err());drop(db);
            let mut adjustment=request("adjustment",-100,75000);adjustment.payment_id=Some(receipt.request_id.clone());adjustment.method.clear();assert!(record(&pool,&state,adjustment).await.is_err());
        }
        state.test_login(&pool,"limited").await;
        let refund=request("refund",15000,75000);record(&pool,&state,refund.clone()).await.unwrap();
        sqlx::query("DELETE FROM user_permissions WHERE section='payments'").execute(&pool).await.unwrap();
        assert!(record(&pool,&state,refund).await.is_err());
        assert!(record(&pool,&AccessState::default(),request("receipt",1,60000)).await.is_err());
    });
}

#[test]
fn concurrent_receipts_cannot_apply_the_same_balance_twice() {
    tauri::async_runtime::block_on(async {
        let (pool,state,_)=fixture().await;
        let state=std::sync::Arc::new(state);
        let one_pool=pool.clone();let one_state=state.clone();
        let two_pool=pool.clone();let two_state=state.clone();
        let one=tokio::spawn(async move {record(&one_pool,&one_state,request("receipt",10000,0)).await});
        let two=tokio::spawn(async move {record(&two_pool,&two_state,request("receipt",10000,0)).await});
        let results=[one.await.unwrap(),two.await.unwrap()];
        assert_eq!(results.iter().filter(|r|r.is_ok()).count(),1);
        assert_eq!(orders::payment_state(&mut *pool.acquire().await.unwrap(),"MONEY-1").await.unwrap().collected_minor,10000);
    });
}

#[test]
fn adjustments_methods_and_retry_protection_survive_database_reopen() {
    tauri::async_runtime::block_on(async {
        let directory=std::env::temp_dir().join(format!("hrl-money-{}",access::random_id()));
        std::fs::create_dir_all(&directory).unwrap();
        let options=sqlx::sqlite::SqliteConnectOptions::new().filename(directory.join("money.db")).create_if_missing(true).foreign_keys(true);
        let (pool,state,_)=fixture_with(options.clone()).await;
        let receipt=request("receipt",50000,0);record(&pool,&state,receipt.clone()).await.unwrap();
        let mut adjustment=request("adjustment",-20000,50000);adjustment.payment_id=Some(receipt.request_id.clone());adjustment.method.clear();
        record(&pool,&state,adjustment.clone()).await.unwrap();
        pool.close().await;drop(pool);
        let pool=sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
        let state=AccessState::default();assert!(ledger(&pool,&state,"MONEY-1").await.is_err());
        state.test_login(&pool,"Test admin").await;
        let value=ledger(&pool,&state,"MONEY-1").await.unwrap();
        assert_eq!(value.payment.collected_minor,30000);assert_eq!(value.movements.len(),2);
        assert_eq!(value.movements.iter().find(|m|m.kind=="receipt").unwrap().method.as_deref(),Some("Transferencia de prueba"));
        assert_eq!(value.movements.iter().find(|m|m.kind=="adjustment").unwrap().payment_id.as_deref(),Some(receipt.request_id.as_str()));
        record(&pool,&state,adjustment).await.unwrap();
        assert_eq!(ledger(&pool,&state,"MONEY-1").await.unwrap().movements.len(),2);
        pool.close().await;drop(pool);std::fs::remove_dir_all(directory).unwrap();
    });
}
