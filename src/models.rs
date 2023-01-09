use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub account: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInsert {
    pub name: String,
    pub account: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: i32,
    pub phone: String,
    pub password: String,
    pub salt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInsert {
    pub phone: String,
    pub password: String,
    pub salt: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Channel {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub administrator: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelInsert {
    pub name: String,
    pub description: String,
    pub administrator: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendApplication {
    pub id: i32,
    pub from: i32,
    pub to: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendApplicationInsert {
    pub from: i32,
    pub to: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinApplication {
    pub id: i32,
    pub from: i32,
    pub to: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinApplicationInsert {
    pub from: i32,
    pub to: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Friend {
    pub id: i32,
    pub user_a: i32,
    pub user_b: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FriendInsert {
    pub user_a: i32,
    pub user_b: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Member {
    pub id: i32,
    pub channel: i32,
    pub user: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberInsert {
    pub channel: i32,
    pub user: i32,
}
