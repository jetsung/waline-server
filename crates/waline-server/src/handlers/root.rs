use axum::response::Html;

/// GET / - Demo page
pub async fn root() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html><html><head><title>Waline</title></head><body><h1>Waline Rust Server</h1><p>Comment system powered by Rust.</p></body></html>"#)
}
