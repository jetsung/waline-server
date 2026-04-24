use serde::Deserialize;

/// Verify a reCAPTCHA v3 token.
pub async fn verify_recaptcha(secret: &str, token: &str) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://www.google.com/recaptcha/api/siteverify")
        .form(&[
            ("secret", secret),
            ("response", token),
        ])
        .send()
        .await
        .map_err(|e| format!("reCAPTCHA request failed: {e}"))?;

    #[derive(Deserialize)]
    struct RecaptchaResponse {
        success: bool,
    }

    let result: RecaptchaResponse = resp
        .json()
        .await
        .map_err(|e| format!("reCAPTCHA response parse failed: {e}"))?;

    Ok(result.success)
}

/// Verify a Cloudflare Turnstile token.
pub async fn verify_turnstile(secret: &str, token: &str) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&[
            ("secret", secret),
            ("response", token),
        ])
        .send()
        .await
        .map_err(|e| format!("Turnstile request failed: {e}"))?;

    #[derive(Deserialize)]
    struct TurnstileResponse {
        success: bool,
    }

    let result: TurnstileResponse = resp
        .json()
        .await
        .map_err(|e| format!("Turnstile response parse failed: {e}"))?;

    Ok(result.success)
}
