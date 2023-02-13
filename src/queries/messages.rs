use crate::queries::db_models::Persons;
use actix::Message;
use diesel::QueryResult;


#[derive(Message)]
#[rtype(result = "QueryResult<Vec<Persons>>")]
pub struct FetchPersonsAll;


#[derive(Message)]
#[rtype(result = "QueryResult<Persons>")]
pub struct CreatePerson{
    pub id:uuid::Uuid,
    pub username:String,
    pub password:String,
}

#[derive(Message)]
#[rtype(result = "QueryResult<Persons>")]
pub struct FetchOnePersons{
    pub username:String
}

#[derive(Message)]
#[rtype(result = "QueryResult<usize>")]
pub struct UpdatePerson{
    pub username:String,
    pub password:String,
}