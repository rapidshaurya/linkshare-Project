use anyhow::Ok;
use linkshare::{run, Configure};
use tracing_subscriber::EnvFilter;

// main function used to declare all routes and helps in establishing connection to database
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let config=Configure::load_env().expect("failed to load env");
    tracing::info!("Env loaded successfully");
    let client = config
        .connect2_mongodb()
        .await
        .expect("failed to connect with db");
    tracing::info!("db connection successfull!!!");
    
    let pool = config
        .connect2_postgres()
        .await
        .expect("failed to connect to postgress");
    tracing::info!("postgress db connection successfull!!!");
    let listner = format!("{}:{}", config.host, config.port);

    if let Err(e) = run(client, pool, listner).await {
        tracing::error!("App stopped due to error {}: ", e);
    }

    Ok(())
}
