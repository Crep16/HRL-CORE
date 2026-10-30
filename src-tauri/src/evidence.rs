use crate::access::{self, AccessState};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::{fs, io::Write, path::{Path, PathBuf}};
use tauri::{Manager, State};
use tauri_plugin_sql::DbInstances;

const MAX_BYTES: usize = 20 * 1024 * 1024;
const DB_ERROR: &str = "No se pudo consultar el registro de evidencias.";
#[derive(Default)]
pub struct EvidenceState(pub tokio::sync::Mutex<()>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    id: String,
    order_id: String,
    original_name: String,
    mime: String,
    size_bytes: i64,
    source: String,
    note: String,
    author_name: String,
    created_at: String,
    deleted: bool,
    revision: i64,
}
fn from_row(row: &sqlx::sqlite::SqliteRow) -> Evidence {
    Evidence { id: row.get("id"), order_id: row.get("id_orden"), original_name: row.get("nombre_original"),
        mime: row.get("tipo_imagen"), size_bytes: row.get("tamano_bytes"), source: row.get("origen"), note: row.get("descripcion"),
        author_name: row.get("nombre_autor"), created_at: row.get("fecha_registro"), deleted: row.get("retirada"), revision: row.get("revision") }
}
fn valid_id(id: &str) -> Result<(), String> {
    if id.len()!=32 || !id.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
        return Err("El identificador de la evidencia no es válido.".into());
    }
    Ok(())
}
fn file_path(root: &Path, order: &str, id: &str, mime: &str) -> Result<PathBuf, String> {
    valid_id(id)?;
    let extension = match mime { "image/jpeg"=>"jpg", "image/png"=>"png", "image/webp"=>"webp", _=>return Err("Formato de imagen no permitido.".into()) };
    // Encode the complete order key so imported folios cannot introduce path components.
    let folder: String = order.as_bytes().iter().map(|byte|format!("{byte:02x}")).collect();
    Ok(root.join(folder).join(format!("{id}.{extension}")))
}
fn root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path().app_local_data_dir().map(|path|path.join("evidencias")).map_err(|_|"No se pudo localizar el almacenamiento de HRL.".into())
}
async fn order_exists(db: &mut sqlx::SqliteConnection, order: &str, writable: bool) -> Result<(), String> {
    if order.is_empty() || order.len()>200 { return Err("Selecciona una orden guardada.".into()); }
    let visible: Option<bool> = sqlx::query_scalar("SELECT coalesce((SELECT accion FROM Historial_papelera WHERE id_orden=Ordenes.id ORDER BY secuencia DESC LIMIT 1),'restored')<>'deleted' FROM Ordenes WHERE id=?")
        .bind(order).fetch_optional(db).await.map_err(|_|DB_ERROR)?;
    match visible { None=>Err("Primero guarda la orden para agregar evidencias.".into()), Some(false) if writable=>Err("Restaura la orden antes de modificar sus evidencias.".into()), _=>Ok(()) }
}
const SELECT: &str = "SELECT e.*,u.nombre AS nombre_autor FROM Evidencias_orden e JOIN Usuarios u ON u.id=e.id_autor";

#[tauri::command]
pub async fn list_evidence(order_id: String, deleted: Option<bool>, instances: State<'_, DbInstances>, state: State<'_, AccessState>) -> Result<Vec<Evidence>, String> {
    let pool=access::pool(&instances).await?;
    let mut tx=pool.begin().await.map_err(|_|DB_ERROR)?;
    state.check_scoped(&mut tx,"evidence.view",Some(&order_id)).await?;
    if deleted==Some(true) {state.check_scoped(&mut tx,"evidence.delete",Some(&order_id)).await?;}
    order_exists(&mut tx,&order_id,false).await?;
    let rows=sqlx::query(&format!("{SELECT} WHERE e.id_orden=? AND e.retirada=? ORDER BY e.fecha_registro,e.id"))
        .bind(&order_id).bind(deleted.unwrap_or(false)).fetch_all(&mut *tx).await.map_err(|_|DB_ERROR)?;
    Ok(rows.iter().map(from_row).collect())
}

