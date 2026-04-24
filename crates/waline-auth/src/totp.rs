use totp_rs::{Algorithm, TOTP, Secret};
use serde::{Deserialize, Serialize};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSetup {
    pub secret: String,
    pub qr_url: String,
}

/// Generate a new TOTP secret for 2FA setup.
pub fn generate_totp_secret(account_name: &str, issuer: &str) -> Result<TotpSetup, String> {
    // Generate a random 20-byte secret
    let secret_bytes: Vec<u8> = (0..20).map(|_| rand::random::<u8>()).collect();
    let secret_str = STANDARD.encode(&secret_bytes);

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes.clone(),
        Some(issuer.to_string()),
        account_name.to_string(),
    )
    .map_err(|e| format!("Failed to create TOTP: {e}"))?;

    let qr_url = totp.get_url();

    Ok(TotpSetup {
        secret: secret_str,
        qr_url,
    })
}

/// Verify a TOTP code against a secret.
pub fn verify_totp(secret: &str, code: &str) -> bool {
    let secret_bytes = match STANDARD.decode(secret) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let totp = match TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Waline".to_string()),
        "user".to_string(),
    ) {
        Ok(t) => t,
        Err(_) => return false,
    };

    totp.check_current(code).unwrap_or(false)
}
