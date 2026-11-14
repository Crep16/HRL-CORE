use super::*;
fn settings()->Value{serde_json::json!({"name":"Taller","taxId":"","phone":"","whatsapp":"","email":"","website":"","address":"","hours":"","logo":"","logoName":"","logoPrint":true,"logoWeb":true,"entryFooter":"Entrada","exitFooter":"Salida","printAddress":true,"signature":true,"webReference":false,"clauses":[],"clausePosition":"foot","numberClauses":true,"paper":"letter","orientation":"portrait","copies":2,"prefix":"ORD-","nextNumber":1,"laborDays":30,"partsDays":90,"country":"México","locale":"es-MX","currency":"MXN","timezone":"America/Cancun","showTax":false,"taxRate":16,"taxName":"IVA"})}

#[test]
fn company_reopens_with_logo_and_rechecks_editor_permissions() {
    tauri::async_runtime::block_on(async {
        let directory = std::env::temp_dir().join(format!("hrl-company-{}", access::random_id()));
        std::fs::create_dir_all(&directory).unwrap();
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(directory.join("company.db"))
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
            .connect_with(options.clone()).await.unwrap();
        crate::test_support::migrate_legacy(&pool).await;
        let state = AccessState::test_admin(&pool).await;
        let original = settings();
        let mut saved = original.clone();
        saved["name"] = serde_json::json!("Taller de prueba de reapertura");
        saved["phone"] = serde_json::json!("9980000000");
        saved["address"] = serde_json::json!("Direccion ficticia de prueba");
        saved["logo"] = serde_json::json!("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+j5XkAAAAASUVORK5CYII=");
        save(&pool, &state, saved.clone(), original).await.unwrap();
        crate::users::save(&pool, &state, crate::users::UserInput {
            id: None, name: "Company reader".into(), password: "test-user-123".into(),
            confirmation: "test-user-123".into(), access_level: "custom".into(), active: true,
            permissions: vec![crate::users::Grant { section: "company".into(), action: "view".into() }],
            master_password: "test-master-123".into(),
        }).await.unwrap();
        pool.close().await;
        drop(pool);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
            .connect_with(options).await.unwrap();
        let stored: String = sqlx::query_scalar("SELECT datos_json FROM Configuracion_empresa WHERE id=1")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(serde_json::from_str::<Value>(&stored).unwrap(), saved);
        let fresh_session = AccessState::default();
        let mut edited = saved.clone();
        edited["phone"] = serde_json::json!("9980000001");
        assert!(save(&pool, &fresh_session, edited.clone(), saved.clone()).await.is_err());
        fresh_session.test_login(&pool, "Company reader").await;
        assert!(save(&pool, &fresh_session, edited.clone(), saved.clone()).await.is_err());
        let reader: String = sqlx::query_scalar("SELECT id FROM Usuarios WHERE nombre='Company reader'")
            .fetch_one(&pool).await.unwrap();
        sqlx::query("INSERT INTO Permisos_usuarios VALUES (?,'company','edit')")
            .bind(&reader).execute(&pool).await.unwrap();
        save(&pool, &fresh_session, edited.clone(), saved).await.unwrap();
        sqlx::query("DELETE FROM Permisos_usuarios WHERE id_usuario=? AND accion='edit'")
            .bind(&reader).execute(&pool).await.unwrap();
        assert!(save(&pool, &fresh_session, edited.clone(), edited.clone()).await.is_err());
        let stored: String = sqlx::query_scalar("SELECT datos_json FROM Configuracion_empresa WHERE id=1")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(serde_json::from_str::<Value>(&stored).unwrap(), edited);
        pool.close().await;
        drop(pool);
        std::fs::remove_dir_all(directory).unwrap();
    });
}
#[test]
fn company_changes_and_audit_commit_together_and_reject_stale_or_sensitive_fields(){
 tauri::async_runtime::block_on(async{
  let pool=sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
  crate::test_support::migrate_legacy(&pool).await;
  let state=AccessState::test_admin(&pool).await;let old=settings();let mut new=old.clone();new["name"]=serde_json::json!("Actualizado");new["logo"]=serde_json::json!("data:image/png;base64,test-only");
  save(&pool,&state,new.clone(),old.clone()).await.unwrap();
  let history:String=sqlx::query_scalar("SELECT detalles_json FROM Bitacora_sistema WHERE operacion='company.edit'").fetch_one(&pool).await.unwrap();assert!(history.contains("Actualizado"));assert!(history.contains("Taller"));assert!(!history.contains("base64"));
  assert!(save(&pool,&state,old.clone(),old).await.is_err());
  let mut bad=new.clone();bad["password"]=serde_json::json!("never-copy");assert!(save(&pool,&state,bad,new.clone()).await.is_err());
  assert!(save(&pool,&AccessState::default(),new.clone(),new.clone()).await.is_err());
  sqlx::raw_sql("CREATE TRIGGER fail_company_audit BEFORE INSERT ON Bitacora_sistema BEGIN SELECT RAISE(ABORT,'test'); END;").execute(&pool).await.unwrap();
  let mut attempted=new.clone();attempted["name"]=serde_json::json!("No guardar");assert!(save(&pool,&state,attempted,new.clone()).await.is_err());
  let stored:String=sqlx::query_scalar("SELECT datos_json FROM Configuracion_empresa").fetch_one(&pool).await.unwrap();assert_eq!(serde_json::from_str::<Value>(&stored).unwrap(),new);
 });
}
