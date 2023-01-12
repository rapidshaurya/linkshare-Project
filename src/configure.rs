
use eyre::WrapErr;
use color_eyre::Result;
use mongodb::{options::ClientOptions, Client};
use tracing::{info, instrument};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configure {
    pub host:String,
    pub port:u32,
    pub mongo_db_url:String
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
        info!("Env loaded successfully");
         return conf;
    }

    #[instrument(name = "connect2Db", skip_all)]
    pub async fn connect2_mongodb(&self)-> Result<Client>{
        
        let client_options = ClientOptions::parse(
            self.mongo_db_url.clone(),
        )
        .await
        .wrap_err("failed to connect with db")?;
        info!("db connection successfull!!!");
        Client::with_options(client_options).wrap_err("failed to handle the database")
    }
}





