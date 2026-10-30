use super::*;
use crate::{orders,users};
async fn fixture()->(SqlitePool,AccessState){
 let pool=sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
 for schema in [include_str!("../migrations/001_access.sql"),include_str!("../migrations/002_workshop.sql"),include_str!("../migrations/003_authentication.sql"),include_str!("../migrations/004_order_records.sql"),include_str!("../migrations/005_order_visibility.sql"),include_str!("../migrations/006_repair_records.sql"),include_str!("../migrations/007_logbook.sql"),include_str!("../migrations/008_financial_logbook.sql")]{sqlx::raw_sql(schema).execute(&pool).await.unwrap();}
 sqlx::raw_sql(include_str!("../migrations/010_payment_adjustments.sql")).execute(&pool).await.unwrap();
 let state=AccessState::test_admin(&pool).await;(pool,state)
}
fn order(id:&str,paid:bool,delivered:bool,status:&str)->orders::Order{
 serde_json::from_value(serde_json::json!({"id":id,"client":"Cliente","phone":"123","received":"28/10/2026","status":status,"serial":"","item":"Laptop","brand":"HP","model":"14","otherShop":false,"otherShopReason":"","dropped":false,"dropReason":"","password":"never-copy-this","work":"Revisión","cost":"1500","deposit":if paid{"1500"}else{"500"},"depositMethod":"Efectivo de prueba","notes":"","paid":paid,"delivered":delivered})).unwrap()
}
#[test]
fn all_four_payment_delivery_combinations_use_the_ledger_and_direct_costs(){
 tauri::async_runtime::block_on(async{
  let(pool,state)=fixture().await;
  for(id,paid,delivered,status)in[("A",true,true,"En diagnóstico"),("B",true,false,"Listo para entrega"),("C",false,true,"Entregado"),("D",false,false,"Reparación terminada")]{orders::save(&pool,&state,order(id,paid,delivered,status),None).await.unwrap();}
  expense(&pool,&state,"A","Refacciones externas",60000,"Costo acordado").await.unwrap();
  let entries=list(&pool,&state).await.unwrap();
  for(id,expected)in[("A","completed"),("B","paid_uncollected"),("C","delivered_unpaid"),("D","pending")]{assert_eq!(entries.iter().find(|o|o.id==id).unwrap().classification,expected);}
  let completed=entries.iter().find(|o|o.id=="A").unwrap();assert_eq!(completed.profit_minor,90000);assert_eq!(completed.costs_minor,60000);assert_eq!(completed.balance_minor,0);
  assert_eq!(entries.iter().find(|o|o.id=="C").unwrap().balance_minor,100000);
  assert!(!serde_json::to_string(&entries).unwrap().contains("never-copy-this"));
  let events:i64=sqlx::query_scalar("SELECT count(*) FROM logbook_events WHERE operation='costs.add' AND reason='Costo acordado'").fetch_one(&pool).await.unwrap();assert_eq!(events,1);
  assert!(expense(&pool,&state,"A","",100,"Motivo").await.is_err());
  sqlx::raw_sql("CREATE TRIGGER fail_logbook BEFORE INSERT ON logbook_events BEGIN SELECT RAISE(ABORT,'test'); END;").execute(&pool).await.unwrap();
  assert!(expense(&pool,&state,"A","Rechazado",500,"Prueba").await.is_err());
  assert_eq!(list(&pool,&state).await.unwrap().iter().find(|o|o.id=="A").unwrap().costs_minor,60000);
 });
}
#[test]
fn exclusions_do_not_remove_money_trace_and_financial_access_is_checked(){
 tauri::async_runtime::block_on(async{
  let(pool,state)=fixture().await;
  let original=order("A",false,true,"Entregado");orders::save(&pool,&state,original.clone(),None).await.unwrap();orders::set_deleted(&pool,&state,original.clone(),true).await.unwrap();
  orders::save(&pool,&state,order("B",true,false,"Cancelada"),None).await.unwrap();
  let entries=list(&pool,&state).await.unwrap();assert!(entries.iter().all(|order|order.excluded));assert_eq!(entries.iter().map(|order|order.collected_minor).sum::<i64>(),200000);
  assert!(expense(&pool,&state,"B","No permitido",100,"Cancelada").await.is_err());
  orders::set_deleted(&pool,&state,original,false).await.unwrap();assert!(!list(&pool,&state).await.unwrap().iter().find(|o|o.id=="A").unwrap().excluded);
  users::save(&pool,&state,users::UserInput{id:None,name:"Limited".into(),password:"test-user-123".into(),confirmation:"test-user-123".into(),access_level:"limited".into(),active:true,permissions:vec![],master_password:"test-master-123".into()}).await.unwrap();state.test_login(&pool,"Limited").await;
  assert!(list(&pool,&state).await.is_err());assert!(expense(&pool,&state,"A","Costo",100,"Sin permiso").await.is_err());assert!(list(&pool,&AccessState::default()).await.is_err());
 });
}
