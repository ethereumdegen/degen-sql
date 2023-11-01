## Degen Sql 

An opinionated postgres driver for rust.  This package helps you easily establish a postgres  connection, run migrations from a folder, and run scripts.  




### Step 1.
Set your env vars (can use .env)

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