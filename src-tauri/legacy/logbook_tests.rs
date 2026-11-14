use super::*;
use crate::{orders, repairs, users};

async fn database() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    crate::test_support::migrate_legacy_range(&pool,1,6).await;
    pool
}
fn order() -> orders::Order {
    serde_json::from_value(serde_json::json!({"id":"ORD-TEST","client":"Cliente","phone":"123","received":"2026-10-28","status":"Recibido","serial":"","item":"Laptop","brand":"","model":"","otherShop":false,"otherShopReason":"","dropped":false,"dropReason":"","failure":"Falla de prueba", "password":"never-show-equipment-password","work":"Revisión","cost":"1500","deposit":"500","depositMethod":"Efectivo de prueba","notes":"Original","paid":false,"delivered":false})).unwrap()
}
async fn migrate(pool: &SqlitePool) {
    crate::test_support::migrate_legacy_range(pool,7,13).await;
}

#[test]
fn migration_preserves_history_and_new_operations_are_atomic_and_immutable() {
    tauri::async_runtime::block_on(async {
        let pool = database().await;
        let password_hash=crate::access::hash_password("test-user-123".into()).await.unwrap();
        sqlx::query("INSERT INTO access_settings VALUES (1,'master-test')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO users(id,name,password_hash,access_level,login_name) VALUES ('admin','Test admin',?,'complete','test admin')").bind(password_hash).execute(&pool).await.unwrap();
        let original = order();
        let author:String=sqlx::query_scalar("SELECT id FROM users LIMIT 1").fetch_one(&pool).await.unwrap();
        sqlx::query("INSERT INTO clients(id,name) VALUES ('client','Cliente')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO equipment(id,client_id,item) VALUES ('equipment','client','Laptop')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO orders(id,equipment_id,created_by,received,status,work,failure,price_minor,record_json) VALUES ('ORD-TEST','equipment',?,'2026-10-28','Recibido','Revisión','Falla de prueba',150000,?)")
            .bind(&author).bind(serde_json::to_string(&original).unwrap()).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO payments VALUES ('payment','ORD-TEST',?,50000,'2026-10-28')").bind(&author).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO audit_events(id,order_id,author_id,field,new_value) VALUES ('created','ORD-TEST',?,'created','ORD-TEST')").bind(author).execute(&pool).await.unwrap();
        migrate(&pool).await;
        let state=AccessState::default();
        state.test_login(&pool,"Test admin").await;
        let historical = list(&pool,&state,Filters::default()).await.unwrap();
        assert_eq!(historical.total,2);
        let created = historical.entries.iter().find(|e| e.operation=="orders.create").unwrap();
        assert_eq!(created.order_id,"ORD-TEST");
        assert_eq!(created.new_value.as_deref(),Some("ORD-TEST"));
        assert_eq!(created.author_name,"Test admin");
        assert!(!created.author_id.is_empty());
        assert!(created.old_value.is_none());
        assert!(created.occurred_at.as_ref().unwrap().ends_with('Z'));
        let payment = historical.entries.iter().find(|e| e.operation=="payments.add").unwrap();
        assert!(payment.occurred_at.is_none());
        assert!(payment.event_date.is_some());
        let mut value = serde_json::to_value(&original).unwrap();
        value["cost"] = serde_json::json!("1700"); value["notes"] = serde_json::json!("Modificada");
        let edited: orders::Order = serde_json::from_value(value).unwrap();
        orders::save(&pool,&state,edited.clone(),Some(original)).await.unwrap();
        let page = list(&pool,&state,Filters::default()).await.unwrap();
        assert_eq!(page.total,4);
        let price = page.entries.iter().find(|e| e.field.as_deref()==Some("price_minor")).unwrap();
        assert_eq!(price.old_value.as_deref(),Some("150000"));
        assert_eq!(price.new_value.as_deref(),Some("170000"));
        let note = page.entries.iter().find(|e| e.field.as_deref()==Some("notes")).unwrap();
        assert_eq!(note.old_value.as_deref(),Some("Original"));
        assert!(!serde_json::to_string(&page).unwrap().contains("never-show-equipment-password"));
        assert!(sqlx::query("UPDATE Bitacora_ordenes SET nombre_autor='Changed'").execute(&pool).await.is_err());
        assert!(sqlx::query("DELETE FROM Bitacora_ordenes").execute(&pool).await.is_err());
        orders::set_deleted(&pool,&state,edited.clone(),true).await.unwrap();
        assert!(list(&pool,&state,Filters::default()).await.unwrap().entries.iter().any(|e|e.operation=="orders.delete"));
        orders::set_deleted(&pool,&state,edited.clone(),false).await.unwrap();
        sqlx::raw_sql("CREATE TRIGGER fail_logbook BEFORE INSERT ON Bitacora_ordenes BEGIN SELECT RAISE(ABORT,'test'); END;").execute(&pool).await.unwrap();
        let before = list(&pool,&state,Filters::default()).await.unwrap().total;
        let mut value = serde_json::to_value(&edited).unwrap(); value["cost"] = serde_json::json!("1800");
        let rejected = serde_json::from_value(value).unwrap();
        assert!(orders::save(&pool,&state,rejected,Some(edited)).await.is_err());
        assert_eq!(list(&pool,&state,Filters::default()).await.unwrap().total,before);
        assert_eq!(sqlx::query_scalar::<_,i64>("SELECT precio_centavos FROM Ordenes").fetch_one(&pool).await.unwrap(),170000);
    });
}

