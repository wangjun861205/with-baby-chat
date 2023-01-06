use crate::error::Error;
use crate::models::{Account, AccountInsert, ChannelInsert, FriendApplicationInsert, FriendInsert, JoinApplicationInsert, MemberInsert, User, UserInsert};
use crate::Dao;
use actix_web::web::Data;
use sqlx::{query, query_as, Pool, Postgres};

#[derive(Debug, Clone)]
pub struct PostgresDao {
    db: Pool<Postgres>,
}

impl PostgresDao {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }
}

impl Dao for PostgresDao {
    async fn delete_friend(&self, id: i32) -> Result<u64, Error> {
        let res = query!("DELETE FROM friends WHERE id = $1", id).execute(&self.db).await?;
        Ok(res.rows_affected())
    }
    async fn delete_member(&self, id: i32) -> Result<u64, Error> {
        let res = query!("DELETE FROM members WHERE id = $1", id).execute(&self.db).await?;
        Ok(res.rows_affected())
    }

    async fn exists_friend(&self, user_a: i32, user_b: i32) -> Result<bool, Error> {
        let res = query!("SELECT EXISTS(SELECT id FROM friends WHERE user_a = $1 AND user_b = $2 OR user_a = $2 AND user_b = $1)", user_a, user_b)
            .fetch_one(&self.db)
            .await?;
        Ok(res.exists.unwrap())
    }

    async fn exists_member(&self, user_id: i32, channel_id: i32) -> Result<bool, Error> {
        let res = query!(r#"SELECT EXISTS(SELECT id FROM members WHERE "user" = $1 AND channel = $2)"#, user_id, channel_id)
            .fetch_one(&self.db)
            .await?;
        Ok(res.exists.unwrap())
    }

    async fn get_account(&self, phone: String) -> Result<Option<Account>, Error> {
        let res: Option<Account> = query_as("SELECT * FROM accounts WHERE phone = $1").bind(phone).fetch_optional(&self.db).await?;
        Ok(res)
    }

    async fn get_user(&self, id: i32) -> Result<Option<User>, Error> {
        let res: Option<User> = query_as("SELECT * FROM users WHERE id = $1").bind(id).fetch_optional(&self.db).await?;
        Ok(res)
    }

    async fn insert_account(&self, acct: AccountInsert) -> Result<i32, Error> {
        let res = query!("INSERT INTO accounts (phone, password, salt) VALUES($1, $2, $3) RETURNING id", acct.phone, acct.password, acct.salt)
            .fetch_one(&self.db)
            .await?;
        Ok(res.id)
    }

    async fn insert_channel(&self, channel: ChannelInsert) -> Result<i32, Error> {
        let res = query!(
            "INSERT INTO channels (name, description, administrator) VALUES($1, $2, $3) RETURNING id",
            channel.name,
            channel.description,
            channel.administrator
        )
        .fetch_one(&self.db)
        .await?;
        Ok(res.id)
    }

    async fn insert_friend(&self, friend: FriendInsert) -> Result<i32, Error> {
        let res = query!("INSERT INTO friends (user_a, user_b) VALUES($1, $2) RETURNING id", friend.user_a, friend.user_b,)
            .fetch_one(&self.db)
            .await?;
        Ok(res.id)
    }

    async fn insert_friend_application(&self, app: FriendApplicationInsert) -> Result<i32, Error> {
        let res = query!(r#"INSERT INTO friend_applications ("from", "to") VALUES ($1, $2) RETURNING id"#, app.from, app.to)
            .fetch_one(&self.db)
            .await?;
        Ok(res.id)
    }

    async fn insert_join_application(&self, app: JoinApplicationInsert) -> Result<i32, Error> {
        let res = query!(r#"INSERT INTO join_applications ("from", "to") VALUES ($1, $2) RETURNING id"#, app.from, app.to)
            .fetch_one(&self.db)
            .await?;
        Ok(res.id)
    }

    async fn insert_member(&self, member: MemberInsert) -> Result<i32, Error> {
        let res = query!(r#"INSERT INTO members (channel, "user") VALUES ($1, $2) RETURNING id"#, member.channel, member.user)
            .fetch_one(&self.db)
            .await?;
        Ok(res.id)
    }

    async fn insert_user(&self, user: UserInsert) -> Result<i32, Error> {
        let res = query!(r#"INSERT INTO users (name, account) VALUES ($1, $2) RETURNING id"#, user.name, user.account)
            .fetch_one(&self.db)
            .await?;
        Ok(res.id)
    }

    async fn query_channel(&self, q: String) -> Result<Vec<crate::models::Channel>, Error> {
        let res = query_as(r#"SELECT * FROM channels WHERE name LIKE '%$1%'"#).bind(q).fetch_all(&self.db).await?;
        Ok(res)
    }

    async fn get_user_by_account_id(&self, account: i32) -> Result<Option<User>, Error> {
        let res = query_as(r#"SELECT * FROM users WHERE account = $1"#).bind(account).fetch_optional(&self.db).await?;
        Ok(res)
    }
}