#[tauri::command]
pub async fn read_evidence(app: tauri::AppHandle, order_id: String, id: String, instances: State<'_, DbInstances>, state: State<'_, AccessState>) -> Result<tauri::ipc::Response, String> {
    valid_id(&id)?;
    let pool=access::pool(&instances).await?;
    let mut tx=pool.begin().await.map_err(|_|DB_ERROR)?;
    state.check_scoped(&mut tx,"evidence.view",Some(&order_id)).await?;
    let row=sqlx::query("SELECT tipo_imagen,tamano_bytes,retirada FROM Evidencias_orden WHERE id=? AND id_orden=?")
        .bind(&id).bind(&order_id).fetch_optional(&mut *tx).await.map_err(|_|DB_ERROR)?.ok_or("La foto ya no está disponible. Actualiza la galería.")?;
    if row.get::<bool,_>("retirada") {state.check_scoped(&mut tx,"evidence.delete",Some(&order_id)).await?;}
    let path=file_path(&root(&app)?,&order_id,&id,row.get("tipo_imagen"))?;
    let bytes=fs::read(path).map_err(|_|"No se encontró el archivo de la foto. Recupera la carpeta de evidencias desde el respaldo.")?;
    if bytes.len()>MAX_BYTES || bytes.len() as i64!=row.get::<i64,_>("tamano_bytes") {return Err("El archivo de la foto cambió fuera de HRL. Revisa el respaldo.".into());}
    Ok(tauri::ipc::Response::new(bytes))
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct NewEvidence { id:String, order_id:String, original_name:String, mime:String, source:String, note:String, bytes:Vec<u8> }
fn validate(input: &NewEvidence) -> Result<(),String> {
    valid_id(&input.id)?;
    if input.original_name.trim().is_empty() || input.original_name.len()>500 || input.original_name.chars().any(char::is_control)
        || input.note.chars().count()>500 || !["file","camera"].contains(&input.source.as_str()) {
        return Err("Revisa el nombre, descripción y origen de la foto.".into());
    }
    if input.bytes.is_empty() || input.bytes.len()>MAX_BYTES {return Err("La foto debe ocupar entre 1 byte y 20 MB.".into());}
    let b=&input.bytes;
    let valid=match input.mime.as_str() {
        "image/jpeg"=>b.starts_with(&[0xff,0xd8,0xff]) && b.ends_with(&[0xff,0xd9]),
        "image/png"=>b.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/webp"=>b.len()>=12 && &b[..4]==b"RIFF" && &b[8..12]==b"WEBP",
        _=>false,
    };
    if !valid {return Err("El archivo no corresponde a una foto JPG, PNG o WebP.".into());}
    Ok(())
}

#[tauri::command]
pub async fn add_evidence(app: tauri::AppHandle, input: NewEvidence, instances: State<'_, DbInstances>, state: State<'_, AccessState>, store: State<'_, EvidenceState>) -> Result<Evidence,String> {
    validate(&input)?;
    let _guard=store.0.lock().await;
    let pool=access::pool(&instances).await?;
    let mut tx=pool.begin().await.map_err(|_|DB_ERROR)?;
    state.check_scoped(&mut tx,"evidence.add",Some(&input.order_id)).await?;
    order_exists(&mut tx,&input.order_id,true).await?;
    let path=file_path(&root(&app)?,&input.order_id,&input.id,&input.mime)?;
    let existing=sqlx::query(&format!("{SELECT} WHERE e.id=?")).bind(&input.id).fetch_optional(&mut *tx).await.map_err(|_|DB_ERROR)?;
    if let Some(row)=existing {
        let previous=from_row(&row);
        if previous.order_id!=input.order_id || previous.original_name!=input.original_name || previous.mime!=input.mime
            || previous.source!=input.source || row.get::<String,_>("descripcion_inicial")!=input.note
            || fs::read(&path).map_err(|_|"No se pudo recuperar el archivo ya registrado.")?!=input.bytes {
            return Err("La solicitud ya existe con otra fotografía. Vuelve a abrir la galería.".into());
        }
        return Ok(previous);
    }
    let author=state.authorize_scoped(&mut tx,"evidence.add",Some(&input.order_id)).await?;
    fs::create_dir_all(path.parent().ok_or("Carpeta de evidencia inválida.")?).map_err(|_|"No se pudo crear la carpeta de evidencias. Revisa el espacio y los permisos del equipo.")?;
    match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file)=>{
            if file.write_all(&input.bytes).and_then(|_|file.sync_all()).is_err() {
                drop(file);let _=fs::remove_file(&path);return Err("No se pudo escribir la foto completa. Revisa el espacio disponible.".into());
            }
        },
        Err(error) if error.kind()==std::io::ErrorKind::AlreadyExists=>{
            if fs::read(&path).map_err(|_|"No se pudo recuperar el archivo pendiente.")?!=input.bytes {
                return Err("Ya existe otro archivo para esta solicitud. No se sobrescribió.".into());
            }
        },
        Err(_)=>return Err("No se pudo crear el archivo de evidencia. Conserva la foto y vuelve a intentar.".into()),
    }
    let result: Result<Evidence,String>=async {
        sqlx::query("INSERT INTO Evidencias_orden(id,id_orden,nombre_original,tipo_imagen,tamano_bytes,origen,descripcion,descripcion_inicial,id_autor) VALUES (?,?,?,?,?,?,?,?,?)")
            .bind(&input.id).bind(&input.order_id).bind(&input.original_name).bind(&input.mime).bind(input.bytes.len() as i64)
            .bind(&input.source).bind(&input.note).bind(&input.note).bind(&author.id).execute(&mut *tx).await.map_err(|_|DB_ERROR)?;
        crate::logbook::system_record(&mut tx,&input.order_id,&author.id,"evidence.add",serde_json::json!([
            {"field":"evidenceId","oldValue":null,"newValue":input.id},
            {"field":"evidenceName","oldValue":null,"newValue":input.original_name},
            {"field":"evidenceSource","oldValue":null,"newValue":input.source},
            {"field":"evidenceNote","oldValue":null,"newValue":input.note}
        ])).await?;
        let row=sqlx::query(&format!("{SELECT} WHERE e.id=?")).bind(&input.id).fetch_one(&mut *tx).await.map_err(|_|DB_ERROR)?;
        Ok(from_row(&row))
    }.await;
    let saved=match result {Ok(saved)=>saved,Err(error)=>{let _=tx.rollback().await;let _=fs::remove_file(&path);return Err(error);}};
    // Keep the file if the commit result is uncertain; retrying the same ID recovers it.
    tx.commit().await.map_err(|_|"No se pudo confirmar el registro. Reintenta la misma foto antes de cerrar esta ventana.")?;
    Ok(saved)
}

