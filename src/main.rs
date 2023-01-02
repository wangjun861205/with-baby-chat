mod author;
mod dispatcher;
mod error;
mod message;
mod models;
mod schema;
mod websocket;

#[macro_use]
extern crate diesel;

use std::sync::Mutex;

use crate::author::JWTAuthor;
use crate::dispatcher::Dispatcher;
use crate::error::Error;
use crate::websocket::WS;
use actix::{self, Actor, Addr};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::web::{self, post, Data, Json};
use actix_web::{App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer};
use actix_web_actors::ws::WsResponseBuilder;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use dotenv;
use message::InnerMessage;
use models::User;
use r2d2::PooledConnection;
use serde::{Deserialize, Serialize};
use sqlx::{self, postgres::PgPoolOptions};
use std::collections::HashMap;

pub trait Author {
    fn auth(&self, db: PooledConnection<ConnectionManager<PgConnection>>, account: String, credential: String) -> Result<String, Error>;
    fn verify(&self, token: String) -> Result<String, Error>;
    fn signup(&self, db: PooledConnection<ConnectionManager<PgConnection>>, account: String, credential: String) -> Result<usize, Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

async fn login<A>(
    dispatcher: Data<Addr<Dispatcher<A>>>,
    author: Data<A>,
    db: Data<Pool<ConnectionManager<PgConnection>>>,
    users: Data<Mutex<HashMap<String, Option<WS<A>>>>>,
    Json(data): Json<LoginRequest>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error>
where
    A: Author + Unpin + 'static,
{
    let token = author.auth(db.get()?, data.username.clone(), data.password.clone())?;
    let (addr, resp) = WsResponseBuilder::new(
        WS {
            name: data.username.clone(),
            dispatcher: dispatcher.clone(),
            author: author.clone(),
            users: users.clone(),
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

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

async fn signup<A>(author: Data<A>, db: Data<Pool<ConnectionManager<PgConnection>>>, Json(data): Json<SignupRequest>) -> Result<HttpResponse, Error>
where
    A: Author,
{
    author.signup(db.get()?, data.username.clone(), data.password.clone())?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).finish())
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().unwrap();
    let db = PgPoolOptions::new().max_connections(5).connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();
    let users: Vec<User> = sqlx::query_as("SELECT * FROM users").fetch_all(&db).await.unwrap();
    let users = Data::new(Mutex::new(users.into_iter().map(|u| (u.username, None)).collect::<HashMap<String, Option<WS<JWTAuthor>>>>()));
    let pool = Pool::new(ConnectionManager::<PgConnection>::new(std::env::var("DATABASE_URL").unwrap())).unwrap();
    let author = JWTAuthor::new("abcdegfh".chars().map(|c| c as u8).collect());
    let dispatcher = Dispatcher::<JWTAuthor>::new();
    let addr = dispatcher.start();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(addr.clone()))
            .app_data(Data::new(author.clone()))
            .app_data(Data::new(pool.clone()))
            .app_data(users.clone())
            .route("/login", post().to(login::<JWTAuthor>))
            .route("/signup", post().to(signup::<JWTAuthor>))
    })
    .bind("0.0.0.0:8000")
    .unwrap()
    .run()
    .await
}
