use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{services, web, App, HttpServer};
use linkshare::*;


use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

// main function used to declare all routes and helps in establishing connection to database
#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
            get_data
                ),
                components(
                    schemas(Access, Content, User, Info)
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

    let client = connect2_mongodb().await;

    //used for indexing
    create_username_index(&client).await;
    create_username_index_in_data(&client).await;
    create_friendname_index(&client).await;
    HttpServer::new(move || {
        let policy = CookieIdentityPolicy::new(&[0; 32])
            .name("auth-cookie")
            .secure(false);
        App::new()
            .app_data(web::Data::new(client.clone()))
            .wrap(IdentityService::new(policy))
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
                get_data
            ])
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi.clone()),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
