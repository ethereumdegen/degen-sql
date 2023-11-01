## Degen Sql 

An opinionated postgres driver for rust and a light wrapper around tokio-postgres. 

Easily establish a postgres connection and run migrations from a folder.  
 
 
 
```

cargo add degen-sql

```

 
#### A note about the migrate and rollback_full scripts 

Since you cannot run scripts of dependencies, it is recommended that you copy and paste the 'scripts' folder from this repo into your project and then set up binaries in your Cargo.toml to run them. 

Then, you will have a migrate script and a rollback script to use in your project. 




### Step 1.
Set your env vars for postgres 

```


DB_HOST="db.co....blb.supabase.co"

DB_USER="postgres"
DB_NAME="postgres"

DB_PASSWORD="Foo....baR"


```



### Step 2.
Use in your code 




```

 
 
use degen_sql::db::postgres::postgres_db::{Database,DatabaseCredentials};
  

use dotenvy::dotenv;
use std::env;
 
 
async fn main() -> io::Result<()> {
    dotenv().ok();  //you dont HAVE to load them in like this but this is typical 

   
 
   
    let database_credentials = DatabaseCredentials::from_env();   //or you can use DatabaseCredentials { ... } and create the struct manually
  
    let database = Arc::new(
        Database::connect(
        database_credentials, None
    ).await.unwrap());
      
  
    //EXAMPLE USING THE DATABASE CONNECTION WITH ACTIX 
     
    HttpServer::new(move || {
        

        let app_state = AppState {
            database: Arc::clone(&database) 
        };

        App::new()
            .app_data(Data::new(app_state)) // Here is where we inject our database for our endpoints to use 
             
            .configure(  ...routes ...   )
             
            
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}




```