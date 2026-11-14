use crate::{domain::user_contracts::*, error::AppError};
use sqlx::{Row, SqliteConnection};
#[derive(Default)]
pub struct UserRepository;
impl UserRepository {
    pub async fn list(&self, db: &mut SqliteConnection) -> Result<Vec<Account>, AppError> {
        let rows=sqlx::query("SELECT u.id,u.nombre_acceso AS nombre,u.activo,n.codigo AS nivel,r.codigo AS rol FROM usuarios u JOIN niveles_acceso n ON n.id=u.id_nivel JOIN roles r ON r.id=u.id_rol ORDER BY u.nombre,u.id").fetch_all(&mut *db).await?;
        let mut accounts = Vec::new();
        for row in rows {
            let id = row.try_get("id")?;
            let grants=sqlx::query("SELECT a.codigo AS apartado,c.codigo AS accion FROM usuario_permiso g JOIN permisos p ON p.id=g.id_permiso JOIN apartados a ON a.id=p.id_apartado JOIN acciones c ON c.id=p.id_accion WHERE g.id_usuario=? ORDER BY a.codigo,c.codigo").bind(id).fetch_all(&mut *db).await?;
            accounts.push(Account {
                id,
                name: row.try_get("nombre")?,
                role: row.try_get("rol")?,
                access_level: row.try_get("nivel")?,
                active: row.try_get("activo")?,
                permissions: grants
                    .into_iter()
                    .map(|g| {
                        Ok(Grant {
                            section: g.try_get("apartado")?,
                            action: g.try_get("accion")?,
                        })
                    })
                    .collect::<Result<Vec<_>, sqlx::Error>>()?,
            });
        }
        Ok(accounts)
    }
    pub async fn save(
        &self,
        db: &mut SqliteConnection,
        input: &SaveUserPayload,
        login: &str,
        hash: Option<&str>,
    ) -> Result<i64, AppError> {
        let id = if let Some(id) = input.id {
            if sqlx::query("UPDATE usuarios SET nombre=?,nombre_acceso=?,id_nivel=(SELECT id FROM niveles_acceso WHERE codigo=?),id_rol=(SELECT id FROM roles WHERE codigo=?),activo=?,hash_contrasena=coalesce(?,hash_contrasena) WHERE id=?").bind(input.name.trim()).bind(login).bind(&input.access_level).bind(&input.role).bind(input.active).bind(hash).bind(id).execute(&mut *db).await?.rows_affected()!=1 {return Err(AppError::NotFound)}
            id
        } else {
            sqlx::query("INSERT INTO usuarios(nombre,nombre_acceso,hash_contrasena,id_nivel,id_rol,activo) VALUES(?,?,?,(SELECT id FROM niveles_acceso WHERE codigo=?),(SELECT id FROM roles WHERE codigo=?),?)").bind(input.name.trim()).bind(login).bind(hash).bind(&input.access_level).bind(&input.role).bind(input.active).execute(&mut *db).await?.last_insert_rowid()
        };
        sqlx::query("DELETE FROM usuario_permiso WHERE id_usuario=?")
            .bind(id)
            .execute(&mut *db)
            .await?;
        if input.access_level != "complete" {
            for grant in &input.permissions {
                if input.access_level == "limited" && grant.section != "payments" {
                    continue;
                }
                sqlx::query("INSERT OR IGNORE INTO usuario_permiso(id_usuario,id_permiso) SELECT ?,p.id FROM permisos p JOIN apartados a ON a.id=p.id_apartado JOIN acciones c ON c.id=p.id_accion WHERE a.codigo=? AND c.codigo=? AND p.solo_completo=0").bind(id).bind(&grant.section).bind(&grant.action).execute(&mut *db).await?;
            }
        }
        Ok(id)
    }
}
