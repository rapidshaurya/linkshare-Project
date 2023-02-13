
use diesel::{prelude::*};
use serde::{Serialize, Deserialize};
use actix::Handler;
use crate::queries::messages::*;
use diesel::prelude::QueryResult;
use diesel::{RunQueryDsl, Insertable};
use crate::schema::{persons::dsl::*, persons};
use actix::{Actor, Addr, SyncContext};
use uuid::Uuid;
use diesel::r2d2::{ConnectionManager, Pool};
#[derive(Debug, Queryable, Serialize, Deserialize )]
pub struct Persons{
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub id:Uuid,
    pub username:String,
    #[serde(skip_serializing)]
    pub password:String
}

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name=persons)]
pub struct NewPerson{
    pub username:String,
    #[serde(skip_serializing)]
    pub password:String
}

pub struct AppState{
    pub db: Addr<DbActor>
}
pub struct DbActor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbActor {
    type Context = SyncContext<Self>;
}

impl Handler<FetchPersonsAll> for DbActor{
    type Result = QueryResult<Vec<Persons>>;

    fn handle(&mut self, _msg: FetchPersonsAll, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.0.get().expect("Fetch User: Unable to establish connection");
        
    return persons.get_results::<Persons>(&mut conn);

    }
}

impl Handler<CreatePerson> for DbActor{
    type Result = QueryResult<Persons>;

    fn handle(&mut self, msg: CreatePerson, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.0.get().expect("Fetch User: Unable to establish connection");
        let new_person= NewPerson{
            username:msg.username,
            password:msg.password
        };
        diesel::insert_into(persons)
        .values(new_person)
        .returning((
            id,
            username,
            password
        )).get_result::<Persons>(&mut conn)
    }
}


impl Handler<FetchOnePersons> for DbActor {
    type Result =  QueryResult<Persons>;

    fn handle(&mut self, msg: FetchOnePersons, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.0.get().expect("Fetch User: Unable to establish connection");

        persons.filter(username.eq(msg.username)).get_result::<Persons>(&mut conn)
    }
}

impl Handler<UpdatePerson> for DbActor{
    type Result = QueryResult<usize>;
    fn handle(&mut self, msg: UpdatePerson, _ctx: &mut Self::Context) -> Self::Result {
        let mut conn = self.0.get().expect("Fetch User: Unable to establish connection");

        diesel::update(persons.filter(username.eq(msg.username))).set(password.eq(msg.password)).execute(& mut conn)
    }
    
}