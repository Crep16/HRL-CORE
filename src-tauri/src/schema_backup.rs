use sqlx::{Connection, sqlite::{SqliteConnectOptions, SqliteConnection}};
use std::{fs, io};
use tauri::{Manager, Runtime, plugin::{Builder, TauriPlugin}};

async fn before_spanish_migration<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let directory = app.path().app_config_dir()?;
    let database = directory.join("hrl_datos.db");
    if !database.is_file() { return Ok(()); }
    let options = SqliteConnectOptions::new().filename(&database).read_only(true).create_if_missing(false);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    let has_migrations: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='_sqlx_migrations')")
        .fetch_one(&mut connection).await?;
    if !has_migrations { connection.close().await?; return Ok(()); }
    let applied: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _sqlx_migrations WHERE version=13 AND success=1)")
        .fetch_one(&mut connection).await?;
    if applied { connection.close().await?; return Ok(()); }

    let backups = directory.join("respaldos");
    fs::create_dir_all(&backups)?;
    let destination = backups.join(format!("antes-esquema-espanol-{}.sqlite", crate::access::random_id()));
    // VACUUM INTO includes committed WAL content; copying only the .db file can omit it.
    if let Err(error) = sqlx::query("VACUUM INTO ?").bind(destination.to_string_lossy().as_ref()).execute(&mut connection).await {
        let _ = fs::remove_file(&destination);
        return Err(error.into());
    }
    connection.close().await?;
    fs::OpenOptions::new().read(true).write(true).open(&destination)?.sync_all()?;
    Ok(())
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("respaldo-esquema")
        .setup(|app, _| {
            tauri::async_runtime::block_on(before_spanish_migration(app)).map_err(|error| {
                io::Error::other(format!("No se pudo crear el respaldo previo al cambio de nombres. La actualización no continuó: {error}"))
            })?;
            Ok(())
        })
        .build()
}
