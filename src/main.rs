mod author;
mod dispatcher;
mod error;
mod message;
mod schema;
mod websocket;

#[macro_use]
extern crate diesel;

use crate::author::JWTAuthor;
use crate::dispatcher::Dispatcher;
use crate::error::Error;
use crate::websocket::WS;
use actix::{self, Actor, Addr};
use actix_web::body::BoxBody;
use actix_web::web::{self, get, Data, Json, Path};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws::WsResponseBuilder;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    Connection, PgConnection,
};
use dotenv;
use message::InnerMessage;
use r2d2::{ManageConnection, PooledConnection};
use serde::{Deserialize, Serialize};

trait Author {
    fn auth(&self, db: PooledConnection<ConnectionManager<PgConnection>>, account: String, credential: String) -> Result<String, Error>;
    fn verify(&self, token: String) -> Result<String, Error>;
    fn signup(&self, db: PooledConnection<ConnectionManager<PgConnection>>, account: String, credential: String) -> Result<usize, Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

async fn login<A>(
    dispatcher: Data<Addr<Dispatcher>>,
    author: Data<A>,
    db: Data<Pool<ConnectionManager<PgConnection>>>,
    Json(data): Json<LoginRequest>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error>
where
    A: Author,
{
    let token = author.auth(db.get()?, data.username.clone(), data.password.clone())?;
    let (addr, resp) = WsResponseBuilder::new(
        WS {
            name: data.username.clone(),
            dispatcher: dispatcher.clone(),
        },
        &req,
        stream,
    )
    .start_with_addr()?;
    dispatcher.try_send(InnerMessage::Register {
        name: data.username.clone(),
        addr: addr,
    })?;
    let resp = resp.set_body(BoxBody::new(token));
    Ok(resp)
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().unwrap();
    let pool = Pool::new(ConnectionManager::<PgConnection>::new(std::env::var("DATABASE_URL").unwrap())).unwrap();
    let author = JWTAuthor::new("abcdegfh".chars().map(|c| c as u8).collect());
    let dispatcher = Dispatcher::new();
    let addr = dispatcher.start();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(addr.clone()))
            .app_data(Data::new(author.clone()))
            .app_data(Data::new(pool.clone()))
            .route("/login/{name}", get().to(login::<JWTAuthor>))
    })
    .bind("0.0.0.0:8000")
    .unwrap()
    .run()
    .await
}
