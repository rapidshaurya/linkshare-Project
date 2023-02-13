use actix_web::{get, post, web, HttpResponse, http::StatusCode ,HttpRequest, HttpMessage};
use mongodb::{bson::{doc, Document}, options::IndexOptions, Client, Collection, IndexModel};
use base64::encode;

use actix_identity::Identity;
use tracing::warn;
use validator::{Validate};
use std::process;
use futures::StreamExt;
use tracing::instrument;
use crate::{Error,ErrorType, FetchOnePersons};
use crate::routes::utils::*;

use data_encoding::HEXUPPER;
use ring::rand::SecureRandom;
use ring::{digest, pbkdf2, rand};
use std::num::NonZeroU32;


use crate::{AppState, DbActor, CreatePerson};
use actix::Addr;
//name of some collection and database
const DB_NAME: &str = "linkshare";
const COLL_NAME1: &str = "user";
const COLL_NAME2: &str = "access";
const COLL_NAME3: &str = "link";
const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
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
pub async fn signup(state: web::Data<AppState>, mut form: web::Json<User>) -> Result<HttpResponse, Error>  {
    let db:Addr<DbActor>=state.as_ref().db.clone();  
    let a=form.validate();
    match a {
        Ok(())=>(),
        Err(e)=>{
            

            warn!("{:?}",e.field_errors());
            
    
            let error =Error::new(ErrorType::BADREQUEST("ValidationError"));
            return Err(error)}
    }
    form.password=encode(&form.password);
    match db.send(CreatePerson{
        id:uuid::Uuid::new_v4(),
        username:form.username.clone(),
        password:form.password.clone()
      }).await
      {
          Ok(Ok(_info)) => Ok(HttpResponse::Ok().finish()),
          Ok(Err(e)) => {
            tracing::warn!("{}",e);
            Err(Error::new(ErrorType::BADREQUEST("User Already exisit")))
          },
            _ => {
                let error=Error::new(ErrorType::InternalServerError("Unable to retrieve users data"));
                return Err(error);},
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
pub async  fn signin(state: web::Data<AppState>, form: web::Json<LoginCred>, request: HttpRequest) -> Result<HttpResponse, Error> {
    let n_iter = NonZeroU32::new(100_000).unwrap();
    let rng = rand::SystemRandom::new();

    let mut salt = [0u8; CREDENTIAL_LEN];
    rng.fill(&mut salt).unwrap();

    
    let password = "Guess Me If You Can!";
    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        n_iter,
        &salt,
        password.as_bytes(),
        &mut pbkdf2_hash,
    );
    println!("Salt: {}", HEXUPPER.encode(&salt));
    println!("PBKDF2 hash: {}", HEXUPPER.encode(&pbkdf2_hash));

    let should_succeed = pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA512,
        n_iter,
        &salt,
        password.as_bytes(),
        &pbkdf2_hash,
    ).is_ok();
    if should_succeed{
        dbg!("success");
    }else{
        dbg!("false");
    }

    let username = form.username.to_string();
    let password = encode(form.password.to_string()); // encode function is used to encode password

    let db: Addr<DbActor> = state.as_ref().db.clone();

  match db.send(FetchOnePersons{
    username:username.clone()
  }).await
  {
      Ok(Ok(info)) => {
        if password.clone().eq(&info.password.clone()){
            Identity::login(&request.extensions(), username.to_owned()).unwrap();
            return Ok(HttpResponse::Ok().finish());
        }else {
            Err(Error::new(ErrorType::UNAUTHORIZED("Invalid user id or password")))
        }
    },
      Ok(Err(e)) => {
        {tracing::warn!("{}",e);
        return Err(Error::new(ErrorType::UNAUTHORIZED("Invalid user id or password")))}},
        _ => Ok(HttpResponse::InternalServerError().json("Unable to retrieve users")),
  }
}

// this route handles the logout part of my project

#[utoipa::path(
    post,
    path = "/home/logout",
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
    id.logout();
    HttpResponse::Accepted().body(format!("logout Successfully")) 
    
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
    path = "/home/giveaccess",
    responses(
        (status = 200, description = "Now he can access your private links"),
    ),
    request_body = Access,
    security(
                 (),
                 ("auth-cookie" = ["read:items", "edit:items"]),
             ),
)]
#[post("/giveaccess")]
pub async fn prv_data(_id: Option<Identity>, client: web::Data<Client>,mut form: web::Json<Access>) -> HttpResponse {
    if let Some(id) = _id {
        form.my_username=id.id().unwrap();
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
    path = "/home/{user}",
    responses(
        (status = 200, description = "Now he can access your private links"),
    ),
    security(
                 (),
                 ("auth-cookie" = ["read:items"]),
             ),
)]
#[get("/{user}")]
pub async fn access_prv_data(id: Option<Identity>, client: web::Data<Client>, path: web::Path<String>) -> HttpResponse {
    if let Some(_id) = id {
     let view =_id.id().unwrap();   
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
    path = "/home/delete/{ans}",
    responses(
        (status = 200, description = "Now he can access your private links"),
    ),
    security(
                 (),
                 ("auth-cookie" = ["read:items", "edit:items"]),
             ),
)]
#[post("/delete/{ans}")]
pub async fn deleteuser(id: Option<Identity>, client: web::Data<Client>,  path: web::Path<String>)  -> HttpResponse {
    if let Some(_id) = id {
        let user=_id.id().unwrap();
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
            
            _id.logout();
            HttpResponse::Ok().body("Account Deleted")
        }
        else {
            _id.logout();
            HttpResponse::Ok().body("Permission Required")
        }
    }
    else{
        HttpResponse::Ok().status(StatusCode::UNAUTHORIZED).body("invalid token")
    }
    
}
