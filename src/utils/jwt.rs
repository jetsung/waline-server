use crate::error::AppError;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

/// Node版 JWT payload: sub = objectId (i64)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // objectId as string (matches jsonwebtoken.sign(objectId, key))
    pub exp: usize,
}

pub fn sign(object_id: i64, secret: &str, expires_in_secs: u64) -> Result<String, AppError> {
    let exp = (chrono::Utc::now().timestamp() as u64 + expires_in_secs) as usize;
    let claims = Claims { sub: object_id.to_string(), exp };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| AppError::Unauthorized)
}

pub fn object_id_from_token(token: &str, secret: &str) -> Result<i64, AppError> {
    let claims = verify(token, secret)?;
    claims.sub.parse::<i64>().map_err(|_| AppError::Unauthorized)
}
