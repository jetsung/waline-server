use waline_common::models::Comment;
use lettre::{
    message::{header::ContentType, Mailbox},
    AsyncTransport, Message,
};
use waline_core::config::Config;

/// Send an email via SMTP
pub async fn send_smtp_email(
    config: &Config,
    to: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), String> {
    let sender_email = config.sender_email.as_deref().unwrap_or(
        config.smtp_user.as_deref().unwrap_or("waline@localhost"),
    );
    let sender_name = config.sender_name.as_deref().unwrap_or("Waline");

    let from_mailbox = Mailbox::new(
        Some(sender_name.to_string()),
        sender_email.parse().map_err(|e| format!("Invalid sender email: {e}"))?,
    );
    let to_mailbox: Mailbox = to.parse().map_err(|e| format!("Invalid recipient email: {e}"))?;

    let email = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body.to_string())
        .map_err(|e| format!("Failed to build email: {e}"))?;

    let mailer = build_transport(config)?;
    mailer.send(email).await
        .map_err(|e| format!("Failed to send email: {e}"))?;

    Ok(())
}

/// Build SMTP transport from config
fn build_transport(
    config: &Config,
) -> Result<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>, String> {
    // Simplified SMTP transport builder
    let host = if let Some(ref service) = config.smtp_service {
        match service.to_lowercase().as_str() {
            "gmail" => "smtp.gmail.com",
            "outlook" | "hotmail" => "smtp.office365.com",
            "qq" => "smtp.qq.com",
            "163" => "smtp.163.com",
            "126" => "smtp.126.com",
            "aliyun" => "smtp.aliyun.com",
            _ => return Err(format!("Unknown SMTP service: {service}")),
        }
    } else {
        config.smtp_host.as_deref().ok_or("SMTP_HOST not set")?
    };

    let port = config.smtp_port.unwrap_or(465);
    let user = config.smtp_user.as_deref().unwrap_or("");
    let pass = config.smtp_pass.as_deref().unwrap_or("");

    let creds = lettre::transport::smtp::authentication::Credentials::new(
        user.to_string(),
        pass.to_string(),
    );

    let transport = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(host)
        .map_err(|e| format!("Failed to create SMTP transport: {e}"))?
        .port(port)
        .credentials(creds)
        .build();

    Ok(transport)
}

/// Send reply notification email to parent commenter
pub async fn send_reply_email(
    config: &Config,
    parent_email: &str,
    comment: &Comment,
    site_name: &str,
    site_url: &str,
) -> Result<(), String> {
    let subject = config.mail_subject.as_deref().unwrap_or("New reply to your comment");
    let default_template = "<p>Hi there,</p><p>Your comment has a new reply:</p><blockquote>{content}</blockquote><p>By: {nick}</p>";
    let template = config.mail_template.as_deref().unwrap_or(default_template);

    let html = template
        .replace("{site_name}", site_name)
        .replace("{site_url}", site_url)
        .replace("{content}", &comment.comment)
        .replace("{nick}", comment.nick.as_deref().unwrap_or("Anonymous"));

    send_smtp_email(config, parent_email, subject, &html).await
}

/// Send author notification email
pub async fn send_author_email(
    config: &Config,
    comment: &Comment,
    site_name: &str,
    site_url: &str,
) -> Result<(), String> {
    let author_email = config.author_email.as_deref().unwrap_or(
        config.smtp_user.as_deref().unwrap_or(""),
    );
    if author_email.is_empty() {
        return Ok(());
    }

    let subject = config.mail_subject_admin.as_deref().unwrap_or("New comment on your site");
    let default_template = "<p>New comment:</p><blockquote>{content}</blockquote><p>By: {nick} ({mail})</p>";
    let template = config.mail_template_admin.as_deref().unwrap_or(default_template);

    let html = template
        .replace("{site_name}", site_name)
        .replace("{site_url}", site_url)
        .replace("{content}", &comment.comment)
        .replace("{nick}", comment.nick.as_deref().unwrap_or("Anonymous"))
        .replace("{mail}", comment.mail.as_deref().unwrap_or(""));

    send_smtp_email(config, author_email, subject, &html).await
}

/// Send registration verification email with 4-digit code
pub async fn send_verification_email(
    config: &Config,
    to: &str,
    code: &str,
    site_name: &str,
) -> Result<(), String> {
    let subject = format!("[{site_name}] Email Verification");
    let html = format!(
        "<p>Your verification code is: <strong>{code}</strong></p><p>This code expires in 1 hour.</p>"
    );
    send_smtp_email(config, to, &subject, &html).await
}

/// Send password reset email with JWT-based login link
pub async fn send_password_reset_email(
    config: &Config,
    to: &str,
    token: &str,
    site_name: &str,
    site_url: &str,
) -> Result<(), String> {
    let subject = format!("[{site_name}] Password Reset");
    let reset_url = format!("{site_url}/ui/profile?token={token}");
    let html = format!(
        "<p>Click the link below to reset your password:</p><p><a href=\"{reset_url}\">{reset_url}</a></p><p>This link expires in 1 hour.</p>"
    );
    send_smtp_email(config, to, &subject, &html).await
}
