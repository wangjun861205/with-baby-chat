use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub phone: String,
    pub password: String,
    pub salt: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Channel {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub administrator: i32,
}

#[derive(Debug, Serialize, Deseraialize)]
pub struct FriendApplication {
    pub id: i32,
    pub from: i32,
    pub to: i32,
}

#[derive(Debug, Serialize, Deseraialize)]
pub struct JoinApplication {
    pub id: i32,
    pub from: i32,
    pub to: i32,
}
