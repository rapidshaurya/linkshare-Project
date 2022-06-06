use actix_web::{get, web, HttpResponse};
use mongodb::{bson::doc, Client, Collection, IndexModel};
use serde::{Deserialize, Serialize};
use actix_identity::{Identity};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Content {
    pub username: String,
    pub content_type: String,
    pub description: String,
    pub links: String,
    pub visibility: bool                    // for public visibility value is true else it's value is false
}

const DB_NAME: &str = "linkshare";
const COLL_NAME: &str = "link";

// Adds a new user to the "users" collection in the database.
#[get("/home/add")]
pub async fn add_data(id: Identity, client: web::Data<Client>, form: web::Form<Content>) -> HttpResponse {

    if let Some(_id) = id.identity() {
        

      let collection = client.database(DB_NAME).collection(COLL_NAME);
      let result = collection.insert_one(form.into_inner(), None).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("Data added Successfully!!!!!!"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    }
    else {
         HttpResponse::Ok().body("Go to signin page")
    }
}

//Gets the user with the supplied username.
#[get("/home/display/{username}")]
pub async  fn get_data(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
    let username = username.into_inner();
    let vs=true;
    let collection: Collection<Content> = client.database(DB_NAME).collection(COLL_NAME);
    let cursor = collection
        .find(doc! { "username": &username, "visibility": vs}, None)
        .await;
    HttpResponse::Ok().body(format!("Result "))
    
}

// Creates an index on the "username" field to force the values to be unique.
pub async fn create_username_index_in_data(client: &Client) {
    let model = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .build();
    client
        .database(DB_NAME)
        .collection::<Content>(COLL_NAME)
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}

