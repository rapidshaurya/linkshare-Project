use actix_identity::{IdentityMiddleware};
use actix_session::{ SessionMiddleware, storage::CookieSessionStore};

use actix_web::{cookie::Key, middleware::Logger, services, web, App, HttpServer};
use actix_web_lab::middleware::from_fn;
use linkshare::*;
use tracing_subscriber::EnvFilter;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;
// main function used to declare all routes and helps in establishing connection to database
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();
    #[derive(OpenApi)]
    #[openapi(
        paths(
            signin,
            add_data,
            logout,
            prv_data,
            access_prv_data,
            deleteuser,
            delete_one_doc,
            delete_all_doc,
            update_data,
            signup,
            get_data,
            mylinks,
                ),
                components(
                    schemas(Access, Content, User, LoginCred)
                ),
                tags(
                    (name = "LINKSHARE", description = "Link management endpoints.")
                ),
        modifiers(&SecurityAddon)
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("auth-cookie"))),
            )
        }
    }

    // Make instance variable of ApiDoc so all worker threads gets the same instance.
    let openapi = ApiDoc::openapi();

    let listner = ConfigConn::new();
    let client = ConfigConn::connect2_mongodb().await;
    //used for indexing
    create_username_index(&client).await;
    create_username_index_in_data(&client).await;
    create_friendname_index(&client).await;
    
    let secret_key = Key::generate();
    HttpServer::new(move || {
        let session_mw =
            SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone()).cookie_name("auth-cookie".to_owned())
                // disable secure cookie for local testing
                .cookie_secure(false)
                .build();
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(session_mw)
            .app_data(web::Data::new(client.clone()))
            .wrap(Logger::default())
            .wrap(from_fn(middleware_wraper))
            .service(services![
                signin,
                add_data,
                logout,
                prv_data,
                access_prv_data,
                deleteuser,
                delete_one_doc,
                delete_all_doc,
                update_data,
                signup,
                get_data,
                mylinks
            ])
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi.clone()),
            )
    })
    .bind(listner)?
    .run()
    .await
}
