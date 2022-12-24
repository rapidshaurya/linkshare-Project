
use mongodb::{options::ClientOptions, Client};
use std::env;
pub async fn connect2_mongodb()-> Client{
    dotenv::dotenv().ok();
    let url = env::var("MONGO_DB_URL").unwrap();
    let client_options = ClientOptions::parse(
        url,
    )
    .await
    .expect("fail to connect tp the server");

    Client::with_options(client_options).expect("failed to handle the database")
}