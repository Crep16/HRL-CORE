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
        !["clients", "repairs", "logbook", "company"].contains(&g.section.as_str())
            || !["view", "create", "edit", "delete"].contains(&g.action.as_str())
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
    state.authorize(&mut tx, "users.manage").await?;
    let master: String = sqlx::query_scalar("SELECT master_hash FROM access_settings WHERE id=1")
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
    if input.id.is_some() {
        let result = sqlx::query("UPDATE users SET name=?,login_name=?,access_level=?,active=?,password_hash=coalesce(?,password_hash) WHERE id=?")
            .bind(input.name.trim()).bind(key).bind(&input.access_level).bind(input.active).bind(hash).bind(&id).execute(&mut *tx).await
            .map_err(|_| "No se pudo actualizar: el usuario puede estar repetido o ser el último administrador activo.")?;
        if result.rows_affected() != 1 {
            return Err("El usuario ya no existe.".into());
        }
    } else {
        sqlx::query("INSERT INTO users(id,name,login_name,password_hash,access_level,active) VALUES (?,?,?,?,?,?)")
            .bind(&id).bind(input.name.trim()).bind(key).bind(hash).bind(&input.access_level).bind(input.active).execute(&mut *tx).await
            .map_err(|_| "No se pudo crear el usuario. Comprueba que su nombre no esté repetido.")?;
    }
    sqlx::query("DELETE FROM user_permissions WHERE user_id=?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|_| "No se pudieron guardar los permisos.")?;
    if input.access_level == "custom" {
        for grant in input.permissions {
            sqlx::query(
                "INSERT OR IGNORE INTO user_permissions(user_id,section,action) VALUES (?,?,?)",
            )
            .bind(&id)
            .bind(grant.section)
            .bind(grant.action)
            .execute(&mut *tx)
            .await
            .map_err(|_| "No se pudieron guardar los permisos.")?;
        }
    }
    tx.commit()
        .await
        .map_err(|_| "No se pudo confirmar la cuenta.")?;
    Ok(())
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
    state.authorize(&mut tx, "users.manage").await?;
    let rows = sqlx::query("SELECT id,name,access_level,active FROM users ORDER BY name")
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| "No se pudieron consultar los usuarios.")?;
    let mut accounts = Vec::new();
    for row in rows {
        let id: String = row.get("id");
        let grants = sqlx::query(
            "SELECT section,action FROM user_permissions WHERE user_id=? ORDER BY section,action",
        )
        .bind(&id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| "No se pudieron consultar los permisos.")?;
        accounts.push(Account {
            id,
            name: row.get("name"),
            access_level: row.get("access_level"),
            active: row.get("active"),
            permissions: grants
                .iter()
                .map(|g| Grant {
                    section: g.get("section"),
                    action: g.get("action"),
                })
                .collect(),
        });
    }
    tx.commit()
        .await
        .map_err(|_| "No se pudo completar la consulta.")?;
    Ok(accounts)
}
