use sqlx::FromRow;

#[derive(FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
}
