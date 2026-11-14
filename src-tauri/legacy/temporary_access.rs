use super::*;

#[cfg(test)]
#[path="temporary_access_tests.rs"]
mod tests;

#[derive(Clone, Serialize)]
#[serde(rename_all="camelCase")]
pub struct TemporaryGrant {
    pub id:String,
    pub recipient_id:String,
    pub approver_id:String,
    pub approver_name:String,
    pub scope:String,
    pub record_id:Option<String>,
    pub operations:Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ApprovalRequest {
    pub name:String,
    pub password:String,
    pub scope:String,
    pub record_id:Option<String>,
    pub operation:String,
    pub reason:String,
}

fn scope_for(operation:&str)->Option<&str> {
    crate::permissions::OPERATIONS.iter().find(|(id,_,_)|*id==operation)
        .map(|(_,section,_)| if *section=="payments" {"repairs"} else {*section})
}
fn scope_matches(operation:&str,scope:&str)->bool {
    scope_for(operation)==Some(scope) || (scope=="clients"&&operation=="payments.add")
        || (scope=="logbook"&&operation=="repairs.edit")
}

impl AccessState {
    pub(crate) async fn approve_temporary(&self,pool:&SqlitePool,input:ApprovalRequest)->Result<TemporaryGrant,String> {
        if !scope_matches(&input.operation,&input.scope) || input.reason.trim().is_empty()
            || input.reason.len()>2000 || input.password.is_empty() || input.password.len()>1024
            || input.record_id.as_ref().is_some_and(|s|s.trim().is_empty()||s.len()>100)
            || (input.scope=="repairs" && input.record_id.is_none())
            || (input.scope!="repairs" && input.record_id.is_some()) {
            return Err("Indica el apartado, registro y motivo de la autorización.".into());
        }
        let key=login_name(&input.name)?;
        let _gate=self.1.lock().await;
        let recipient={
            let session=self.0.lock().await;
            if session.approval_retry_at.is_some_and(|time|time>Instant::now()) {return Err("Espera unos segundos antes de volver a solicitar autorización.".into());}
            session.user.clone().ok_or("Inicia sesión para solicitar autorización.")?
        };
        let mut tx=pool.begin().await.map_err(|_|DATABASE_ERROR)?;
        let current=sqlx::query("SELECT id,nombre,nivel_acceso FROM Usuarios WHERE id=? AND activo=1").bind(&recipient.id).fetch_optional(&mut *tx).await.map_err(|_|DATABASE_ERROR)?.ok_or("Tu sesión ya no está activa.")?;
        let recipient=User{id:current.get("id"),name:current.get("nombre"),access_level:current.get("nivel_acceso")};
        let row=sqlx::query("SELECT id,nombre,hash_contrasena,nivel_acceso FROM Usuarios WHERE nombre_acceso=? AND activo=1").bind(key).fetch_optional(&mut *tx).await.map_err(|_|DATABASE_ERROR)?;
        let valid=if let Some(row)=&row {verify_password(input.password,row.get("hash_contrasena")).await?} else {false};
        if !valid {
            let mut session=self.0.lock().await;session.approval_failures+=1;
            if session.approval_failures>=5 {session.approval_retry_at=Some(Instant::now()+Duration::from_secs(5));}
            return Err(INVALID_LOGIN.into());
        }
        let row=row.ok_or(INVALID_LOGIN)?;
        let approver=User{id:row.get("id"),name:row.get("nombre"),access_level:row.get("nivel_acceso")};
        let evaluator=AccessState::default();evaluator.0.lock().await.user=Some(approver.clone());
        let mut operations=Vec::new();
        for (operation,_,_) in crate::permissions::OPERATIONS {
            if scope_matches(operation,&input.scope) && evaluator.authorize(&mut tx,operation).await.is_ok() {operations.push(operation.to_string());}
        }
        if !operations.contains(&input.operation) {return Err("La persona que autoriza no tiene ese permiso. Consulta con un superior que sí lo tenga.".into());}
        if let Some(id)=&input.record_id {
            let exists:i64=sqlx::query_scalar("SELECT count(*) FROM Ordenes WHERE id=?").bind(id).fetch_one(&mut *tx).await.map_err(|_|DATABASE_ERROR)?;
            if exists!=1 {return Err("La orden solicitada no existe.".into());}
        }
        let grant=TemporaryGrant{id:random_id(),recipient_id:recipient.id.clone(),approver_id:approver.id,approver_name:approver.name,scope:input.scope,record_id:input.record_id,operations};
        self.0.lock().await.temporary=None;
        sqlx::query("UPDATE Accesos_temporales SET fecha_finalizacion=strftime('%Y-%m-%dT%H:%M:%fZ','now'),motivo_finalizacion='Reemplazada por otra autorización' WHERE id_receptor=? AND fecha_finalizacion IS NULL")
            .bind(&recipient.id).execute(&mut *tx).await.map_err(|_|"No se pudo registrar el fin de la autorización anterior.")?;
        sqlx::query("INSERT INTO Accesos_temporales(id,id_receptor,nombre_receptor,id_autorizador,nombre_autorizador,apartado,id_registro,operaciones_json,motivo) VALUES (?,?,?,?,?,?,?,?,?)")
            .bind(&grant.id).bind(&recipient.id).bind(&recipient.name).bind(&grant.approver_id).bind(&grant.approver_name).bind(&grant.scope).bind(&grant.record_id)
            .bind(serde_json::to_string(&grant.operations).map_err(|_|DATABASE_ERROR)?).bind(input.reason.trim())
            .execute(&mut *tx).await.map_err(|_|"No se pudo guardar la autorización y su historial. No se concedió acceso.")?;
        tx.commit().await.map_err(|_|"No se pudo confirmar la autorización. Vuelve a solicitarla.")?;
        let mut session=self.0.lock().await;
        session.temporary=Some(grant.clone());session.approval_failures=0;session.approval_retry_at=None;
        Ok(grant)
    }

