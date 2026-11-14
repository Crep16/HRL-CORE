use crate::access::{self,AccessState};
use serde_json::Value;
use sqlx::SqlitePool;
use tauri::State;
use tauri_plugin_sql::DbInstances;
#[cfg(test)]
#[path="company_tests.rs"]
mod tests;

fn validate(value:&Value)->Result<(),String>{
 let Some(object)=value.as_object() else{return Err("La configuración de Empresa no es válida.".into())};
 let fields=["name","taxId","phone","whatsapp","email","website","address","hours","logo","logoName","logoPrint","logoWeb","entryFooter","exitFooter","printAddress","signature","webReference","clauses","clausePosition","numberClauses","paper","orientation","copies","prefix","nextNumber","laborDays","partsDays","country","locale","currency","timezone","showTax","taxRate","taxName"];
 if object.keys().any(|key|!fields.contains(&key.as_str()))||object.len()!=fields.len()||value.to_string().len()>1_600_000{return Err("La configuración contiene campos desconocidos o es demasiado grande.".into());}
 for field in ["name","taxId","phone","whatsapp","email","website","address","hours","logo","logoName","entryFooter","exitFooter","prefix","country","taxName"]{if !value[field].is_string(){return Err("Revisa los textos de Empresa.".into());}}
 for field in ["logoPrint","logoWeb","printAddress","signature","webReference","numberClauses","showTax"]{if !value[field].is_boolean(){return Err("Revisa las opciones de Empresa.".into());}}
 for (field,min,max)in[("copies",1,5),("nextNumber",1,999999999),("laborDays",0,3650),("partsDays",0,3650)]{if !value[field].as_i64().is_some_and(|n|n>=min&&n<=max){return Err("Revisa los números de Empresa.".into());}}
 if value["name"].as_str().unwrap().trim().is_empty()||!value["taxRate"].as_f64().is_some_and(|n|(0. ..=100.).contains(&n))||!["MXN","USD","EUR"].contains(&value["currency"].as_str().unwrap_or(""))||!["letter","ticket","ticket60"].contains(&value["paper"].as_str().unwrap_or(""))||!["portrait","landscape"].contains(&value["orientation"].as_str().unwrap_or(""))||!["foot","back"].contains(&value["clausePosition"].as_str().unwrap_or(""))||!["es-MX","en-US","es-ES"].contains(&value["locale"].as_str().unwrap_or(""))||!value["timezone"].is_string()||!value["clauses"].is_array(){return Err("Revisa el formato y los datos de Empresa.".into());}
 if !["America/Cancun","America/Mexico_City","America/Tijuana","America/Hermosillo","UTC"].contains(&value["timezone"].as_str().unwrap_or("")){return Err("Selecciona una zona horaria disponible.".into());}
 for clause in value["clauses"].as_array().unwrap(){let Some(c)=clause.as_object()else{return Err("Revisa las cláusulas.".into())};if c.keys().any(|k|!["id","title","text","entry","exit"].contains(&k.as_str()))||!["id","title","text"].iter().all(|key|clause[*key].is_string())||!clause["entry"].is_boolean()||!clause["exit"].is_boolean(){return Err("Revisa los campos de las cláusulas.".into());}}
 Ok(())
}
pub(crate) async fn save(pool:&SqlitePool,state:&AccessState,value:Value,expected:Value)->Result<(),String>{
 validate(&value)?;validate(&expected)?;
 let mut tx=pool.begin().await.map_err(|_|"No se pudo abrir la base local.")?;
 let author=state.authorize_scoped(&mut tx,"company.edit",None).await?;
 let raw:Option<String>=sqlx::query_scalar("SELECT datos_json FROM Configuracion_empresa WHERE id=1").fetch_optional(&mut *tx).await.map_err(|_|"No se pudo consultar Empresa.")?;
 let old:Value=if let Some(raw)=raw{serde_json::from_str(&raw).map_err(|_|"La configuración guardada necesita revisión.")?}else{expected.clone()};
 if old!=expected{return Err("La configuración cambió desde que la abriste. Vuelve a consultar Empresa antes de guardar.".into());}
 let changes:Vec<_>=value.as_object().unwrap().iter().filter(|(key,new)|old[*key]!=**new).map(|(key,new)|if key=="logo"{serde_json::json!({"field":"logo","oldValue":if old[key].as_str()==Some(""){Value::Null}else{Value::String("Imagen anterior".into())},"newValue":if new.as_str()==Some(""){Value::Null}else{Value::String("Imagen actualizada".into())}})}else{serde_json::json!({"field":key,"oldValue":old[key],"newValue":new})}).collect();
 sqlx::query("INSERT INTO Configuracion_empresa(id,datos_json) VALUES (1,?) ON CONFLICT(id) DO UPDATE SET datos_json=excluded.datos_json").bind(value.to_string()).execute(&mut *tx).await.map_err(|_|"No se pudo guardar Empresa.")?;
 if !changes.is_empty(){crate::logbook::system_record(&mut tx,"Empresa",&author.id,"company.edit",serde_json::json!(changes)).await?;}
 tx.commit().await.map_err(|_|"No se pudo confirmar Empresa y su historial.".into())
}
#[tauri::command]
pub async fn save_company_settings(value:Value,expected:Value,instances:State<'_,DbInstances>,state:State<'_,AccessState>)->Result<(),String>{save(&access::pool(&instances).await?,&state,value,expected).await}
#[tauri::command]
pub async fn load_company_settings(instances:State<'_,DbInstances>,state:State<'_,AccessState>)->Result<Option<Value>,String>{
 let pool=access::pool(&instances).await?;let mut db=pool.acquire().await.map_err(|_|"No se pudo abrir la base local.")?;
 let mut allowed=false;for operation in ["company.view","clients.view","repairs.view","logbook.view"]{if state.check_scoped(&mut db,operation,None).await.is_ok(){allowed=true;break;}}
 if !allowed{return Err("Tu acceso no permite consultar la configuración.".into());}
 let raw:Option<String>=sqlx::query_scalar("SELECT datos_json FROM Configuracion_empresa WHERE id=1").fetch_optional(&mut *db).await.map_err(|_|"No se pudo consultar Empresa.")?;
 raw.map(|raw|serde_json::from_str(&raw).map_err(|_|"La configuración guardada necesita revisión.".into())).transpose()
}
