use crate::error::Error;
use crate::schema::users;
use crate::Author;
use actix_web::web::Data;
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    ExpressionMethods, PgConnection, QueryDsl, Queryable, RunQueryDsl,
};
use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha384};
use sqlx::{query, query_as, FromRow, Pool, Postgres};
use std::future::Future;
use std::pin::Pin;

#[derive(FromRow)]
pub struct User {
    id: i32,
    username: String,
    password: String,
    salt: String,
}

#[derive(Debug, Clone)]
pub struct JWTAuthor {
    secret: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    pub account: String,
}

impl JWTAuthor {
    pub fn new(secret: Vec<u8>) -> Self {
        Self { secret }
    }
}

impl Author for JWTAuthor {
    fn auth(&self, mut db: Data<Pool<Postgres>>, account: String, credential: String) -> Pin<Box<dyn Future<Output = Result<String, Error>>>> {
        let secret = self.secret.clone();
        Box::pin(async move {
            let user: User = query_as("SELECT * FROM users WHERE username = $1").bind(account.clone()).fetch_one(db.as_ref()).await?;
            let mut hasher = Sha384::new();
            hasher.update(format!("{}{}", credential, user.salt));
            let hashed_pwd = format!("{:x}", hasher.finalize());
            if user.password != hashed_pwd {
                return Err(Error("invalid account".into()));
            }
            let key: Hmac<Sha384> = Hmac::new_from_slice(&secret)?;
            let token = Token::new(
                Header {
                    algorithm: AlgorithmType::Hs384,
                    ..Default::default()
                },
                Claim { account },
            )
            .sign_with_key(&key)?;
            Ok(token.as_str().to_owned())
        })
    }

    fn verify(&self, token_str: String) -> Result<String, crate::error::Error> {
        let key: Hmac<Sha384> = Hmac::new_from_slice(&self.secret)?;
        let token: Token<Header, Claim, _> = token_str.verify_with_key(&key)?;
        Ok(token.claims().account.clone())
    }

    fn signup(&self, mut db: Data<Pool<Postgres>>, account: String, credential: String) -> Pin<Box<dyn Future<Output = Result<usize, crate::error::Error>>>> {
        Box::pin(async move {
            let rng = thread_rng();
            let salt: String = rng.sample_iter(Alphanumeric).take(32).map(|c| c as char).collect();
            let mut hasher = Sha384::new();
            hasher.update(format!("{}{}", credential, salt));
            let hashed_pwd = format!("{:x}", hasher.finalize());
            let res = query!("INSERT INTO users (username, password, salt) VALUES ($1, $2, $3) RETURNING id", account, hashed_pwd, salt)
                .fetch_one(db.as_ref())
                .await?
                .id;
            Ok(res as usize)
        })
    }
}
