use std::env;
use mongodb::{options::ClientOptions, Client};
use tracing::{info};
use tracing_subscriber::EnvFilter;
pub struct ConfigConn {}

impl ConfigConn {
    pub fn new() -> String {
        dotenv::dotenv().ok();
        tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
        info!("loading env");
        let host = env::var("HOST").expect("Failed to load host from env");
        let port = env::var("PORT").expect("Failed to load port from env");
        let listner=format!("{}:{}",host,port);
        info!("connected with host {}", listner.clone());
        listner
    }

   
    pub async fn connect2_mongodb()-> Client{
        
        info!("Connecting to db!!!");
        dotenv::dotenv().ok();
        let url = env::var("MONGO_DB_URL").expect("failed to load db url from");
        let client_options = ClientOptions::parse(
            url,
        )
        .await
        .expect("fail to connect with db");
        info!("db connection successfull!!!");
        Client::with_options(client_options).expect("failed to handle the database")
    }
}





