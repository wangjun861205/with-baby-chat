#![feature(async_fn_in_trait)]

mod author;
mod dao;
mod error;
mod message;
mod models;
mod websocket;

#[macro_use]
extern crate diesel;

use std::sync::RwLock;

use crate::author::JWTAuthor;
use crate::error::Error;
use crate::websocket::WS;
use actix::{self, Addr};
use actix_web::http::StatusCode;
use actix_web::web::{self, post, Data, Json};
use actix_web::{App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer};
use actix_web_actors::ws::WsResponseBuilder;
use dotenv;
use models::{Account, AccountInsert, Channel, ChannelInsert, Friend, FriendApplication, FriendInsert, JoinApplication, Member, User};
use serde::{Deserialize, Serialize};
use sqlx::{self, postgres::PgPoolOptions};
use std::collections::HashMap;

pub trait Author {
    async fn auth(&self, account: String, credential: String) -> Result<Option<String>, Error>;
    fn verify(&self, token: String) -> Result<String, Error>;
    async fn signup(&self, account: String, credential: String) -> Result<usize, Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

async fn login<A>(author: Data<A>, users: Data<RwLock<HashMap<String, Option<Addr<WS<A>>>>>>, Json(data): Json<LoginRequest>, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error>
where
    A: Author + Clone + Unpin + 'static,
{
    let res = WsResponseBuilder::new(WS::new(data.username.clone(), author.clone(), users.clone()), &req, stream).start()?;
    Ok(res)
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

async fn signup<A>(author: Data<A>, Json(data): Json<SignupRequest>) -> Result<HttpResponse, Error>
where
    A: Author,
{
    author.signup(data.username.clone(), data.password.clone()).await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK).finish())
}

pub trait Dao {
    async fn insert_account(&self, account: AccountInsert) -> Result<i32, Error>;
    async fn get_account(&self, phone: String) -> Result<Option<Account>, Error>;
    async fn insert_user(&self, user: User) -> Result<i32, Error>;
    async fn get_user(&self, id: i32) -> Result<Option<User>, Error>;
    async fn insert_channel(&self, channel: ChannelInsert) -> Result<i32, Error>;
    async fn query_channel(&self, q: String) -> Result<Vec<Channel>, Error>;
    async fn insert_friend_application(&self, app: FriendApplication) -> Result<i32, Error>;
    async fn insert_join_application(&self, app: JoinApplication) -> Result<i32, Error>;
    async fn insert_friend(&self, friend: FriendInsert) -> Result<i32, Error>;
    async fn exists_friend(&self, user_a: i32, user_b: i32) -> Result<bool, Error>;
    async fn delete_friend(&self, id: i32) -> Result<u64, Error>;
    async fn insert_member(&self, member: Member) -> Result<i32, Error>;
    async fn delete_member(&self, id: i32) -> Result<u64, Error>;
    async fn exists_member(&self, user_id: i32, channel_id: i32) -> Result<bool, Error>;
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().unwrap();
    let db = PgPoolOptions::new().max_connections(5).connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();
    let users: Vec<User> = sqlx::query_as("SELECT * FROM users").fetch_all(&db).await.unwrap();
    let users = Data::new(RwLock::new(users.into_iter().map(|u| (u.username, None)).collect::<HashMap<String, Option<WS<JWTAuthor>>>>()));
    let author = JWTAuthor::new(db, "abcdegfh".chars().map(|c| c as u8).collect());
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(author.clone()))
            .app_data(users.clone())
            .route("/login", post().to(login::<JWTAuthor>))
            .route("/signup", post().to(signup::<JWTAuthor>))
    })
    .bind("0.0.0.0:8000")
    .unwrap()
    .run()
    .await
}
