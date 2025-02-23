use tokio_postgres::Client;
use crate::db::postgres::models::model::PostgresModelError;
use tokio::time::timeout;
use tokio::time::Duration;
use tokio::time::sleep;
use log::info;
use tokio;
use tokio_postgres::{Error as PostgresError, NoTls};

use std::error::Error;
use tokio_postgres_migration::Migration;

use dotenvy::dotenv;
use std::env;

use std::fs;
use std::str;

type MigrationDefinition = (String, String);

#[derive(Clone)]
struct Migrations {
    up: Vec<MigrationDefinition>,
    down: Vec<MigrationDefinition>,
}

pub trait MigrationAsStr {
    fn to_str(&self) -> (&str, &str);
}

impl MigrationAsStr for MigrationDefinition {
    fn to_str(&self) -> (&str, &str) {
        return (self.0.as_str(), self.1.as_str());
    }
}

pub struct Database {
  //  pub client: Option<  tokio_postgres::Client > ,
    pub migrations_dir_path: Option<String>,
    pub connection_url:  String  , 
   
    pub max_reconnect_attempts: u32, 
    pub timeout_duration: Duration, 
}

#[derive(Debug,Clone)]
pub struct DatabaseCredentials {
    pub db_name: String,
    pub db_host: String,
    pub db_user: String,
    pub db_password: String,
}

impl Default for DatabaseCredentials {
    fn default() -> Self {
        Self {
            db_name: "postgres".into(),
            db_host: "localhost".into(),
            db_user: "postgres".into(),
            db_password: "postgres".into(),
        }
    }
}

impl DatabaseCredentials {
    pub fn from_env() -> Self {
        //dotenv().expect(".env file not found"); //need to run this beforehand !! 

        Self {
            db_name: env::var("DB_NAME").unwrap_or("postgres".into()),
            db_host: env::var("DB_HOST").unwrap_or("localhost".into()),
            db_user: env::var("DB_USER").unwrap_or("postgres".into()),
            db_password: env::var("DB_PASSWORD").unwrap_or("postgres".into()),
        }
    }

    pub fn build_connection_url(&self) -> String {
        return format!(
            "postgres://{}:{}@{}/{}",
            self.db_user, self.db_password, self.db_host, self.db_name
        );
    }
}

enum PostgresInputType {
    Query,
    QueryOne,
    Execute,
}

struct PostgresInput<'a> {
    input_type: PostgresInputType,
    query: String,
    params: &'a [&'a (dyn tokio_postgres::types::ToSql + Sync)],
}

impl Database {

    pub fn new(

         conn_url: String, 
         migrations_dir_path: Option<String>,
     ) -> Result<Database, PostgresError> {



        Ok(Database {
            //client: None,  
            migrations_dir_path,
            connection_url:  conn_url.clone() ,
            max_reconnect_attempts: 3 ,
            timeout_duration: Duration::from_secs( 5 )
        })

    }

