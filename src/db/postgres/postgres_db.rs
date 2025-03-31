use deadpool_postgres::Timeouts;
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

     pool: deadpool_postgres::Pool, // Or other pool implementation

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
        max_pool_connections: usize, 
        migrations_dir_path: Option<String>,
    ) -> Result<Database, PostgresModelError> {
        // Parse the connection config from the URL
        let config: tokio_postgres::Config = conn_url.parse()
            .map_err(|_e| PostgresModelError::ConnectionFailed )?;
            
        // Create a manager using the config
        let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);

/*
        let deadpool_timeouts = Timeouts {
            create: Some(Duration::from_secs(5)),
            recycle: Some(Duration::from_secs(5)),
            wait: Some(Duration::from_secs(5))
        };  */
        
        // Create the pool with builder pattern
        let pool = deadpool_postgres::Pool::builder(manager)
            .max_size( max_pool_connections )
           // .timeouts( deadpool_timeouts ) 
            .build()
            .map_err(|e| PostgresModelError::PoolCreationFailed(e.to_string()))?;
 

        
        Ok(Database {
            pool,
            migrations_dir_path,
            connection_url: conn_url,
            max_reconnect_attempts: 3,
            timeout_duration: Duration::from_secs(5)
        })
    }
}


impl Database {

   

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
    ) -> Result<Vec<tokio_postgres::Row>, PostgresModelError> {

        /*let client = &self.connect().await?;

        let rows = client.query(query, params).await?;
        Ok(rows)*/

         // Get a client from the pool
        let client = self.pool.get().await?;
        
        // Execute the query and let the client be dropped automatically afterward
        let rows = client.query(query, params).await?;
        
        Ok(rows)

    } 
 

    pub async fn query_one(
        & self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<tokio_postgres::Row, PostgresModelError> {
            /*
          let client = &self.connect().await?;

        let rows =  client.query_one(query, params).await?;
        Ok(rows)

        */


         let client = self.pool.get().await ?;
        
        // Execute the query and let the client be dropped automatically afterward
        let row = client.query_one(query, params).await?;
        
        Ok(row)
    }
 
    //use for insert, update, etc
    pub async fn execute(
        &  self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, PostgresModelError> {
        /*
          let client = &self.connect().await?;

        

        let rows = client.execute(query, params).await?;
        Ok(rows)*/


         // Get a client from the pool
        let client = self.pool.get().await?;
        
        // Execute the query and let the client be dropped automatically afterward
        let count = client.execute(query, params).await?;
        
        Ok(count)


    }   



    pub async fn check_connection(&self) -> Result<bool, PostgresModelError> {
    // Get a client from the pool
    let client = self.pool.get().await?;
    
    // Execute a simple query to check the connection
    match client.execute("SELECT 1", &[]).await {
        Ok(_) => Ok(true),
        Err(e) => Err(PostgresModelError::from(e))
    }
}
    
    /*
    pub async fn recreate_pool(&mut self) -> Result<(), PostgresModelError> {
        // Parse the connection config from the URL
        let config: tokio_postgres::Config = self.connection_url.parse()
            .map_err(|_e| PostgresModelError::ConnectionFailed)?;
            
        // Create a manager using the config
        let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
        
        // Create a new pool
        let new_pool = deadpool_postgres::Pool::builder(manager)
            .max_size(16)
            .build()
            .map_err(|e| PostgresModelError::PoolCreationFailed(e.to_string()))?;
        
        // Replace the old pool
        self.pool = new_pool;
        
        Ok(())
    }

         
     pub async fn query_with_timeout(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, PostgresModelError> {
        match timeout(self.timeout_duration, self.query(query, params)).await {
            Ok(result) => result,
            Err(_) => Err(PostgresModelError::ConnectionFailed) // Timeout occurred
        }
    }

    pub async fn query_with_retry(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, PostgresModelError> {
        let mut attempts = 0;
        
        while attempts < self.max_reconnect_attempts {
            match self.query(query, params).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // If it's a connection error, retry
                    if let PostgresModelError::PoolError(_) = e {
                        attempts += 1;
                        if attempts >= self.max_reconnect_attempts {
                            return Err(e);
                        }
                        // Wait before retrying
                        sleep(Duration::from_secs(2_u64.pow(attempts))).await;
                    } else {
                        // For other errors, return immediately
                        return Err(e);
                    }
                }
            }
        }
        
        Err(PostgresModelError::ConnectionFailed)
    }*/

 
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
