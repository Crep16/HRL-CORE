use crate::access::{self, AccessState};
use serde::Serialize;
use sqlx::{Row, SqlitePool};
use tauri::State;
use tauri_plugin_sql::DbInstances;

#[cfg(test)]
#[path="finances_tests.rs"]
mod tests;

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Payment { amount_minor:i64, occurred_at:Option<String>, kind:String }
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Expense { description:String, amount_minor:i64, occurred_at:String }
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct FinancialOrder {
 id:String, client:String, equipment:String, status:String, currency:String,
 total_minor:i64, collected_minor:i64, balance_minor:i64, costs_minor:i64, profit_minor:i64,
 received_minor:i64, adjusted_minor:i64, refunded_minor:i64, credit_minor:i64,
 delivered:bool, paid:bool, classification:String, excluded:bool,
 received:String, delivered_at:Option<String>, settled_at:Option<String>,
 payments:Vec<Payment>, expenses:Vec<Expense>,
}

pub(crate) fn classification(paid:bool,delivered:bool)-> &'static str {
 match (paid,delivered) { (true,true)=>"completed",(true,false)=>"paid_uncollected",(false,true)=>"delivered_unpaid",(false,false)=>"pending" }
}
pub(crate) async fn list(pool:&SqlitePool,state:&AccessState)->Result<Vec<FinancialOrder>,String>{
 let mut tx=pool.begin().await.map_err(|_|"No se pudo abrir la base local.")?;
 state.check_scoped(&mut tx,"logbook.view",None).await?;
 let rows=sqlx::query("SELECT o.id,c.nombre AS client,e.articulo,e.marca,e.modelo,o.estado,o.moneda,o.precio_centavos,o.fecha_recepcion,o.fecha_entrega,coalesce(json_extract(o.datos_json,'$.delivered'),o.fecha_entrega IS NOT NULL) AS delivered,coalesce((SELECT accion FROM Historial_papelera WHERE id_orden=o.id ORDER BY secuencia DESC LIMIT 1),'restored')='deleted' AS retirada FROM Ordenes o JOIN Equipo e ON e.id=o.id_equipo JOIN Clientes c ON c.id=e.id_cliente ORDER BY o.fecha_registro DESC,o.id")
   .fetch_all(&mut *tx).await.map_err(|_|"No se pudieron consultar las órdenes financieras.")?;
 let mut result=Vec::new();
 for row in rows {
  let id:String=row.get("id");
  let payments:Vec<Payment>=sqlx::query("SELECT importe_centavos,nullif(fecha_evento,'') AS fecha_evento,'receipt' AS tipo,id FROM Abonos WHERE id_orden=? UNION ALL SELECT CASE tipo WHEN 'refund' THEN -importe_centavos ELSE importe_centavos END,fecha_evento,tipo,id FROM Movimientos_abonos WHERE id_orden=? ORDER BY fecha_evento,id")
   .bind(&id).bind(&id).fetch_all(&mut *tx).await.map_err(|_|"No se pudieron consultar los cobros.")?.into_iter().map(|p|Payment{amount_minor:p.get("importe_centavos"),occurred_at:p.get("fecha_evento"),kind:p.get("tipo")}).collect();
  let expenses:Vec<Expense>=sqlx::query("SELECT descripcion,importe_centavos,fecha_evento FROM Gastos_orden WHERE id_orden=? UNION ALL SELECT nombre,precio_centavos,NULL FROM Refacciones_orden WHERE id_orden=?")
   .bind(&id).bind(&id).fetch_all(&mut *tx).await.map_err(|_|"No se pudieron consultar los costos.")?.into_iter().map(|e|Expense{description:e.get("descripcion"),amount_minor:e.get("importe_centavos"),occurred_at:e.get::<Option<String>,_>("fecha_evento").unwrap_or_default()}).collect();
  let total:i64=row.get("precio_centavos");
  let collected=payments.iter().try_fold(0_i64,|sum,p|sum.checked_add(p.amount_minor)).ok_or("El total de pagos supera el límite admitido.")?;
  let state=crate::orders::payment_state(&mut tx,&id).await?;
  let costs=expenses.iter().try_fold(0_i64,|sum,e|sum.checked_add(e.amount_minor)).ok_or("El total de costos supera el límite admitido.")?;
  let paid=collected>=total;
  let delivered:bool=row.get("delivered");
  let mut running=0_i64;
  let mut settled=None;
  for movement in &payments {
   let previously_paid=running>=total;
   running+=movement.amount_minor;
   if running<total { settled=None; }
   else if !previously_paid { settled=movement.occurred_at.clone(); }
  }
  if !paid { settled=None; }
  let status:String=row.get("estado");
  let excluded=row.get::<bool,_>("retirada")||status.trim().to_lowercase().starts_with("cancelad");
  result.push(FinancialOrder{id,client:row.get("client"),equipment:format!("{} {} {}",row.get::<String,_>("articulo"),row.get::<String,_>("marca"),row.get::<String,_>("modelo")).trim().into(),status,currency:row.get("moneda"),total_minor:total,collected_minor:collected,balance_minor:(total-collected).max(0),received_minor:state.received_minor,adjusted_minor:state.adjusted_minor,refunded_minor:state.refunded_minor,credit_minor:state.credit_minor,costs_minor:costs,profit_minor:total-costs,delivered,paid,classification:classification(paid,delivered).into(),excluded,received:row.get("fecha_recepcion"),delivered_at:row.get("fecha_entrega"),settled_at:settled,payments,expenses});
 }
 Ok(result)
}
#[tauri::command]
pub async fn list_order_finances(instances:State<'_,DbInstances>,state:State<'_,AccessState>)->Result<Vec<FinancialOrder>,String>{list(&access::pool(&instances).await?,&state).await}

