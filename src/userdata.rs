use actix_web::{get, post, web, HttpResponse};
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};
use serde::{Deserialize, Serialize};
use actix_identity::{Identity};
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub username: String,
    pub password: String
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Access {
    pub my_username: String,
    pub friend_username: String
}

const DB_NAME: &str = "linkshare";
const COLL_NAME1: &str = "user";
const COLL_NAME2: &str = "access";

// Adds a new user to the "users" collection in the database.
#[post("/signup")]
pub async fn signup(client: web::Data<Client>, form: web::Form<User>) -> HttpResponse {
    let collection = client.database(DB_NAME).collection(COLL_NAME1);
    let result = collection.insert_one(form.into_inner(), None).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("Welcome to linkshare\n go to signin page"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// Gets the user with the supplied username.
#[post("/signin")]
pub async  fn signin(id: Identity, client: web::Data<Client>, form: web::Form<Info>) -> HttpResponse {
    let username = form.username.to_string();
    let password = form.password.to_string();


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
//not working
#[post("/deleteuser")]
pub async  fn delete_user(client: web::Data<Client>, info: web::Form<Info>) -> HttpResponse {
    let username = info.username.to_string();
    let password = info.password.to_string();
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME1);
    let _delete = collection
        .delete_one(doc! { "username": &username, "password":&password}, None)
        .await;
    HttpResponse::Ok().body(format!("Deleted"))
}

#[post("/logout")]
async fn logout(id: Identity) -> HttpResponse {
    // remove identity
    id.forget();
    HttpResponse::Ok().finish()
}

// Creates an index on the "username" field to force the values to be unique.
pub async fn create_username_index(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(options)
        .build();
    client
        .database(DB_NAME)
        .collection::<User>(COLL_NAME1)
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}

#[get("/Home/giveaccess")]
pub async fn prv_data(id: Identity, client: web::Data<Client>, form: web::Form<Access>) -> HttpResponse {
    if let Some(_id) = id.identity() {
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

