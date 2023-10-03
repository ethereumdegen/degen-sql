use serde_json::Error as SerdeJsonError;
use tokio_postgres::Error as PostgresError;

pub trait Model {}

#[derive(Debug, thiserror::Error)]
pub enum PostgresModelError {
    #[error(transparent)]
    Postgres(#[from] PostgresError),

    #[error(transparent)]
    SerdeJson(#[from] SerdeJsonError),

    #[error("Error parsing row for database")]
    RowParseError,
}
