 
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

     #[error("UnexpectedRowsCount")]
    UnexpectedRowsCount,

    #[error(transparent)]
    SerdeJson(#[from] SerdeJsonError),

    #[error("Error parsing row for database: {0:?}")]
    RowParseError(Option<String>),

    #[error("ConnectionFailed  ")]
     ConnectionFailed ,

      #[error("PoolCreationFailed {0:?}")]

    PoolCreationFailed(String),


     #[error("QueryFailed {0:?}")]
    QueryFailed(tokio_postgres::Error),

    
     #[error("PostgresError {0:?}")]
    PostgresError(tokio_postgres::Error),

         #[error("PoolError {0:?}")]
    PoolError(deadpool::managed::PoolError<tokio_postgres::Error>),


}

impl PartialEq for PostgresModelError {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

impl Eq for PostgresModelError {}
 

impl From<deadpool::managed::PoolError<tokio_postgres::Error>> for PostgresModelError {
    fn from(error: deadpool::managed::PoolError<tokio_postgres::Error>) -> Self {
        PostgresModelError::PoolError(error)
    }
}