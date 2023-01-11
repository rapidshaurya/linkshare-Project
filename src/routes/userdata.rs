use actix_web::{get, post, web, HttpResponse, http::StatusCode};
use mongodb::{bson::{doc, Document}, options::IndexOptions, Client, Collection, IndexModel};
use base64::encode;
use actix_identity::Identity;
use chrono::prelude::*;
use tracing::warn;
use validator::{Validate};
use std::process;
use futures::StreamExt;
use tracing::instrument;
use crate::{Error,ErrorType};
use crate::routes::utils::*;
//name of some collection and database
const DB_NAME: &str = "linkshare";
const COLL_NAME1: &str = "user";
const COLL_NAME2: &str = "access";
const COLL_NAME3: &str = "link";

// using User struct to store user data in "user" collection


// Adds a new user to the "user" collection in the database.
#[utoipa::path(
    post,
    path = "/signup",
    responses(
        (status = 200, description = "Welcome to linkshare\n go to signin page"),
    ),
    request_body = User,
    
)]
#[instrument(name = "signup", skip_all)]
#[post("/signup")]
pub async fn signup(client: web::Data<Client>, mut form: web::Json<User>) -> Result<HttpResponse, Error>  {
    let a=form.validate();
    match a {
        Ok(())=>(),
        Err(e)=>{
            

            warn!("{:?}",e.field_errors());
            
    
            let error =Error::new(ErrorType::BADREQUEST("ValidationError"));
            return Err(error)}
    }
    let when =  Utc::now().to_string();
    form.password=encode(&form.password);
    let doc = doc! {
        "first_name": &form.first_name,
        "last_name": &form.last_name,
        "username": &form.username,
        "password": &form.password,
        "when" : when
    };
    let collection = client.database(DB_NAME).collection(COLL_NAME1);
    let result = collection.insert_one(doc, None).await;
    match result {
        Ok(_) => Ok(HttpResponse::Created().body("Welcome to linkshare\n go to signin page")),
        Err(err) => Ok(HttpResponse::Conflict().body(err.to_string())),
    }
}

#[utoipa::path(
    post,
    path = "/signin",
    responses(
        (status = 200, description = "Welcome to linkshare\n go to signin page"),
    ),
    security(
        (),
        ("auth-cookie" = ["read:items", "edit:items"]),
    ),
    request_body = LoginCred
)]
#[post("/signin")]
pub async  fn signin(id: Identity, client: web::Data<Client>, form: web::Json<LoginCred>) -> HttpResponse {
    let username = form.username.to_string();
    let password = encode(form.password.to_string()); // encode function is used to encode password

    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME1);
    let ans = collection
        .find_one(doc! { "username": &username, "password":&password}, None)
        .await;
    match ans
    {
        Ok(Some(_user)) =>  { 
            id.remember(username.to_owned());
            HttpResponse::Ok().body(format!("Welcome {}", username)) 
        } 
        Ok(None) => {
            HttpResponse::Unauthorized().body(format!("Invalid username or password"))
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// this route handles the logout part of my project

#[utoipa::path(
    post,
    path = "/logout",
    responses(
        (status = 200, description = "logout Successfully"),
    ),
    security(
                 (),
                 ("auth-cookie" = ["read:items", "edit:items"]),
             ),
)]
#[post("/logout")]
async fn logout(id: Identity) -> HttpResponse {
    // remove identity
    if let Some(_id) = id.identity() {
        id.forget();
        HttpResponse::Accepted().body(format!("logout Successfully")) 
    } else {
        HttpResponse::Ok().status(StatusCode::UNAUTHORIZED).body("invalid token") 
    }
    
}

// Creates an index on the "username" field to force the values to be unique.
pub async fn create_username_index(client: &Client) {
    let options1 = IndexOptions::builder().unique(true).build();
    
    let model1 = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(options1)
        .build();
    client
        .database(DB_NAME)
        .collection::<User>(COLL_NAME1)
        .create_index(model1, None)
        .await
        .expect("creating an index should succeed");
    
}

