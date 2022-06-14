use actix_web::{get, post, web, App, HttpServer, HttpResponse, services};
use mongodb::{ bson::doc,Client, options::ClientOptions, Collection};
use actix_identity::{ Identity, CookieIdentityPolicy, IdentityService};
use std::process;
use futures::TryStreamExt;

mod login;
mod userdata;
mod dataa;

pub use login::*;
pub use userdata::*;
pub use dataa::*;

const DB_NAME: &str = "linkshare";
const COLL_NAME1: &str = "user";
const COLL_NAME2: &str = "access";
const COLL_NAME3: &str = "link";

// this route is used to view private link of user(who have access to view private links)
#[get("/Home/{user}")]
pub async fn access_prv_data(id: Identity, client: web::Data<Client>, path: web::Path<String>) -> HttpResponse {
    if let Some(id) = id.identity() {
     let view =id;   
     let user = path.into_inner();
     let collection2: Collection<Access> = client.database(DB_NAME).collection(COLL_NAME2);
     let collection3: Collection<Content> = client.database(DB_NAME).collection(COLL_NAME3);
     match collection2
                .find_one(doc! { "my_username": &view, "friend_username":&user}, None)
                .await
                {
                 Ok(Some(_user)) =>    { let cur = collection3
                                        .find(doc! { "username": &view }, None)
                                        .await;
                                        let cursor = match cur { //cursor: Cursor<Document>
                                            Ok(x) => x,
                                            Err(_) => process::exit(1)
                                        };
                                       let doc = cursor.try_collect().await.unwrap_or_else(|_| vec![]);
                                       HttpResponse::Ok().body(format!("Result {:?}", doc))
                                         },
                Ok(None) => {HttpResponse::NotFound().body(format!("No Content available ")) },
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                 }
    }
    else {
        HttpResponse::Ok().body("Go to signin page")
    }
}

//this route is used to delete user and all data of user stored in collections
#[post("/Home/delete/{ans}")]
pub async fn deleteuser(id: Identity, client: web::Data<Client>,  path: web::Path<String>)  -> HttpResponse {
    if let Some(_id) = id.identity() {
        let user=_id;
        let ans = path.into_inner();
        if ans == "Yes" {
            let collection1: Collection<Access> = client.database(DB_NAME).collection(COLL_NAME1);
            let collection2: Collection<Content> = client.database(DB_NAME).collection(COLL_NAME2);
            let collection3: Collection<Content> = client.database(DB_NAME).collection(COLL_NAME3);
            let deleted1 =collection1
                                        .delete_one(doc! { "username": &user  }, None)
                                        .await;
            println!("{:?}", deleted1);
            
            let deleted2 =collection2
                                        .delete_many(doc! { "my_username": &user  }, None)
                                        .await;
            println!("{:?}", deleted2);
            let deleted3 =collection3
                                        .delete_many(doc! { "username": &user  }, None)
                                        .await;
            println!("{:?}", deleted3);
            
            id.forget();
            HttpResponse::Ok().body("Account Deleted")
        }
        else {
            id.forget();
            HttpResponse::Ok().body("Permission Resquired")
        }
        
    }
    else{
        HttpResponse::Ok().body("Go to signin page")
    }
    
}

// main function used to declare all routes and helps in establishing connection to database
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let  client_options = ClientOptions::parse("mongodb+srv://rapidshaurya:12345@cluster0.do1yg.mongodb.net/?retryWrites=true&w=majority").await.expect("fail to connect tp the server");
    let client = Client::with_options(client_options).expect("failed to handle the database");
    
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
            .route("/", web::get().to(login_form))
            .service(
                services![signin, add_data, logout, prv_data,
                 access_prv_data, deleteuser, delete_one_doc, delete_all_doc, update_data])
            .service(signup)
            .service(get_data)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}