    pub(crate) async fn authorize_scoped(&self,db:&mut sqlx::SqliteConnection,operation:&str,record:Option<&str>)->Result<User,String> {
        self.authorize_scoped_inner(db,operation,record,true).await
    }
    pub(crate) async fn check_scoped(&self,db:&mut sqlx::SqliteConnection,operation:&str,record:Option<&str>)->Result<User,String> {
        self.authorize_scoped_inner(db,operation,record,false).await
    }
    async fn authorize_scoped_inner(&self,db:&mut sqlx::SqliteConnection,operation:&str,record:Option<&str>,audit:bool)->Result<User,String> {
        if let Ok(user)=self.authorize(db,operation).await {return Ok(user);}
        let (recipient,grant)={let session=self.0.lock().await;(session.user.clone().ok_or("Inicia sesión para continuar.")?,session.temporary.clone().ok_or("No tienes acceso a esta opción. Solicita autorización temporal a un superior.")?)};
        if grant.recipient_id!=recipient.id || !scope_matches(operation,&grant.scope) || grant.record_id.as_deref().is_some_and(|id|Some(id)!=record)
            || !grant.operations.iter().any(|allowed|allowed==operation) {return Err("La autorización temporal no cubre esta opción o registro.".into());}
        let current=sqlx::query("SELECT u.id,u.nombre,u.nivel_acceso FROM Accesos_temporales g JOIN Usuarios u ON u.id=g.id_receptor WHERE g.id=? AND g.fecha_finalizacion IS NULL AND u.activo=1")
            .bind(&grant.id).fetch_optional(&mut *db).await.map_err(|_|DATABASE_ERROR)?.ok_or("La autorización temporal ya terminó o tu cuenta fue desactivada.")?;
        let recipient=User{id:current.get("id"),name:current.get("nombre"),access_level:current.get("nivel_acceso")};
        let evaluator=AccessState::default();
        evaluator.0.lock().await.user=Some(User{id:grant.approver_id.clone(),name:grant.approver_name.clone(),access_level:String::new()});
        evaluator.authorize(db,operation).await.map_err(|_|"La persona que autorizó ya no tiene este permiso. Solicita una nueva autorización.")?;
        if audit && !operation.ends_with(".view") {
            crate::logbook::system_record(db,record.unwrap_or(&grant.scope),&recipient.id,"access.temporary.use",serde_json::json!([
                {"field":"temporaryGrant","oldValue":null,"newValue":grant.id},
                {"field":"authorizedBy","oldValue":null,"newValue":grant.approver_name},
                {"field":"performedBy","oldValue":null,"newValue":recipient.name},
                {"field":"temporaryOperation","oldValue":null,"newValue":operation}
            ])).await?;
        }
        Ok(recipient)
    }

    pub(super) async fn end_temporary_locked(&self,pool:&SqlitePool,reason:&str)->Result<(),String> {
        let grant=self.0.lock().await.temporary.take();
        if let Some(grant)=grant {
            sqlx::query("UPDATE Accesos_temporales SET fecha_finalizacion=strftime('%Y-%m-%dT%H:%M:%fZ','now'),motivo_finalizacion=? WHERE id=? AND fecha_finalizacion IS NULL")
                .bind(reason).bind(grant.id).execute(pool).await.map_err(|_|"El acceso temporal fue retirado, pero no se pudo registrar su finalización. Vuelve a consultar la sesión.")?;
        }
        Ok(())
    }
    pub(crate) async fn end_temporary(&self,pool:&SqlitePool)->Result<(),String> {
        let _gate=self.1.lock().await;self.end_temporary_locked(pool,"Se cerró la ventana o se finalizó el acceso temporal").await
    }
    pub(crate) async fn recover_temporary(&self,pool:&SqlitePool)->Result<(),String> {
        let _gate=self.1.lock().await;
        let active=self.0.lock().await.temporary.as_ref().map(|g|g.id.clone()).unwrap_or_default();
        sqlx::query("UPDATE Accesos_temporales SET fecha_finalizacion=strftime('%Y-%m-%dT%H:%M:%fZ','now'),motivo_finalizacion='Fin detectado al abrir o recuperar la sesión; no se conoce la hora exacta de cierre' WHERE fecha_finalizacion IS NULL AND id<>?")
            .bind(active).execute(pool).await.map_err(|_|"No se pudo registrar el fin de una autorización de la sesión anterior.")?;
        Ok(())
    }
}

#[tauri::command]
pub async fn grant_temporary_access(input:ApprovalRequest,instances:State<'_,DbInstances>,state:State<'_,AccessState>)->Result<TemporaryGrant,String> {
    state.approve_temporary(&pool(&instances).await?,input).await
}
#[tauri::command]
pub async fn end_temporary_access(instances:State<'_,DbInstances>,state:State<'_,AccessState>)->Result<(),String> {
    state.end_temporary(&pool(&instances).await?).await
}