#[tauri::command]
pub async fn update_evidence(order_id:String,id:String,note:String,deleted:bool,expected_revision:i64,instances:State<'_,DbInstances>,state:State<'_,AccessState>,store:State<'_,EvidenceState>) -> Result<Evidence,String> {
    valid_id(&id)?;
    if note.chars().count()>500 {return Err("La descripción admite hasta 500 caracteres.".into());}
    let _guard=store.0.lock().await;
    let pool=access::pool(&instances).await?;
    let mut tx=pool.begin().await.map_err(|_|DB_ERROR)?;
    order_exists(&mut tx,&order_id,true).await?;
    state.check_scoped(&mut tx,"evidence.view",Some(&order_id)).await?;
    let row=sqlx::query(&format!("{SELECT} WHERE e.id=? AND e.id_orden=?")).bind(&id).bind(&order_id).fetch_optional(&mut *tx).await.map_err(|_|DB_ERROR)?.ok_or("La evidencia ya no existe.")?;
    let previous=from_row(&row);
    if previous.revision!=expected_revision {return Err("La foto cambió desde que la abriste. Actualiza la galería antes de modificarla.".into());}
    let operation=if deleted!=previous.deleted {"evidence.delete"}else{"evidence.edit"};
    let author=state.authorize_scoped(&mut tx,operation,Some(&order_id)).await?;
    if note!=previous.note && operation!="evidence.edit" {state.authorize_scoped(&mut tx,"evidence.edit",Some(&order_id)).await?;}
    if previous.note==note && previous.deleted==deleted {return Ok(previous);}
    sqlx::query("UPDATE Evidencias_orden SET descripcion=?,retirada=?,revision=revision+1 WHERE id=? AND revision=?")
        .bind(&note).bind(deleted).bind(&id).bind(expected_revision).execute(&mut *tx).await.map_err(|_|DB_ERROR)?;
    crate::logbook::system_record(&mut tx,&order_id,&author.id,if operation=="evidence.delete" && !deleted {"evidence.restore"}else{operation},serde_json::json!([
        {"field":"evidenceId","oldValue":id,"newValue":id},
        {"field":"evidenceName","oldValue":previous.original_name,"newValue":previous.original_name},
        {"field":"evidenceNote","oldValue":previous.note,"newValue":note},
        {"field":"evidenceRemoved","oldValue":previous.deleted,"newValue":deleted}
    ])).await?;
    let row=sqlx::query(&format!("{SELECT} WHERE e.id=?")).bind(&id).fetch_one(&mut *tx).await.map_err(|_|DB_ERROR)?;
    let saved=from_row(&row);
    tx.commit().await.map_err(|_|"No se pudo confirmar el cambio. Actualiza la galería antes de repetirlo.")?;
    Ok(saved)
}
