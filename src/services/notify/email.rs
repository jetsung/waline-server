use lettre::{
    Message, SmtpTransport, Transport,
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use serde_json::Value;

use crate::config::Config;
use crate::locales::translate;

struct SmtpPreset {
    host: &'static str,
    port: u16,
    secure: bool,
}

fn smtp_preset(service: &str) -> Option<SmtpPreset> {
    match service.to_lowercase().as_str() {
        "gmail" => Some(SmtpPreset { host: "smtp.gmail.com", port: 587, secure: false }),
        "outlook" => Some(SmtpPreset { host: "smtp.office365.com", port: 587, secure: false }),
        "qq" => Some(SmtpPreset { host: "smtp.qq.com", port: 465, secure: true }),
        "163" => Some(SmtpPreset { host: "smtp.163.com", port: 465, secure: true }),
        "126" => Some(SmtpPreset { host: "smtp.126.com", port: 465, secure: true }),
        "aliyun" => Some(SmtpPreset { host: "smtp.aliyun.com", port: 465, secure: true }),
        _ => None,
    }
}

/// Render a template string, replacing {{key}} placeholders from data.
fn render_template(template: &str, data: &Value) -> String {
    let mut result = template.to_string();
    if let Some(obj) = data.as_object() {
        for (k, v) in obj {
            let placeholder = format!("{{{{{k}}}}}");
            let val = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => v.to_string(),
            };
            result = result.replace(&placeholder, &val);
        }
    }
    result
}

/// Flatten template data for simple key substitution.
fn flatten_data(data: &Value) -> Value {
    let mut flat = serde_json::Map::new();
    if let Some(site) = data.get("site").and_then(|v| v.as_object()) {
        for (k, v) in site {
            flat.insert(format!("site.{k}"), v.clone());
        }
    }
    if let Some(s) = data.get("self").and_then(|v| v.as_object()) {
        for (k, v) in s {
            flat.insert(format!("comment.{k}"), v.clone());
        }
    }
    Value::Object(flat)
}

pub async fn send_mail(config: &Config, to: &str, subject_key: &str, template_key: &str, data: Value) {
    let (smtp_user, smtp_pass) = match (&config.smtp_user, &config.smtp_pass) {
        (Some(u), Some(p)) => (u.clone(), p.clone()),
        _ => return,
    };

    let (host, port, secure) = if let Some(svc) = &config.smtp_service {
        if let Some(preset) = smtp_preset(svc) {
            (preset.host.to_string(), preset.port, preset.secure)
        } else {
            return;
        }
    } else if let Some(h) = &config.smtp_host {
        let port = config.smtp_port.unwrap_or(465);
        let secure = config.smtp_secure.unwrap_or(true);
        (h.clone(), port, secure)
    } else {
        return;
    };

    let flat = flatten_data(&data);
    let subject = render_template(&translate("en", subject_key), &flat);
    let body = render_template(&translate("en", template_key), &flat);

    let sender_email = config.sender_email.as_deref().unwrap_or(&smtp_user);
    let sender_name = config.sender_name.as_deref().unwrap_or("Waline");
    let from = format!("{sender_name} <{sender_email}>");

    let msg = match Message::builder()
        .from(from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body)
    {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Build email failed: {e}");
            return;
        }
    };

    let mailer = if secure {
        SmtpTransport::relay(&host)
            .unwrap()
            .credentials(Credentials::new(smtp_user, smtp_pass))
            .port(port)
            .build()
    } else {
        SmtpTransport::starttls_relay(&host)
            .unwrap()
            .credentials(Credentials::new(smtp_user, smtp_pass))
            .port(port)
            .build()
    };

    match mailer.send(&msg) {
        Ok(_) => tracing::info!("Email sent to {to}"),
        Err(e) => tracing::error!("Email send failed: {e}"),
    }
}
