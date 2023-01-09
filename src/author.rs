use crate::error::Error;
use crate::Author;
use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha384};

#[derive(Debug, Clone)]
pub struct JWTAuthor {
    secret: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    pub uid: i32,
}

impl JWTAuthor {
    pub fn new(secret: Vec<u8>) -> Self {
        Self { secret }
    }
}

impl Author for JWTAuthor {
    fn hash_password(&self, pwd: String, salt: String) -> String {
        let mut hasher = Sha384::new();
        hasher.update(pwd);
        hasher.update(salt);
        format!("{:x}", hasher.finalize())
    }

    fn gen_token(&self, uid: i32) -> Result<String, Error> {
        let key: Hmac<Sha384> = Hmac::new_from_slice(&self.secret)?;
        let header = Header {
            algorithm: AlgorithmType::Hs384,
            ..Default::default()
        };
        let token = Token::new(header, Claim { uid }).sign_with_key(&key)?;
        Ok(token.as_str().to_owned())
    }

    fn verify(&self, token_str: String) -> Result<i32, crate::error::Error> {
        let key: Hmac<Sha384> = Hmac::new_from_slice(&self.secret)?;
        let token: Token<Header, Claim, _> = token_str.verify_with_key(&key)?;
        Ok(token.claims().uid.clone())
    }
}
