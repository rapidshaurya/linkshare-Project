


use actix_web::{web, HttpResponse};
use crate::queries::messages::*;
use crate::{AppState, DbActor, CreatePerson};
use actix::Addr;


pub async fn users(state: web::Data<AppState>)->HttpResponse{
   let db:Addr<DbActor>=state.as_ref().db.clone();  
   match db.send(FetchPersonsAll).await {
      Ok(Ok(info)) => HttpResponse::Ok().json(info),
      Ok(Err(_)) => HttpResponse::NotFound().json("No users found"),
      _ => HttpResponse::InternalServerError().json("Unable to retrieve users"),
  }

}



pub async fn search(state: web::Data<AppState>, user: web::Path<String>)->HttpResponse{
  let id = user.into_inner();
  // format!("POST /users/{id}/articles")

  let db: Addr<DbActor> = state.as_ref().db.clone();

  match db.send(FetchOnePersons{
    username:id.clone()
  }).await
  {
      Ok(Ok(info)) => HttpResponse::Ok().json(info),
      Ok(Err(e)) => {
        tracing::warn!("{}",e);
        HttpResponse::NotFound().json("No users found")},
        _ => HttpResponse::InternalServerError().json("Unable to retrieve users"),
  }
}


pub async fn new_user(state: web::Data<AppState>, user: web::Path<String>)->HttpResponse{
   let id = user.into_inner();
    // format!("POST /users/{id}/articles")

    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db.send(CreatePerson{
      id:uuid::Uuid::new_v4(),
      username:id.clone(),
      password:"abc".to_owned()
    }).await
    {
        Ok(Ok(info)) => HttpResponse::Ok().json(info),
        Ok(Err(e)) => {
          tracing::warn!("{}",e);
          HttpResponse::AlreadyReported().json("alreadt exist")},
          _ => HttpResponse::InternalServerError().json("Unable to retrieve users"),
    }
}




pub async fn update_user(state: web::Data<AppState>, user: web::Path<String>)->HttpResponse{
   let id = user.into_inner();
    // format!("POST /users/{id}/articles")

    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db.send(UpdatePerson{
      username:id.clone(),
      password:"12345678".to_owned() 
    }).await
    {
        Ok(Ok(info)) => HttpResponse::Ok().json(info),
        Ok(Err(e)) => {
          tracing::warn!("{}",e);
          HttpResponse::AlreadyReported().json("alreadt exist")},
          _ => HttpResponse::InternalServerError().json("Unable to retrieve users"),
    }
}