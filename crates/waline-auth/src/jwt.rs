use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// User ID
    pub sub: String,
    /// User type (administrator, guest, etc.)
    pub r#type: String,
    /// Issued at
    pub iat: i64,
    /// Expiration
    pub exp: i64,
}

impl TokenClaims {
    /// Create a new token claim for a user.
    pub fn new(user_id: &str, user_type: &str, expires_hours: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            r#type: user_type.to_string(),
            iat: now,
            exp: now + expires_hours * 3600,
        }
    }

    /// Create claims with 7-day expiration (default for Waline).
    pub fn new_default(user_id: &str, user_type: &str) -> Self {
        Self::new(user_id, user_type, 24 * 7) // 7 days
    }
}

/// Create a JWT token.
pub fn create_token(claims: &TokenClaims, secret: &str) -> Result<String, String> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("Failed to create token: {e}"))
}

/// Verify and decode a JWT token.
pub fn verify_token(token: &str, secret: &str) -> Result<TokenClaims, String> {
    let mut validation = Validation::default();
    validation.leeway = 60; // 60 seconds leeway
    decode::<TokenClaims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
        .map(|data| data.claims)
        .map_err(|e| format!("Invalid token: {e}"))
}

/// Create a short-lived token for email verification or password reset.
pub fn create_short_lived_token(user_id: &str, user_type: &str, secret: &str, expires_hours: i64) -> Result<String, String> {
    let claims = TokenClaims::new(user_id, user_type, expires_hours);
    create_token(&claims, secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_token() {
        let secret = "test_secret";
        let claims = TokenClaims::new_default("123", "administrator");
        let token = create_token(&claims, secret).unwrap();
        let verified = verify_token(&token, secret).unwrap();
        assert_eq!(verified.sub, "123");
        assert_eq!(verified.r#type, "administrator");
    }

    #[test]
    fn test_invalid_token() {
        let result = verify_token("invalid_token", "secret");
        assert!(result.is_err());
    }
}