    pub async fn connect(
       // credentials: DatabaseCredentials,
       &  self 
    ) -> Result<Client, PostgresError> {
        // Define the connection URL.
       // let conn_url = credentials.build_connection_url();

        info!("Connecting to db: {}", self.connection_url);

        let (client, connection) = tokio_postgres::connect(&self.connection_url, NoTls).await?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            //this is a blocking call i think !!!
            if let Err(e) = connection.await {
                eprintln!("postgres connection error: {}", e);
            }
        });

     //   self.client = Some(client);

        Ok( client )
    }



    pub async fn reconnect(&mut self  ) -> Result<Option< Client >, PostgresError> {
        let max_retries = 5;
        let mut attempt = 0;
         let conn_url = self.connection_url.clone() ;

        while attempt < max_retries {
            info!("Attempt {}: Reconnecting to database...", attempt + 1);

            match tokio_postgres::connect(&conn_url, NoTls).await {
                Ok((client, connection)) => {
                    // Spawn a task to keep the connection alive
                    tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            eprintln!("postgres connection error: {}", e);
                        }
                    });

                //    self.client = Some(client); // Replace old client with the new one
                    info!("Reconnection successful.");
                    return Ok( Some(client) );
                }
                Err(e) => {
                  
                    attempt += 1;

                    if attempt == max_retries {
                        return Err( e.into() )
                    }
                      eprintln!("Reconnection failed: {}. Retrying...", e);
                    sleep(Duration::from_secs(2_u64.pow(attempt))).await; // Exponential backoff
                }
            }
        }

       Ok(None)  //should error ? 
    }


    fn read_migration_files(migrations_dir_path: Option<String>) -> Migrations {
        let mut migrations = Migrations {
            up: Vec::new(),
            down: Vec::new(),
        };

        let migrations_dir =
            migrations_dir_path.unwrap_or("./src/db/postgres/migrations".to_string());
        let migration_dir_files = fs::read_dir(&migrations_dir).expect("Failed to read directory");

        for file in migration_dir_files {
            let file = file.expect("Failed to read migration file");

            let path = file.path();
            let filename = path.file_stem().unwrap().to_str().unwrap();

            let filename_without_extension: &str = filename.split('.').next().unwrap();

            // Read file contents
            let contents = fs::read_to_string(file.path()).expect("Failed to read file contents");

            //let contents = str::from_utf8(file.contents()).unwrap();

            info!("File name: {}", filename);

            if filename.contains(".down") {
                info!("File contents: {}", contents);
                migrations
                    .down
                    .push((filename_without_extension.into(), contents.clone()));
            }

            if filename.contains(".up") {
                info!("File contents: {}", contents);
                migrations
                    .up
                    .push((filename_without_extension.into(), contents.clone()));
            }
        }
        
            
        // Sort `up` migrations in ascending alphabetical order
        migrations.up.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Sort `down` migrations in descending alphabetical order
        migrations.down.sort_by(|a, b| b.0.cmp(&a.0));
                

        return migrations;
    }

    pub async fn migrate(&mut self) -> Result<(), Box<dyn Error>> {
        let client = &mut self.connect().await?;

        let migrations_dir_path = self.migrations_dir_path.clone();
        let mut migrations: Migrations = Self::read_migration_files(migrations_dir_path);

        for up_migration in migrations.up.drain(..) {
            println!("migrating {} {} ", up_migration.0, up_migration.1);
            let migration = Migration::new("migrations".to_string());

            // execute non existing migrations
            migration.up(client, &[up_migration.to_str()]).await?;
        }

        // ...
        Ok(())
    }

    //basically need to do the DOWN migrations and also delete some records from the migrations table
    //need to read from the migrations table with MigrationsModel::find
    pub async fn rollback(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub async fn rollback_full(&mut self) -> Result<(), Box<dyn Error>> {
        let migrations_dir_path = self.migrations_dir_path.clone();

        let mut migrations: Migrations = Self::read_migration_files(migrations_dir_path);

            let client = &mut self.connect().await?;

        for down_migration in migrations.down.drain(..)
         {
            println!("migrating {}", down_migration.0);
            let migration = Migration::new("migrations".to_string());
            // execute non existing migrations
            migration.down(client, &[down_migration.to_str()]).await?;
        }

        Ok(())
    }

     pub async fn query(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, PostgresError> {

        let client = &self.connect().await?;

        let rows = client.query(query, params).await?;
        Ok(rows)
    } 

   /*  pub async fn query_and_connect(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)] 
       
    ) -> Result<Vec<tokio_postgres::Row>, PostgresModelError> {

        let client = &mut self.connect().await?;

        let max_tries = self.max_reconnect_attempts; 
        let timeout_duration = self.timeout_duration;

        let mut attempts = 0;

        loop {

            attempts += 1; 

            let insert_result = timeout(
                timeout_duration,
                 client.query(query, params),
            ).await;

            match insert_result {
                Ok(Ok(rows)) => return Ok(rows),
                Ok(Err(e)) => {
                    eprintln!("Database error: {:?}", e);
                    return Err(e .into());
                },
                Err( _ ) => {
                    eprintln!("Database timeout occurred.");
                    let _reconnect_result = self.reconnect().await;

                    if attempts == max_tries {

                        return Err(PostgresModelError::Timeout ) ;
                    }
                    // After reconnection, the loop will continue to retry the query
                }
            }
        }
    }*/



    pub async fn query_one(
        & self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<tokio_postgres::Row, PostgresError> {

          let client = &self.connect().await?;

        let rows =  client.query_one(query, params).await?;
        Ok(rows)
    }

 
 /*   pub async fn query_one_with_reconnect(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
       
    ) -> Result<tokio_postgres::Row, PostgresModelError> {

        let timeout_duration = self.timeout_duration;

        let max_tries = self.max_reconnect_attempts; 
        let mut attempts = 0;

        loop {

            attempts += 1; 

            let insert_result = timeout(
                timeout_duration,
                self.client.query_one(query, params),
            ).await;

            match insert_result {
                Ok(Ok(row)) => return Ok(row),
                Ok(Err(e)) => {


                    eprintln!("Database error: {:?}", e);
                    return Err(e .into());
                },
                Err( _ ) => {
                    eprintln!("Database timeout occurred.");
                    let _reconnect_result = self.reconnect().await;

                    if attempts == max_tries {

                        return Err(PostgresModelError::Timeout ) ;
                    }
                    // After reconnection, the loop will continue to retry the query
                }
            }
        }
    }
*/

    //use for insert, update, etc
    pub async fn execute(
        &  self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, PostgresError> {

          let client = &self.connect().await?;

        

        let rows = client.execute(query, params).await?;
        Ok(rows)
    }
    
/*
    pub async fn execute_with_reconnect(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
        //timeout_duration: Duration,
    ) -> Result<u64, PostgresModelError> {

         let timeout_duration = self.timeout_duration;
         
        let max_tries = self.max_reconnect_attempts; 
        let mut attempts = 0;

        loop {

            attempts += 1; 

            let insert_result = timeout(
                timeout_duration,
                self.client.execute(query, params),
            ).await;

            match insert_result {
                Ok(Ok(row)) => return Ok(row),
                Ok(Err(e)) => {
                    eprintln!("Database error: {:?}", e);
                    return Err(e .into());
                },
                Err( _ ) => {
                    eprintln!("Database timeout occurred.");
                    let _reconnect_result = self.reconnect().await;

                    if attempts == max_tries {

                        return Err(PostgresModelError::Timeout ) ;
                    }
                    // After reconnection, the loop will continue to retry the query
                }
            }
        }
    }*/

    async fn atomic_transaction(
        &  self,
        steps: Vec<PostgresInput<'_>>,
    ) -> Result<(), PostgresError> {
          let client = &mut self.connect().await?;

        // Start a transaction
        let transaction = client.transaction().await?;

        //for each step in steps
        for step in steps {
            //execute the step
            let result = transaction.execute(&step.query, step.params).await;
            //check if the result is ok
            if result.is_err() {
                //if not rollback
                transaction.rollback().await?;
                //return error
                return Err(PostgresError::from(result.err().unwrap()));
            }
        }

        //if all steps are ok commit
        transaction.commit().await?;
        //return ok
        Ok(())
    }
}

pub fn try_get_option<'a, T: tokio_postgres::types::FromSql<'a>>(
    row: &'a tokio_postgres::Row,
    column: &str,
) -> Option<T> {
    match row.try_get::<&str, T>(column) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}
