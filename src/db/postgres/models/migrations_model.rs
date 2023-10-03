use crate::db::postgres::postgres_db::Database;

use super::model::PostgresModelError;

use std::str::FromStr;

pub struct Migration {
    name: String,
    executed_at: u64,
}

pub struct MigrationsModel {}

impl MigrationsModel {
    pub async fn find(psql_db: &Database) -> Result<Vec<Migration>, PostgresModelError> {
        let rows = psql_db
            .query(
                "
        SELECT 
            name,
            executed_at  
        FROM migrations
       
        ORDER BY executed_at DESC
        ;
        ",
                &[],
            )
            .await;

        match rows {
            Ok(rows) => {
                let mut migrations = Vec::new();

                for row in rows {
                    let migration = Migration {
                        name: row.get("name"),

                        //change this type ??
                        executed_at: row.get::<_, i64>("executed_at") as u64,
                    };

                    migrations.push(migration);
                }

                Ok(migrations)
            }
            Err(e) => {
                eprintln!("Database error: {:?}", e);
                Err(PostgresModelError::Postgres(e))
            }
        }
    }
}
