use crate::error::Error;
use crate::models::{Account, AccountInsert, ChannelInsert, FriendInsert, User};
use crate::Dao;
use sqlx::{query, query_as, Pool, Postgres};

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
}
