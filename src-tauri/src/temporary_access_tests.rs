use super::*;
use crate::{orders,repairs,users};

async fn fixture()->(SqlitePool,AccessState) {
    let pool=sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    for sql in [include_str!("../migrations/001_access.sql"),include_str!("../migrations/002_workshop.sql"),include_str!("../migrations/003_authentication.sql"),include_str!("../migrations/004_order_records.sql"),include_str!("../migrations/005_order_visibility.sql"),include_str!("../migrations/006_repair_records.sql"),include_str!("../migrations/007_logbook.sql"),include_str!("../migrations/008_financial_logbook.sql"),include_str!("../migrations/009_company_settings.sql"),include_str!("../migrations/010_payment_adjustments.sql"),include_str!("../migrations/011_temporary_access.sql")] {
        sqlx::raw_sql(sql).execute(&pool).await.unwrap();
    }
    let state=AccessState::test_admin(&pool).await;
    for id in ["A","B"] {
        let order=serde_json::from_value(serde_json::json!({"id":id,"client":"Prueba","phone":"123","received":"2026-10-29","status":"Recibido","serial":"","item":"Laptop","brand":"","model":"","otherShop":false,"otherShopReason":"","dropped":false,"dropReason":"","password":"","work":"Prueba","cost":"750","deposit":"0","notes":"","paid":false,"delivered":false})).unwrap();
        orders::save(&pool,&state,order,None).await.unwrap();
    }
    for (name,level,grants) in [("Recipient","limited",vec![]),("Approver","custom",vec![users::Grant{section:"repairs".into(),action:"view".into()},users::Grant{section:"repairs".into(),action:"edit".into()}])] {
        users::save(&pool,&state,users::UserInput{id:None,name:name.into(),password:"test-user-123".into(),confirmation:"test-user-123".into(),access_level:level.into(),active:true,permissions:grants,master_password:"test-master-123".into()}).await.unwrap();
    }
    state.test_login(&pool,"Recipient").await;
    (pool,state)
}
fn approval(name:&str,operation:&str)->ApprovalRequest {
    ApprovalRequest{name:name.into(),password:"test-user-123".into(),scope:"repairs".into(),record_id:Some("A".into()),operation:operation.into(),reason:"Requiere editar el técnico de esta orden".into()}
}
async fn technician(pool:&SqlitePool,state:&AccessState,id:&str,name:&str)->Result<(),String> {
    let snapshot=serde_json::to_value(repairs::load(pool,state,id,None).await?).unwrap();
    let mut repair=snapshot["repair"].clone();repair["technician"]=serde_json::json!(name);
    repairs::save(pool,state,id,serde_json::from_value(repair).unwrap(),snapshot["revision"].as_i64().unwrap()).await?;
    Ok(())
}

#[test]
fn temporary_rights_are_record_scoped_and_keep_recipient_authorship() {
    tauri::async_runtime::block_on(async {
        let (pool,state)=fixture().await;
        assert!(technician(&pool,&state,"A","Ana").await.is_err());
        let grant=state.approve_temporary(&pool,approval("Test admin","repairs.edit")).await.unwrap();
        assert_eq!(state.status(&pool).await.unwrap().user.unwrap().name,"Recipient");
        technician(&pool,&state,"A","Ana").await.unwrap();
        assert!(technician(&pool,&state,"B","Fuera de alcance").await.is_err());
        let mut db=pool.acquire().await.unwrap();
        assert!(state.authorize_scoped(&mut db,"company.edit",None).await.is_err());
        assert!(state.authorize(&mut db,"repairs.edit").await.is_err());drop(db);
        let author:String=sqlx::query_scalar("SELECT u.name FROM repair_changes r JOIN users u ON u.id=r.author_id WHERE r.order_id='A'").fetch_one(&pool).await.unwrap();assert_eq!(author,"Recipient");
        let history:String=sqlx::query_scalar("SELECT group_concat(details_json) FROM system_logbook_events WHERE operation LIKE 'access.temporary.%'").fetch_one(&pool).await.unwrap();
        assert!(history.contains(&grant.id));assert!(history.contains("Test admin"));assert!(history.contains("Recipient"));assert!(!history.contains("test-user-123"));assert!(!history.contains("password_hash"));
        state.end_temporary(&pool).await.unwrap();
        assert!(technician(&pool,&state,"A","Ya terminó").await.is_err());
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM temporary_access_grants WHERE ended_at IS NOT NULL").fetch_one(&pool).await.unwrap(),1);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM system_logbook_events WHERE operation='access.temporary.end'").fetch_one(&pool).await.unwrap(),1);
    });
}

#[test]
fn approver_can_only_lend_current_permanent_permissions_and_revocation_is_rechecked() {
    tauri::async_runtime::block_on(async {
        let (pool,state)=fixture().await;
        assert!(state.approve_temporary(&pool,approval("Approver","payments.refund")).await.is_err());
        state.approve_temporary(&pool,approval("Approver","repairs.edit")).await.unwrap();
        technician(&pool,&state,"A","Autorizado").await.unwrap();
        sqlx::query("DELETE FROM user_permissions WHERE user_id=(SELECT id FROM users WHERE name='Approver') AND action='edit'").execute(&pool).await.unwrap();
        assert!(technician(&pool,&state,"A","Permiso revocado").await.is_err());
        state.end_temporary(&pool).await.unwrap();
        assert!(state.approve_temporary(&pool,approval("Approver","repairs.edit")).await.is_err());
        state.approve_temporary(&pool,approval("Test admin","repairs.edit")).await.unwrap();
        sqlx::query("UPDATE users SET active=0 WHERE name='Recipient'").execute(&pool).await.unwrap();
        assert!(technician(&pool,&state,"A","Usuario inactivo").await.is_err());
    });
}

