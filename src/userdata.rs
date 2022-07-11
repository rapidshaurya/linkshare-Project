use actix_web::{get, post, web, HttpResponse};
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};
use serde::{Deserialize, Serialize};
use base64::encode;
use actix_identity::{Identity};
use chrono::prelude::*;

// using User struct to store user data in "user" collection
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize,)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub password: String  
}

// using Info struct to store sign username and password. 
#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub username: String,
    pub password: String
}

// using Access struct to store which user is giving access to other user, for get access of private links of user
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Access {
    pub my_username: String,
    pub friend_username: String
}

//name of some collection and database
const DB_NAME: &str = "linkshare";
const COLL_NAME1: &str = "user";
const COLL_NAME2: &str = "access";

// Adds a new user to the "user" collection in the database.
#[post("/signup")]
pub async fn signup(client: web::Data<Client>, mut form: web::Form<User>) -> HttpResponse {
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
        Ok(_) => HttpResponse::Ok().body("Welcome to linkshare\n go to signin page"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// this routes handles login part of my project.
#[post("/signin")]
pub async  fn signin(id: Identity, client: web::Data<Client>, form: web::Form<Info>) -> HttpResponse {
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
            HttpResponse::Found().body(format!("Welcome {}", username)) 
        } 
        Ok(None) => {
            HttpResponse::NotFound().body(format!("Invalid username or password"))
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// this route handles the logout part of my project
#[get("/logout")]
async fn logout(id: Identity) -> HttpResponse {
    // remove identity
    if let Some(_id) = id.identity() {
        id.forget();
        HttpResponse::Found().body(format!("logout Successfully")) 
    } else {
        HttpResponse::Found().body(format!("Go to signin page")) 
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
#[get("/Home/giveaccess")]
pub async fn prv_data(id: Identity, client: web::Data<Client>,mut form: web::Form<Access>) -> HttpResponse {
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
      HttpResponse::Ok().body("Go to signin page")
    }
    
}

