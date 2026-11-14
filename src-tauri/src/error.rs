use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    Migration(sqlx::migrate::MigrateError),
    LegacyDatabase,
    Validation(String),
    Unauthorized,
    Forbidden,
    InvalidCredentials,
    RateLimited,
    Conflict(String),
    NotFound,
    Crypto,
    Window,
    RecoveryUnavailable,
    RecoveryRejected,
    Commit(sqlx::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Database(_) => "No se pudo acceder a la base local.",
            Self::Migration(_) => "No se pudo inicializar la nueva base de datos.",
            Self::LegacyDatabase => "La nueva base requiere un archivo vacío; no se migran datos anteriores en esta fase.",
            Self::Validation(message) | Self::Conflict(message) => message,
            Self::Unauthorized => "Inicia sesión para continuar.",
            Self::Forbidden => "Tu acceso no permite realizar esta acción en este apartado o registro.",
            Self::InvalidCredentials => "Usuario o clave incorrectos.",
            Self::RateLimited => "Espera unos segundos antes de volver a intentar el acceso.",
            Self::NotFound => "El registro no está disponible.",
            Self::Crypto => "No se pudo proteger o recuperar la credencial del dispositivo.",
            Self::Window => "No se pudo ajustar el tamaño de la ventana. Vuelve a intentar el ajuste.",
            Self::RecoveryUnavailable => "La recuperación de soporte no está configurada para esta instalación.",
            Self::RecoveryRejected => "El desafío o la firma no son válidos, caducaron o ya fueron utilizados. Solicita un desafío nuevo.",
            Self::Commit(_) => "No se pudo confirmar el resultado. Consulta el registro antes de repetir la operación; si hay abono, conserva el mismo identificador y solicitud.",
        })
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Migration(error) => Some(error),
            Self::Commit(error) => Some(error),
            _ => None,
        }
    }
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Database(_) => "STORAGE_ERROR",
            Self::Migration(_) => "MIGRATION_ERROR",
            Self::LegacyDatabase => "LEGACY_DATABASE",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Unauthorized => "UNAUTHENTICATED",
            Self::Forbidden => "FORBIDDEN",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::RateLimited => "RATE_LIMITED",
            Self::Conflict(_) => "CONFLICT",
            Self::NotFound => "NOT_FOUND",
            Self::Crypto => "CRYPTO_ERROR",
            Self::Window => "WINDOW_ERROR",
            Self::RecoveryUnavailable => "RECOVERY_UNAVAILABLE",
            Self::RecoveryRejected => "RECOVERY_REJECTED",
            Self::Commit(_) => "RESULT_UNCERTAIN",
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut output = serializer.serialize_struct("AppError", 2)?;
        output.serialize_field("code", self.code())?;
        output.serialize_field("message", &self.to_string())?;
        output.end()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(error: sqlx::migrate::MigrateError) -> Self {
        Self::Migration(error)
    }
}