#[test]
fn incorrect_approval_passwords_do_not_replace_the_session_and_are_rate_limited() {
    tauri::async_runtime::block_on(async {
        let (pool,state)=fixture().await;
        for _ in 0..5 {let mut input=approval("Test admin","repairs.edit");input.password="incorrecta".into();assert!(state.approve_temporary(&pool,input).await.is_err());}
        assert!(state.approve_temporary(&pool,approval("Test admin","repairs.edit")).await.err().unwrap().contains("Espera"));
        assert_eq!(state.status(&pool).await.unwrap().user.unwrap().name,"Recipient");
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM temporary_access_grants").fetch_one(&pool).await.unwrap(),0);
    });
}

#[test]
fn authorization_and_use_fail_closed_if_their_audit_cannot_be_saved() {
    tauri::async_runtime::block_on(async {
        let (pool,state)=fixture().await;
        sqlx::raw_sql("CREATE TRIGGER fail_approval BEFORE INSERT ON system_logbook_events WHEN NEW.operation='access.temporary.start' BEGIN SELECT RAISE(ABORT,'test'); END;").execute(&pool).await.unwrap();
        assert!(state.approve_temporary(&pool,approval("Test admin","repairs.edit")).await.is_err());
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM temporary_access_grants").fetch_one(&pool).await.unwrap(),0);
        assert!(technician(&pool,&state,"A","No concedido").await.is_err());
        sqlx::raw_sql("DROP TRIGGER fail_approval; CREATE TRIGGER fail_use BEFORE INSERT ON system_logbook_events WHEN NEW.operation='access.temporary.use' BEGIN SELECT RAISE(ABORT,'test'); END;").execute(&pool).await.unwrap();
        state.approve_temporary(&pool,approval("Test admin","repairs.edit")).await.unwrap();
        assert!(technician(&pool,&state,"A","No confirmado").await.is_err());
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM repair_records").fetch_one(&pool).await.unwrap(),0);
        sqlx::raw_sql("DROP TRIGGER fail_use; CREATE TRIGGER fail_end BEFORE INSERT ON system_logbook_events WHEN NEW.operation='access.temporary.end' BEGIN SELECT RAISE(ABORT,'test'); END;").execute(&pool).await.unwrap();
        assert!(state.end_temporary(&pool).await.is_err());
        assert!(technician(&pool,&state,"A","Acceso retirado").await.is_err());
        sqlx::raw_sql("DROP TRIGGER fail_end").execute(&pool).await.unwrap();
        state.recover_temporary(&pool).await.unwrap();
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM temporary_access_grants WHERE ended_at IS NULL").fetch_one(&pool).await.unwrap(),0);
    });
}

#[test]
fn switching_session_and_recovering_a_new_app_state_do_not_restore_a_loan() {
    tauri::async_runtime::block_on(async {
        let (pool,state)=fixture().await;
        state.approve_temporary(&pool,approval("Test admin","repairs.edit")).await.unwrap();
        state.test_login(&pool,"Approver").await;
        state.test_login(&pool,"Recipient").await;
        assert!(technician(&pool,&state,"A","Sesión distinta").await.is_err());
        state.approve_temporary(&pool,approval("Test admin","repairs.edit")).await.unwrap();
        let restarted=AccessState::default();restarted.recover_temporary(&pool).await.unwrap();
        restarted.test_login(&pool,"Recipient").await;
        assert!(technician(&pool,&restarted,"A","Reinicio").await.is_err());
        assert!(technician(&pool,&state,"A","Préstamo anterior invalidado").await.is_err());
    });
}

#[test]
fn temporary_refunds_keep_both_identities_and_do_not_duplicate_approval_use_on_retry() {
    tauri::async_runtime::block_on(async {
        let (pool,state)=fixture().await;
        state.test_login(&pool,"Test admin").await;
        let mut request=crate::payments::MoneyRequest{request_id:random_id(),order_id:"A".into(),kind:"receipt".into(),payment_id:None,amount_minor:75000,expected_minor:0,method:"Método libre de prueba".into(),reason:"".into()};
        crate::payments::record(&pool,&state,request.clone()).await.unwrap();
        let order=orders::repair_order(&pool,&state,"A").await.unwrap();
        let mut cheaper=serde_json::to_value(&order).unwrap();cheaper["cost"]=serde_json::json!("600");orders::save(&pool,&state,serde_json::from_value(cheaper).unwrap(),Some(order)).await.unwrap();
        state.test_login(&pool,"Recipient").await;
        assert!(state.authorize(&mut *pool.acquire().await.unwrap(),"payments.refund").await.is_err());
        state.approve_temporary(&pool,approval("Test admin","payments.refund")).await.unwrap();
        request.request_id=random_id();request.kind="refund".into();request.amount_minor=15000;request.expected_minor=75000;request.reason="Devolver todo el excedente".into();
        crate::payments::record(&pool,&state,request.clone()).await.unwrap();crate::payments::record(&pool,&state,request).await.unwrap();
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT count(*) FROM system_logbook_events WHERE operation='access.temporary.use'").fetch_one(&pool).await.unwrap(),1);
        let author:String=sqlx::query_scalar("SELECT u.name FROM payment_operations p JOIN users u ON u.id=p.author_id").fetch_one(&pool).await.unwrap();assert_eq!(author,"Recipient");
        state.end_temporary(&pool).await.unwrap();
        assert!(state.authorize_scoped(&mut *pool.acquire().await.unwrap(),"payments.refund",Some("A")).await.is_err());
    });
}