#[test]
fn repair_details_and_payments_have_author_values_and_no_duplicate_payment_events() {
    tauri::async_runtime::block_on(async {
        let pool=database().await; migrate(&pool).await;
        let state=AccessState::test_admin(&pool).await;
        orders::save(&pool,&state,order(),None).await.unwrap();
        let snapshot=repairs::load(&pool,&state,"ORD-TEST",None).await.unwrap();
        let mut value=serde_json::to_value(snapshot).unwrap()["repair"].clone();
        value["technician"]=serde_json::json!("Ana");
        value["publicNote"]=serde_json::json!("Diagnóstico terminado");
        value["payments"]=serde_json::json!([{"date":"2026-10-28T11:30","amount":100,"method":"Efectivo de prueba"}]);
        repairs::save(&pool,&state,"ORD-TEST",serde_json::from_value(value).unwrap(),0).await.unwrap();
        let page=list(&pool,&state,Filters::default()).await.unwrap();
        assert_eq!(page.entries.iter().filter(|e|e.operation=="payments.add").count(),2);
        let payment=page.entries.iter().find(|e|e.new_value.as_deref()==Some("10000")).unwrap();
        assert!(payment.occurred_at.as_ref().unwrap().ends_with('Z'));
        assert_eq!(payment.event_date.as_deref(),Some("2026-10-28T11:30"));
        let repair=page.entries.iter().find(|e|e.operation=="repairs.edit").unwrap();
        assert!(repair.details.as_ref().unwrap().as_array().unwrap().iter().any(|d| d["field"]=="technician" && d["oldValue"]=="" && d["newValue"]=="Ana"));
        sqlx::query("UPDATE Usuarios SET nombre='Renamed admin' WHERE nombre='Test admin'").execute(&pool).await.unwrap();
        assert!(list(&pool,&state,Filters::default()).await.unwrap().entries.iter().all(|e|e.author_name=="Test admin"));
        let saved = repairs::load(&pool,&state,"ORD-TEST",None).await.unwrap();
        let mut value = serde_json::to_value(saved).unwrap()["repair"].clone();
        value["technician"] = serde_json::json!("Cambio rechazado");
        sqlx::raw_sql("CREATE TRIGGER fail_logbook BEFORE INSERT ON Bitacora_ordenes BEGIN SELECT RAISE(ABORT,'test'); END;").execute(&pool).await.unwrap();
        assert!(repairs::save(&pool,&state,"ORD-TEST",serde_json::from_value(value).unwrap(),1).await.is_err());
        let saved=serde_json::to_value(repairs::load(&pool,&state,"ORD-TEST",None).await.unwrap()).unwrap();
        assert_eq!(saved["repair"]["technician"],"Ana");
    });
}

#[test]
fn importing_orders_does_not_invent_a_historical_payment_date() {
    tauri::async_runtime::block_on(async {
        let pool=database().await; migrate(&pool).await;
        let state=AccessState::test_admin(&pool).await;
        orders::import(&pool,&state,vec![order()]).await.unwrap();
        let page=list(&pool,&state,Filters::default()).await.unwrap();
        let payment=page.entries.iter().find(|entry|entry.operation=="payments.add").unwrap();
        assert!(payment.occurred_at.is_some());
        assert!(payment.event_date.is_none());
    });
}

#[test]
fn logbook_permissions_filters_and_pagination_are_enforced_in_backend() {
    tauri::async_runtime::block_on(async {
        let pool=database().await; migrate(&pool).await;
        let state=AccessState::test_admin(&pool).await;
        orders::save(&pool,&state,order(),None).await.unwrap();
        let author: String=sqlx::query_scalar("SELECT id FROM Usuarios LIMIT 1").fetch_one(&pool).await.unwrap();
        for i in 0..60 {
            sqlx::query("INSERT INTO Historial_ordenes(id,id_orden,id_autor,campo,valor_anterior,valor_nuevo) VALUES (?,'ORD-TEST',?,'status','Anterior','Nuevo')")
                .bind(format!("test-{i}")).bind(&author).execute(&pool).await.unwrap();
        }
        let first=list(&pool,&state,Filters::default()).await.unwrap();
        assert_eq!(first.entries.len(),50); assert_eq!(first.total,62);
        let second=list(&pool,&state,Filters {page:1,..Filters::default()}).await.unwrap();
        assert_eq!(second.entries.len(),12);
        assert!(!second.entries.iter().any(|entry|first.entries.iter().any(|other|other.id==entry.id)));
        assert_eq!(list(&pool,&state,Filters {query:"ORD-TEST".into(),operation:"orders.edit".into(),author_id:author,page:0,..Filters::default()}).await.unwrap().total,60);
        assert_eq!(list(&pool,&state,Filters {query:"%' OR 1=1 --".into(),..Filters::default()}).await.unwrap().total,0);
        for (name,level,grants) in [("Limited","limited",vec![]),("Reader","custom",vec![users::Grant{section:"logbook".into(),action:"view".into()}])] {
            users::save(&pool,&state,users::UserInput{id:None,name:name.into(),password:"test-user-123".into(),confirmation:"test-user-123".into(),access_level:level.into(),active:true,permissions:grants,master_password:"test-master-123".into()}).await.unwrap();
        }
        state.test_login(&pool,"Limited").await;
        assert!(list(&pool,&state,Filters::default()).await.is_err());
        state.test_login(&pool,"Reader").await;
        assert_eq!(list(&pool,&state,Filters::default()).await.unwrap().total,64);
        sqlx::query("DELETE FROM Permisos_usuarios WHERE apartado='logbook'").execute(&pool).await.unwrap();
        assert!(list(&pool,&state,Filters::default()).await.is_err());
        assert!(list(&pool,&AccessState::default(),Filters::default()).await.is_err());
    });
}
