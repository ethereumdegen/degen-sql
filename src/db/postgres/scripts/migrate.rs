use degen_sql::db::postgres::postgres_db::{Database, DatabaseCredentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = DatabaseCredentials::from_env();

    let conn_url = credentials.build_connection_url();

    let mut database = Database::connect(conn_url, None).await?;

    let _migration = database.migrate().await?;

    Ok(())
}
