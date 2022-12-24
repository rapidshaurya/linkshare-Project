use actix_identity::Identity;
use actix_web::{get, http::StatusCode, web, HttpResponse, post};
use chrono::prelude::*;
use futures::StreamExt;
use mongodb::{
    bson::{doc, Document},
    Client, Collection, IndexModel,
};

use std::process;
use crate::routes::utils::*;

// name of collections
const DB_NAME: &str = "linkshare";
const COLL_NAME: &str = "link";

// Adds data to the "link" collection in the database.

#[utoipa::path(
    post,
    path = "/home/add",
    request_body = Content,
    security(
        (),
        ("auth-cookie" = ["read:items"]),
    ),
)]
#[post("/home/add")]
pub async fn add_data(
    id: Identity,
    client: web::Data<Client>,
    form: web::Json<Content>,
    
) -> HttpResponse {
    if let Some(id) = id.identity() {
        let when = Utc::now().to_string();
        let doc = doc! {
            "username": &id,
            "content_type": &form.content_type,
            "description": &form.description,
            "links": &form.links,
            "visibility": &form.visibility,
            "when" : when
        };

        let collection: Collection<Document> = client.database(DB_NAME).collection(COLL_NAME);
        let result = collection
            .find_one(
                doc! {
                "username": &id,
                "description": &form.description,
                "content_type": &form.content_type },
                None,
            )
            .await;
        match result {
            Ok(Some(_user)) => HttpResponse::Ok().body("Data Already added Successfully!!!!!!"),
            Ok(None) => {
                let result = collection.insert_one(doc, None).await;
                match result {
                    Ok(res) => HttpResponse::Ok().body(format!("{:?}", res)),
                    Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                }
            }
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .body("invalid token")
    }
}


#[utoipa::path(
    post,
    path = "/home/deletealldoc",
    security(
        (),
        ("auth-cookie" = ["read:items", "edit:items"]),
    ),
    
)]
// Delete all the doc which is stored by user in "link" collection
#[post("/home/deletealldoc")]
pub async fn delete_all_doc(id: Identity, client: web::Data<Client>) -> HttpResponse {
    if let Some(id) = id.identity() {
        let collection: Collection<Document> = client.database(DB_NAME).collection(COLL_NAME);
        let deleted = collection.delete_many(doc! { "username": id  }, None).await;
        println!("Deleted {:#?}", deleted);
        HttpResponse::Ok().body("Deleted Suceessfully")
    } else {
        HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .body("invalid token")
    }
}

#[utoipa::path(
    post,
    path = "/home/deletealldoc",
    request_body = Content,
    security(
        (),
        ("auth-cookie" = ["read:items", "edit:items"]),
    ),
    
)]
// Delete only one doc which is stored by user in "link" collection on the basis of username, description, and content type
#[post("/home/deleteonedoc")]
pub async fn delete_one_doc(
    id: Identity,
    client: web::Data<Client>,
    form: web::Json<Content>,
) -> HttpResponse {
    if let Some(id) = id.identity() {
        let collection: Collection<Document> = client.database(DB_NAME).collection(COLL_NAME);
        let deleted = collection
            .find_one_and_delete(
                doc! {
                "username": &id,
                "content_type": &form.content_type,
                "description": &form.description
                 },
                None,
            )
            .await;
        match deleted {
            Ok(Some(user)) => HttpResponse::Accepted().body(format!("Deleted Data is\n {:#?}", user)),
            Ok(None) => HttpResponse::NoContent().body(format!("No Content Available")),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .body("invalid token")
    }
}

// update only one doc which is stored by user in "link" collection on the basis of username, description, and content type

#[utoipa::path(
    post,
    path = "/home/update",
    request_body = Content,
    security(
        (),
        ("auth-cookie" = ["read:items"]),
    ),
    
)]

#[post("/home/update")]
pub async fn update_data(
    id: Identity,
    client: web::Data<Client>,
    form: web::Json<Content>,
) -> HttpResponse {
    if let Some(id) = id.identity() {
        let collection: Collection<Document> = client.database(DB_NAME).collection(COLL_NAME);
        let deleted =collection
                                        .update_one(doc! { "username": &id,"content_type": &form.content_type ,"description": &form.description  },
                                         doc!{ "$set":{
                                            "links": &form.links,
                                            "when":  Utc::now().to_string()
                                                    }
                                        } , None)
                                        .await;
        match deleted {
            Ok(_) => HttpResponse::Accepted().body("Data Updated Successfully!!!!!!"),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        }
    } else {
        HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .body("invalid token")
    }
}

#[utoipa::path(
    get,
    path = "/home/display/{username}",
    security(
        (),
        ("auth-cookie" = ["read:items"]),
    ),
    
)]
//display all the doc of user in "link" collection if the doc visbility is true(i.e., doc is public).
#[get("/home/display/{username}")]
pub async fn get_data(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
    let username = username.into_inner();
    let vs = true;
    let collection: Collection<Document> = client.database(DB_NAME).collection(COLL_NAME);
    let cur = collection
        .find(doc! { "username": &username, "visibility":&vs}, None)
        .await;

    let mut cursor = match cur {
        //cursor: Cursor<Document>
        Ok(x) => x,
        Err(_) => process::exit(1),
    };
    let mut ans:Vec<PubContent> = Vec::new();
    while let Some(doc) = cursor.next().await {
        let a = doc.unwrap();
        ans.push(PubContent{
            username:a.get_str("username").unwrap().to_string(),
            content_type:
            a.get_str("content_type").unwrap().to_string(),
            description:
            a.get_str("description").unwrap().to_string(),
            links:
            a.get_str("links").unwrap().to_string()
        })
    }

    HttpResponse::Ok().json(serde_json::json!({ "result": ans }))
}

// Creates an index on the "username" field.
pub async fn create_username_index_in_data(client: &Client) {
    let model = IndexModel::builder().keys(doc! { "username": 1 }).build();
    client
        .database(DB_NAME)
        .collection::<Document>(COLL_NAME)
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}
