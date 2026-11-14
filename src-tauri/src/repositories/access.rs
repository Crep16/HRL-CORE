use crate::{
    domain::security::{AccessLevel, Role, User},
    error::AppError,
};
use sqlx::{Row, SqliteConnection};

#[derive(Default)]
pub struct AccessRepository;
pub struct RecoveryAccount {
    pub id: i64,
    pub login: String,
    pub password_hash: String,
    pub master_hash: String,
}
pub struct AuthRecord {
    pub user: User,
    pub hash: String,
}
pub struct PermissionRecord {
    pub id: i64,
    pub section: String,
    pub action: String,
}
pub struct GrantRecord {
    pub id: i64,
    pub recipient: i64,
    pub approver: i64,
    pub scope: String,
    pub order_id: Option<i64>,
}

const USER_SELECT: &str = "SELECT u.id,u.nombre,u.nombre_acceso,u.hash_contrasena,u.activo,u.creado_en,n.codigo AS nivel,r.codigo AS rol FROM usuarios u JOIN niveles_acceso n ON n.id=u.id_nivel JOIN roles r ON r.id=u.id_rol";
fn user(row: &sqlx::sqlite::SqliteRow) -> Result<AuthRecord, AppError> {
    let access_level = match row.try_get::<&str, _>("nivel")? {
        "complete" => AccessLevel::Complete,
        "limited" => AccessLevel::Limited,
        "custom" => AccessLevel::Custom,
        _ => return Err(AppError::Forbidden),
    };
    let role = match row.try_get::<&str, _>("rol")? {
        "administrator" => Role::Administrator,
        "technician" => Role::Technician,
        _ => return Err(AppError::Forbidden),
    };
    Ok(AuthRecord {
        user: User {
            id: row.try_get("id")?,
            name: row.try_get("nombre")?,
            login_name: row.try_get("nombre_acceso")?,
            role,
            access_level,
            active: row.try_get("activo")?,
            created_at: row.try_get("creado_en")?,
        },
        hash: row.try_get("hash_contrasena")?,
    })
}
impl AccessRepository {
    pub async fn login_names(&self, db: &mut SqliteConnection) -> Result<Vec<String>, AppError> {
        Ok(sqlx::query_scalar("SELECT nombre_acceso FROM usuarios WHERE activo=1 ORDER BY nombre_acceso COLLATE NOCASE, id")
            .fetch_all(db).await?)
    }

    pub async fn master_hash(&self, db: &mut SqliteConnection) -> Result<String, AppError> {
        Ok(
            sqlx::query_scalar("SELECT hash FROM claves_maestras WHERE id=1")
                .fetch_one(db)
                .await?,
        )
    }
    pub async fn recovery_account(
        &self,
        db: &mut SqliteConnection,
    ) -> Result<Option<RecoveryAccount>, AppError> {
        // La recuperación afecta al administrador activo más antiguo; nunca a un id recibido de la UI.
        sqlx::query("SELECT u.id,u.nombre_acceso,u.hash_contrasena,m.hash FROM usuarios u JOIN niveles_acceso n ON n.id=u.id_nivel JOIN roles r ON r.id=u.id_rol JOIN claves_maestras m ON m.id=1 WHERE u.activo=1 AND n.codigo='complete' AND r.codigo='administrator' ORDER BY u.id LIMIT 1")
            .fetch_optional(db).await?.map(|row| Ok(RecoveryAccount {
                id: row.try_get("id")?, login: row.try_get("nombre_acceso")?,
                password_hash: row.try_get("hash_contrasena")?, master_hash: row.try_get("hash")?,
            })).transpose()
    }

