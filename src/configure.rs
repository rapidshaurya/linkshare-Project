

use eyre::WrapErr;
use color_eyre::Result;
use mongodb::{options::ClientOptions, Client};

use tracing::{instrument};
use serde::Deserialize;
use diesel::prelude::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};


#[derive(Debug, Deserialize)]
pub struct Configure {
    pub host:String,
    pub port:u32,
    pub mongo_db_url:String,
    pub postgres_db_url:String,
}



impl Configure {
    #[instrument(name = "load env")]
    pub fn load_env() -> Result<Configure> {
        dotenv::dotenv().ok();
        let c = config::Config::builder()
        .add_source(
            config::Environment::with_prefix("APP").try_parsing(true),
        )
        .build()
        .unwrap();
        
        let conf =c.try_deserialize::<Configure>().wrap_err("failed to load env");
        
         return conf;
    }

    #[instrument(name = "connect2Db", skip_all)]
    pub async fn connect2_mongodb(&self)-> Result<Client>{
        
        let client_options = ClientOptions::parse(
            self.mongo_db_url.clone(),
        )
        .await
        .wrap_err("failed to connect with db")?;
        
        Client::with_options(client_options).wrap_err("failed to handle the database")
    }

    #[instrument(name = "connect2Postgres", skip_all)]
    pub async fn connect2_postgres(&self)->Result<Pool<ConnectionManager<PgConnection>>>{

        let manager= ConnectionManager::<PgConnection>::new(&self.postgres_db_url);
        Pool::builder().build(manager).wrap_err("Error building in connection pool")
        
    
    }
}





