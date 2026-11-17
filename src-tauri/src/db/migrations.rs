use crate::error::AppError;
use sqlx::{
    SqlitePool,
    migrate::{Migration, MigrationType, Migrator},
};
use std::borrow::Cow;

pub const BASELINE_SQL: &str = include_str!("../../migrations/001_esquema_objetivo.sql");

pub fn migrator() -> Migrator {
    Migrator {
        migrations: Cow::Owned(vec![
            Migration::new(
                1,
                "esquema_objetivo_3fn".into(),
                MigrationType::Simple,
                BASELINE_SQL.into(),
                false,
            ),
            Migration::new(
                2,
                "prefijo_documentos".into(),
                MigrationType::Simple,
                include_str!("../../migrations/002_prefijo_documentos.sql").into(),
                false,
            ),
            Migration::new(
                3,
                "antecedentes_equipo".into(),
                MigrationType::Simple,
                include_str!("../../migrations/003_antecedentes_equipo.sql").into(),
                false,
            ),
            Migration::new(
                4,
                "ocultar_nombres_etapas".into(),
                MigrationType::Simple,
                include_str!("../../migrations/004_ocultar_nombres_etapas.sql").into(),
                false,
            ),
            Migration::new(
                5,
                "retirar_etapas_pendientes".into(),
                MigrationType::Simple,
                include_str!("../../migrations/005_retirar_etapas_pendientes.sql").into(),
                false,
            ),
        ]),
        ..Migrator::DEFAULT
    }
}

pub async fn apply(pool: &SqlitePool) -> Result<(), AppError> {
    let legacy: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='Ordenes' AND instr(sql,'datos_json')>0)",
    ).fetch_one(pool).await?;
    if legacy {
        return Err(AppError::LegacyDatabase);
    }
    migrator().run(pool).await?;
    Ok(())
}
