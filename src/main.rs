use actix_web::{get, web, App, HttpServer, HttpResponse, services};
use mongodb::{ bson::doc,Client, options::ClientOptions, Collection};
use actix_identity::{ Identity, CookieIdentityPolicy, IdentityService};
mod login;
mod userdata;
mod dataa;

pub use login::*;
pub use userdata::*;
pub use dataa::*;

const DB_NAME: &str = "linkshare";
const COLL_NAME2: &str = "access";
const COLL_NAME3: &str = "link";


#[get("/Home/{user}/{view}")]
pub async fn access_prv_data(id: Identity, client: web::Data<Client>, path: web::Path<(String, String)>) -> HttpResponse {
    if let Some(_id) = id.identity() {
        
     let (user, view) = path.into_inner();
     println!("hi");
     let collection2: Collection<Access> = client.database(DB_NAME).collection(COLL_NAME2);
     let collection3: Collection<Content> = client.database(DB_NAME).collection(COLL_NAME3);
     match collection2
                .find_one(doc! { "my_username": &view, "friend_username":&user}, None)
                .await
                {
                 Ok(Some(_user)) =>    { match collection3
                                        .find_one(doc! { "username": &view }, None)
                                        .await
                                        {
                                            Ok(Some(data)) => HttpResponse::Ok().json(data),
                                            Ok(None) => {HttpResponse::NotFound().body(format!("No Content")) },
                                            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                                        } },
                Ok(None) => {HttpResponse::NotFound().body(format!("No Content available ")) },
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                 }
    }
    else {
        HttpResponse::Ok().body("Go to signin page")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let  client_options = ClientOptions::parse("mongodb+srv://{name}:<password>@cluster0.do1yg.mongodb.net/?retryWrites=true&w=majority").await.expect("fail to connect tp the server");
    let client = Client::with_options(client_options).expect("failed to handle the database");
     
    create_username_index(&client).await;
    create_username_index_in_data(&client).await;

    HttpServer::new(move || {
        let policy = CookieIdentityPolicy::new(&[0; 32])
        .name("auth-cookie")
        .secure(false);
        App::new()
            .app_data(web::Data::new(client.clone()))
            .wrap(IdentityService::new(policy))
            .route("/", web::get().to(login_form))
            .service(services![signin, add_data, logout, prv_data, access_prv_data])
            .service(signup)
            .service(get_data)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
