use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;
use serde::Serialize;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub errno: i32,
    pub errmsg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[allow(dead_code)]
impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { errno: 0, errmsg: String::new(), data: Some(data) }
    }
}

#[allow(dead_code)]
impl ApiResponse<Value> {
    pub fn ok_empty() -> Self {
        ApiResponse { errno: 0, errmsg: String::new(), data: None }
    }

    pub fn err(errno: i32, errmsg: impl Into<String>) -> Self {
        ApiResponse { errno, errmsg: errmsg.into(), data: None }
    }
}

pub struct Json(pub Value);

impl IntoResponse for Json {
    fn into_response(self) -> axum::response::Response {
        let body = axum::body::Body::from(serde_json::to_vec(&self.0).unwrap());
        axum::response::Response::builder()
            .header(header::CONTENT_TYPE, HeaderValue::from_static("application/json; charset=utf-8"))
            .body(body)
            .unwrap()
    }
}

#[allow(dead_code)]
pub fn ok<T: Serialize>(data: T) -> Json {
    Json(serde_json::to_value(ApiResponse::ok(data)).unwrap())
}

#[allow(dead_code)]
pub fn ok_empty() -> Json {
    Json(serde_json::to_value(ApiResponse::<Value>::ok_empty()).unwrap())
}

#[allow(dead_code)]
pub fn err(errno: i32, errmsg: impl Into<String>) -> Json {
    Json(serde_json::to_value(ApiResponse::<Value>::err(errno, errmsg)).unwrap())
}
