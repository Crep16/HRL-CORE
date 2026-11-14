use crate::access::{self, AccessState};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::State;
use tauri_plugin_sql::DbInstances;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Grant {
    pub section: String,
    pub action: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInput {
    pub id: Option<String>,
    pub name: String,
    pub password: String,
    pub confirmation: String,
    pub access_level: String,
    pub active: bool,
    pub permissions: Vec<Grant>,
    pub master_password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    id: String,
    name: String,
    access_level: String,
    active: bool,
    permissions: Vec<Grant>,
}

pub(crate) async fn save(
    pool: &SqlitePool,
    state: &AccessState,
    input: UserInput,
) -> Result<(), String> {
    let key = access::login_name(&input.name)?;
    if !["complete", "limited", "custom"].contains(&input.access_level.as_str()) {
        return Err("Selecciona un acceso válido.".into());
    }
    if input.permissions.iter().any(|g| {
        if g.section == "payments" { !["add","refund"].contains(&g.action.as_str()) } else {
        !["clients", "repairs", "logbook", "company"].contains(&g.section.as_str())
            || !["view", "create", "edit", "delete"].contains(&g.action.as_str())
        }
    }) {
        return Err("Hay un permiso desconocido.".into());
    }
    if input.master_password.len() > 1024 {
        return Err("Revisa la clave maestra.".into());
    }
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo abrir la base local.")?;
    let author=state.authorize_scoped(&mut tx, "users.manage",None).await?;
    let master: String = sqlx::query_scalar("SELECT hash_clave_maestra FROM Configuracion_acceso WHERE id=1")
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| "Falta la configuración de la clave maestra.")?;
    if !access::verify_password(input.master_password, master).await? {
        return Err("La clave maestra no coincide.".into());
    }
    let hash = if input.id.is_none() || !input.password.is_empty() {
        access::validate_password(&input.password, &input.confirmation)?;
        Some(access::hash_password(input.password).await?)
    } else {
        None
    };
    let id = input.id.clone().unwrap_or_else(access::random_id);
    let credentials_updated=hash.is_some()&&input.id.is_some();
    let previous=audit_snapshot(&mut tx,&id).await?;
    if input.id.is_some() {
        let result = sqlx::query("UPDATE Usuarios SET nombre=?,nombre_acceso=?,nivel_acceso=?,activo=?,hash_contrasena=coalesce(?,hash_contrasena) WHERE id=?")
            .bind(input.name.trim()).bind(key).bind(&input.access_level).bind(input.active).bind(hash).bind(&id).execute(&mut *tx).await
            .map_err(|_| "No se pudo actualizar: el usuario puede estar repetido o ser el último administrador activo.")?;
        if result.rows_affected() != 1 {
            return Err("El usuario ya no existe.".into());
        }
    } else {
        sqlx::query("INSERT INTO Usuarios(id,nombre,nombre_acceso,hash_contrasena,nivel_acceso,activo) VALUES (?,?,?,?,?,?)")
            .bind(&id).bind(input.name.trim()).bind(key).bind(hash).bind(&input.access_level).bind(input.active).execute(&mut *tx).await
            .map_err(|_| "No se pudo crear el usuario. Comprueba que su nombre no esté repetido.")?;
    }
    sqlx::query("DELETE FROM Permisos_usuarios WHERE id_usuario=?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|_| "No se pudieron guardar los permisos.")?;
    if input.access_level == "custom" || input.access_level == "limited" {
        for grant in input.permissions {
            if input.access_level == "limited" && grant.section != "payments" { continue; }
            sqlx::query(
                "INSERT OR IGNORE INTO Permisos_usuarios(id_usuario,apartado,accion) VALUES (?,?,?)",
            )
            .bind(&id)
            .bind(grant.section)
            .bind(grant.action)
            .execute(&mut *tx)
            .await
            .map_err(|_| "No se pudieron guardar los permisos.")?;
        }
    }
    let current=audit_snapshot(&mut tx,&id).await?;
    let mut changes:Vec<_>=["name","accessLevel","active","permissions"].iter().filter(|field|previous[**field]!=current[**field]).map(|field|serde_json::json!({"field":field,"oldValue":previous[*field],"newValue":current[*field]})).collect();
    if credentials_updated{changes.push(serde_json::json!({"field":"credentialsUpdated","oldValue":null,"newValue":true}));}
    if !changes.is_empty(){crate::logbook::system_record(&mut tx,&format!("Usuario: {id}"),&author.id,if previous.is_null(){"users.create"}else{"users.edit"},serde_json::json!(changes)).await?;}
    tx.commit()
        .await
        .map_err(|_| "No se pudo confirmar la cuenta.")?;
    Ok(())
}
async fn audit_snapshot(db:&mut sqlx::SqliteConnection,id:&str)->Result<serde_json::Value,String>{
    let row=sqlx::query("SELECT nombre,nivel_acceso,activo FROM Usuarios WHERE id=?").bind(id).fetch_optional(&mut *db).await.map_err(|_|"No se pudo consultar el historial de la cuenta.")?;
    let Some(row)=row else{return Ok(serde_json::Value::Null)};
    let grants:Vec<_>=sqlx::query("SELECT apartado,accion FROM Permisos_usuarios WHERE id_usuario=? ORDER BY apartado,accion").bind(id).fetch_all(&mut *db).await.map_err(|_|"No se pudieron consultar los permisos.")?.into_iter().map(|g|serde_json::json!({"section":g.get::<String,_>("apartado"),"action":g.get::<String,_>("accion")})).collect();
    Ok(serde_json::json!({"name":row.get::<String,_>("nombre"),"accessLevel":row.get::<String,_>("nivel_acceso"),"active":row.get::<bool,_>("activo"),"permissions":grants}))
}

#[tauri::command]
pub async fn save_user(
    input: UserInput,
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<(), String> {
    save(&access::pool(&instances).await?, &state, input).await
}

#[tauri::command]
pub async fn list_users(
    instances: State<'_, DbInstances>,
    state: State<'_, AccessState>,
) -> Result<Vec<Account>, String> {
    let pool = access::pool(&instances).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| "No se pudo abrir la base local.")?;
    state.check_scoped(&mut tx, "users.manage",None).await?;
    let rows = sqlx::query("SELECT id,nombre,nivel_acceso,activo FROM Usuarios ORDER BY nombre")
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| "No se pudieron consultar los usuarios.")?;
    let mut accounts = Vec::new();
    for row in rows {
        let id: String = row.get("id");
        let grants = sqlx::query(
            "SELECT apartado,accion FROM Permisos_usuarios WHERE id_usuario=? ORDER BY apartado,accion",
        )
        .bind(&id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| "No se pudieron consultar los permisos.")?;
        accounts.push(Account {
            id,
            name: row.get("nombre"),
            access_level: row.get("nivel_acceso"),
            active: row.get("activo"),
            permissions: grants
                .iter()
                .map(|g| Grant {
                    section: g.get("apartado"),
                    action: g.get("accion"),
                })
                .collect(),
        });
    }
    tx.commit()
        .await
        .map_err(|_| "No se pudo completar la consulta.")?;
    Ok(accounts)
}
