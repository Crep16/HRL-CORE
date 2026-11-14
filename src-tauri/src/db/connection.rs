use crate::error::AppError;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::path::Path;

pub struct Database {
    pool: SqlitePool,
    asset_directory: Option<std::path::PathBuf>,
}

impl Database {
    pub async fn open_file(path: &Path) -> Result<Self, AppError> {
        let mut database = Self::connect(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true),
            4,
        )
        .await?;
        database.asset_directory = path.parent().map(|parent| parent.join("assets-3fn"));
        Ok(database)
    }

    pub async fn in_memory() -> Result<Self, AppError> {
        Self::connect(SqliteConnectOptions::new().in_memory(true), 1).await
    }

    async fn connect(options: SqliteConnectOptions, connections: u32) -> Result<Self, AppError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(connections)
            .after_connect(|connection, _| {
                Box::pin(async move {
                    sqlx::query("PRAGMA foreign_keys=ON")
                        .execute(connection)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(options.foreign_keys(true))
            .await?;
        if let Err(error) = super::migrations::apply(&pool).await {
            pool.close().await;
            return Err(error);
        }
        Ok(Self {
            pool,
            asset_directory: None,
        })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
    pub fn asset_directory(&self) -> Option<std::path::PathBuf> {
        self.asset_directory.clone()
    }
    pub async fn close(self) {
        self.pool.close().await;
    }
}