    pub async fn reset_recovery_credentials(
        &self,
        db: &mut SqliteConnection,
        admin: i64,
        master_hash: &str,
        admin_hash: &str,
    ) -> Result<(), AppError> {
        let master = sqlx::query("UPDATE claves_maestras SET hash=? WHERE id=1")
            .bind(master_hash)
            .execute(&mut *db)
            .await?;
        let user = sqlx::query("UPDATE usuarios SET hash_contrasena=? WHERE id=? AND activo=1 AND id_nivel=(SELECT id FROM niveles_acceso WHERE codigo='complete') AND id_rol=(SELECT id FROM roles WHERE codigo='administrator')")
            .bind(admin_hash).bind(admin).execute(db).await?;
        if master.rows_affected() != 1 || user.rows_affected() != 1 {
            return Err(AppError::RecoveryRejected);
        }
        Ok(())
    }
    pub async fn setup_status(&self, db: &mut SqliteConnection) -> Result<bool, AppError> {
        Ok(
            sqlx::query_scalar("SELECT configurado FROM vista_configuracion_acceso")
                .fetch_one(db)
                .await?,
        )
    }
    pub async fn setup_occupied(&self, db: &mut SqliteConnection) -> Result<bool, AppError> {
        Ok(sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM usuarios) OR EXISTS(SELECT 1 FROM claves_maestras)",
        )
        .fetch_one(db)
        .await?)
    }
    pub async fn by_login(
        &self,
        db: &mut SqliteConnection,
        login: &str,
    ) -> Result<Option<AuthRecord>, AppError> {
        sqlx::query(&format!(
            "{USER_SELECT} WHERE u.nombre_acceso=? AND u.activo=1"
        ))
        .bind(login)
        .fetch_optional(db)
        .await?
        .as_ref()
        .map(user)
        .transpose()
    }
    pub async fn by_id(
        &self,
        db: &mut SqliteConnection,
        id: i64,
    ) -> Result<Option<AuthRecord>, AppError> {
        sqlx::query(&format!("{USER_SELECT} WHERE u.id=? AND u.activo=1"))
            .bind(id)
            .fetch_optional(db)
            .await?
            .as_ref()
            .map(user)
            .transpose()
    }
    pub async fn permissions(
        &self,
        db: &mut SqliteConnection,
        user_id: i64,
    ) -> Result<Vec<PermissionRecord>, AppError> {
        sqlx::query("SELECT p.id,a.codigo AS apartado,c.codigo AS accion FROM usuario_permiso up JOIN permisos p ON p.id=up.id_permiso JOIN apartados a ON a.id=p.id_apartado JOIN acciones c ON c.id=p.id_accion WHERE up.id_usuario=?")
            .bind(user_id).fetch_all(db).await?.iter().map(|r|Ok(PermissionRecord{id:r.try_get("id")?,section:r.try_get("apartado")?,action:r.try_get("accion")?})).collect()
    }
    pub async fn permission_id(
        &self,
        db: &mut SqliteConnection,
        section: &str,
        action: &str,
    ) -> Result<i64, AppError> {
        Ok(sqlx::query_scalar("SELECT p.id FROM permisos p JOIN apartados a ON a.id=p.id_apartado JOIN acciones c ON c.id=p.id_accion WHERE a.codigo=? AND c.codigo=?").bind(section).bind(action).fetch_one(db).await?)
    }
    pub async fn bootstrap(
        &self,
        db: &mut SqliteConnection,
        name: &str,
        login: &str,
        hash: &str,
        master: &str,
    ) -> Result<i64, AppError> {
        sqlx::query("INSERT INTO claves_maestras(id,hash) VALUES (1,?)")
            .bind(master)
            .execute(&mut *db)
            .await?;
        Ok(sqlx::query("INSERT INTO usuarios(nombre,nombre_acceso,hash_contrasena,id_nivel,id_rol) SELECT ?,?,?,n.id,r.id FROM niveles_acceso n JOIN roles r WHERE n.codigo='complete' AND r.codigo='administrator'")
            .bind(name).bind(login).bind(hash).execute(db).await?.last_insert_rowid())
    }
    pub async fn order_exists(&self, db: &mut SqliteConnection, id: i64) -> Result<bool, AppError> {
        Ok(sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM ordenes WHERE id=? AND eliminada_en IS NULL)",
        )
        .bind(id)
        .fetch_one(db)
        .await?)
    }
    pub async fn grant(
        &self,
        db: &mut SqliteConnection,
        recipient: i64,
        approver: i64,
        scope: &str,
        order: Option<i64>,
        reason: &str,
    ) -> Result<i64, AppError> {
        Ok(sqlx::query("INSERT INTO accesos_temporales(id_receptor,id_autorizador,id_apartado,id_orden,motivo) SELECT ?,?,id,?,? FROM apartados WHERE codigo=?")
            .bind(recipient).bind(approver).bind(order).bind(reason).bind(scope).execute(db).await?.last_insert_rowid())
    }
    pub async fn grant_permission(
        &self,
        db: &mut SqliteConnection,
        grant: i64,
        permission: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT OR IGNORE INTO acceso_temporal_permiso(id_acceso,id_permiso) VALUES (?,?)",
        )
        .bind(grant)
        .bind(permission)
        .execute(db)
        .await?;
        Ok(())
    }
    pub async fn live_grant(
        &self,
        db: &mut SqliteConnection,
        id: i64,
    ) -> Result<Option<GrantRecord>, AppError> {
        sqlx::query("SELECT t.id,t.id_receptor,t.id_autorizador,a.codigo,t.id_orden FROM accesos_temporales t JOIN apartados a ON a.id=t.id_apartado WHERE t.id=? AND t.finalizado_en IS NULL").bind(id).fetch_optional(db).await?.map(|r|Ok(GrantRecord{id:r.try_get("id")?,recipient:r.try_get("id_receptor")?,approver:r.try_get("id_autorizador")?,scope:r.try_get("codigo")?,order_id:r.try_get("id_orden")?})).transpose()
    }
    pub async fn grant_contains(
        &self,
        db: &mut SqliteConnection,
        grant: i64,
        permission: i64,
    ) -> Result<bool, AppError> {
        Ok(sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM acceso_temporal_permiso WHERE id_acceso=? AND id_permiso=?)").bind(grant).bind(permission).fetch_one(db).await?)
    }
    pub async fn open_grants(
        &self,
        db: &mut SqliteConnection,
    ) -> Result<Vec<(i64, i64)>, AppError> {
        sqlx::query("SELECT id,id_receptor FROM accesos_temporales WHERE finalizado_en IS NULL")
            .fetch_all(db)
            .await?
            .iter()
            .map(|r| Ok((r.try_get("id")?, r.try_get("id_receptor")?)))
            .collect()
    }
    pub async fn close_grant(
        &self,
        db: &mut SqliteConnection,
        id: i64,
        reason: &str,
    ) -> Result<bool, AppError> {
        Ok(sqlx::query("UPDATE accesos_temporales SET finalizado_en=strftime('%Y-%m-%dT%H:%M:%fZ','now'),motivo_finalizacion=? WHERE id=? AND finalizado_en IS NULL").bind(reason).bind(id).execute(db).await?.rows_affected()==1)
    }
}