pub(crate) async fn expense(pool:&SqlitePool,state:&AccessState,order_id:&str,description:&str,amount_minor:i64,reason:&str)->Result<(),String>{
 if description.trim().is_empty()||description.len()>500||reason.trim().is_empty()||reason.len()>1000||!(1..=9_000_000_000_000_i64).contains(&amount_minor){return Err("Escribe concepto, importe positivo y motivo del costo directo.".into());}
 let mut tx=pool.begin().await.map_err(|_|"No se pudo abrir la base local.")?;
 state.check_scoped(&mut tx,"logbook.view",None).await?;
 let author=state.authorize_scoped(&mut tx,"repairs.edit",Some(order_id)).await?;
 let active:i64=sqlx::query_scalar("SELECT count(*) FROM Ordenes o WHERE id=? AND lower(trim(estado)) NOT LIKE 'cancelad%' AND coalesce((SELECT accion FROM Historial_papelera WHERE id_orden=o.id ORDER BY secuencia DESC LIMIT 1),'restored')<>'deleted'").bind(order_id).fetch_one(&mut *tx).await.map_err(|_|"No se pudo consultar la orden.")?;
 if active!=1{return Err("La orden no está activa para registrar costos.".into());}
 sqlx::query("INSERT INTO Gastos_orden(id,id_orden,id_autor,descripcion,importe_centavos,motivo) VALUES (?,?,?,?,?,?)").bind(access::random_id()).bind(order_id).bind(author.id).bind(description.trim()).bind(amount_minor).bind(reason.trim()).execute(&mut *tx).await.map_err(|_|"No se pudo guardar el costo y su historial.")?;
 tx.commit().await.map_err(|_|"No se pudo confirmar el costo directo.".into())
}
#[tauri::command]
pub async fn add_direct_expense(order_id:String,description:String,amount_minor:i64,reason:String,instances:State<'_,DbInstances>,state:State<'_,AccessState>)->Result<(),String>{expense(&access::pool(&instances).await?,&state,&order_id,&description,amount_minor,&reason).await}
