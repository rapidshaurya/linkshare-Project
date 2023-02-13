
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use actix_identity::{IdentityMiddleware, config::LogoutBehaviour};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};

use actix_web::{cookie::Key, middleware::Logger, services, web, App, HttpServer};
use actix_web_lab::middleware::from_fn;
use utoipa_swagger_ui::SwaggerUi;
use anyhow::Result;
use crate::{ users,middleware_wraper};
use crate::routes::*;
use mongodb::Client;
use diesel::prelude::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use actix::SyncArbiter;
use crate::{DbActor, AppState};
use crate::queries::*;
pub async fn run(client: Client, pool: Pool<ConnectionManager<PgConnection>>, listner: String)->Result<(), std::io::Error>{
    let db_addr=SyncArbiter::start(5, move || DbActor(pool.clone()));
    
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

    
    //used for indexing
    create_username_index(&client).await;
    create_username_index_in_data(&client).await;
    create_friendname_index(&client).await;

    let secret_key = Key::generate();
    HttpServer::new(move || {
        let session_mw =
            SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                .cookie_name("auth-cookie".to_owned())
                // disable secure cookie for local testing
                .cookie_secure(false)
                .build();
        App::new()
            .wrap(IdentityMiddleware::builder().logout_behaviour(LogoutBehaviour::PurgeSession).build())
            .wrap(session_mw)
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(AppState{db: db_addr.clone()}))
            .wrap(Logger::default())
            .service(services![signin, signup, get_data])
            .service(web::scope("/home")
            .wrap(
                from_fn(middleware_wraper)
            )
            .service(services![
                add_data,
                logout,
                prv_data,
                access_prv_data,
                deleteuser,
                delete_one_doc,
                delete_all_doc,
                update_data,
                mylinks,
                
            ]))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi.clone()),
            )
    })
    .bind(listner)?
    .run()
    .await
}