pub async fn create_friendname_index(client: &Client) {
    let options2 = IndexOptions::builder().unique(true).build();
    let model2 = IndexModel::builder()
        .keys(doc! { "friend_username": 1, "username": 1 })
        .options(options2)
        .build();
    client
        .database(DB_NAME)
        .collection::<User>(COLL_NAME2)
        .create_index(model2, None)
        .await
        .expect("creating an index should succeed");
}

// this route is user to store "access" collection data 

#[utoipa::path(
    post,
    path = "/Home/giveaccess",
    responses(
        (status = 200, description = "Now he can access your private links"),
    ),
    request_body = Access,
    security(
                 (),
                 ("auth-cookie" = ["read:items", "edit:items"]),
             ),
)]
#[post("/Home/giveaccess")]
pub async fn prv_data(id: Identity, client: web::Data<Client>,mut form: web::Json<Access>) -> HttpResponse {
    if let Some(id) = id.identity() {
        form.my_username=id;
        let collection = client.database(DB_NAME).collection(COLL_NAME2);
        let result = collection.insert_one(form.into_inner(), None).await;
        match result {
            Ok(_) => HttpResponse::Ok().body(format!("Now he can access your private links")),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    }
    else {
        HttpResponse::Ok().status(StatusCode::UNAUTHORIZED).body("invalid token") 
    }
    
}

#[utoipa::path(
    get,
    path = "/Home/{user}",
    responses(
        (status = 200, description = "Now he can access your private links"),
    ),
    security(
                 (),
                 ("auth-cookie" = ["read:items"]),
             ),
)]
#[get("/Home/{user}")]
pub async fn access_prv_data(id: Identity, client: web::Data<Client>, path: web::Path<String>) -> HttpResponse {
    if let Some(id) = id.identity() {
     let view =id;   
     let user = path.into_inner();
     let collection2: Collection<Access> = client.database(DB_NAME).collection(COLL_NAME2);
     let collection3: Collection<Document> = client.database(DB_NAME).collection(COLL_NAME3);
     match collection2
                .find_one(doc! { "my_username": &view, "friend_username":&user}, None)
                .await
                {
                 Ok(Some(_user)) =>    { let cur = collection3
                                        .find(doc! { "username": &view }, None)
                                        .await;
                                        let mut cursor = match cur { //cursor: Cursor<Document>
                                            Ok(x) => x,
                                            Err(_) => process::exit(1)
                                        };
                                        let mut ans:Vec<PubContent> = Vec::new();
                                        while let Some(doc) = cursor.next().await {
                                        let a = doc.unwrap();
                                        
                                        ans.push(PubContent{
                                        id:a.get_object_id("_id").unwrap().to_string(),    
                                        username:a.get_str("username").unwrap().to_string(),
                                           content_type: a.get_str("content_type").unwrap().to_string(),
                                           description: a.get_str("description").unwrap().to_string(),
                                           links: a.get_str("links").unwrap().to_string()
        })
    }

    HttpResponse::Ok().json(serde_json::json!({ "result": ans }))
                                         },
                Ok(None) => {HttpResponse::NoContent().body(format!("No Content available ")) },
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                 }
    }
    else {
        HttpResponse::Ok().status(StatusCode::UNAUTHORIZED).body("invalid token")
    }
}



#[utoipa::path(
    post,
    path = "/Home/delete/{ans}",
    responses(
        (status = 200, description = "Now he can access your private links"),
    ),
    security(
                 (),
                 ("auth-cookie" = ["read:items", "edit:items"]),
             ),
)]
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
            HttpResponse::Ok().body("Permission Required")
        }
    }
    else{
        HttpResponse::Ok().status(StatusCode::UNAUTHORIZED).body("invalid token")
    }
    
}
