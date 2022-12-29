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

#[derive(Queryable)]
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
    fn auth(&self, mut db: PooledConnection<ConnectionManager<PgConnection>>, account: String, credential: String) -> Result<String, Error> {
        let user: User = users::table.filter(users::username.eq(account.clone())).get_result(&mut db)?;
        let mut hasher = Sha384::new();
        hasher.update(format!("{}{}", credential, user.salt));
        let hashed_pwd = format!("{:x}", hasher.finalize());
        if user.password != hashed_pwd {
            return Err(Error("invalid account".into()));
        }
        let key: Hmac<Sha384> = Hmac::new_from_slice(&self.secret)?;
        let token = Token::new(
            Header {
                algorithm: AlgorithmType::Hs384,
                ..Default::default()
            },
            Claim { account },
        )
        .sign_with_key(&key)?;
        Ok(token.as_str().to_owned())
    }

    fn verify(&self, token_str: String) -> Result<String, crate::error::Error> {
        let key: Hmac<Sha384> = Hmac::new_from_slice(&self.secret)?;
        let token: Token<Header, Claim, _> = token_str.verify_with_key(&key)?;
        Ok(token.claims().account.clone())
    }

    fn signup(&self, mut db: PooledConnection<ConnectionManager<PgConnection>>, account: String, credential: String) -> Result<usize, crate::error::Error> {
        let rng = thread_rng();
        let salt: String = rng.sample_iter(Alphanumeric).take(32).map(|c| c as char).collect();
        let res = diesel::insert_into(users::table)
            .values((users::username.eq(account), users::password.eq(credential), users::salt.eq(salt)))
            .execute(&mut db)?;
        Ok(res)
    }
}
