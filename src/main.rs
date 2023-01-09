#![feature(async_fn_in_trait)]

mod author;
mod dao;
mod error;
mod message;
mod models;
mod websocket;

use std::sync::RwLock;

use crate::author::JWTAuthor;
use crate::dao::PostgresDao;
use crate::error::Error;
use crate::websocket::WS;
use actix::{self, Addr};
use actix_web::web::{self, get, Data};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws::{self};
use dotenv;
use models::{Account, AccountInsert, Channel, ChannelInsert, FriendApplicationInsert, FriendInsert, JoinApplicationInsert, MemberInsert, User, UserInsert};
use sqlx::{self, postgres::PgPoolOptions};
use std::collections::HashMap;

pub trait Author {
    fn hash_password(&self, pwd: String, salt: String) -> String;
    fn gen_token(&self, uid: i32) -> Result<String, Error>;
    fn verify(&self, token: String) -> Result<i32, Error>;
}

async fn start<A, D>(author: Data<A>, users: Data<RwLock<HashMap<String, Option<Addr<WS<A, D>>>>>>, dao: Data<D>, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error>
where
    A: Author + Clone + Unpin + 'static,
    D: Dao + Clone + Unpin + 'static,
{
    let actor = WS::new("".into(), author.clone(), users.clone(), dao.clone());
    let res = ws::start(actor, &req, stream)?;
    Ok(res)
}

pub trait Dao {
    async fn insert_account(&self, account: AccountInsert) -> Result<i32, Error>;
    async fn get_account(&self, phone: String) -> Result<Option<Account>, Error>;
    async fn insert_user(&self, user: UserInsert) -> Result<i32, Error>;
    async fn get_user(&self, id: i32) -> Result<Option<User>, Error>;
    async fn get_user_by_account_id(&self, account: i32) -> Result<Option<User>, Error>;
    async fn insert_channel(&self, channel: ChannelInsert) -> Result<i32, Error>;
    async fn query_channel(&self, q: String) -> Result<Vec<Channel>, Error>;
    async fn insert_friend_application(&self, app: FriendApplicationInsert) -> Result<i32, Error>;
    async fn insert_join_application(&self, app: JoinApplicationInsert) -> Result<i32, Error>;
    async fn insert_friend(&self, friend: FriendInsert) -> Result<i32, Error>;
    async fn exists_friend(&self, user_a: i32, user_b: i32) -> Result<bool, Error>;
    async fn delete_friend(&self, id: i32) -> Result<u64, Error>;
    async fn insert_member(&self, member: MemberInsert) -> Result<i32, Error>;
    async fn delete_member(&self, id: i32) -> Result<u64, Error>;
    async fn exists_member(&self, user_id: i32, channel_id: i32) -> Result<bool, Error>;
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    dotenv::dotenv().unwrap();
    let db = PgPoolOptions::new().max_connections(5).connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();
    let users: Vec<User> = sqlx::query_as("SELECT * FROM users").fetch_all(&db).await.unwrap();
    let dao = Data::new(PostgresDao::new(db));
    let users = Data::new(RwLock::new(
        users.into_iter().map(|u| (u.name, None)).collect::<HashMap<String, Option<Addr<WS<JWTAuthor, PostgresDao>>>>>(),
    ));
    let author = Data::new(JWTAuthor::new("abcdegfh".chars().map(|c| c as u8).collect()));
    HttpServer::new(move || {
        App::new()
            .app_data(author.clone())
            .app_data(users.clone())
            .app_data(dao.clone())
            .route("/", get().to(start::<JWTAuthor, PostgresDao>))
    })
    .bind("0.0.0.0:8000")
    .unwrap()
    .run()
    .await
}
