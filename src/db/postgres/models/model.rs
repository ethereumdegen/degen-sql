use crate::db::postgres::postgres_db::DatabaseError;
use std::mem::discriminant;

use serde_json::Error as SerdeJsonError;
use tokio_postgres::Error as PostgresError;

pub trait Model {}

#[derive(Debug, thiserror::Error )]
pub enum PostgresModelError {
    #[error("Timeout")]
    Timeout,

    #[error(transparent)]
    Postgres(#[from] PostgresError),

      


    #[error(transparent)]
    SerdeJson(#[from] SerdeJsonError),

    #[error("Error parsing row for database: {0:?}")]
    RowParseError(Option<String>),

}

impl PartialEq for PostgresModelError {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

impl Eq for PostgresModelError {}