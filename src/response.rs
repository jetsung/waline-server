use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub errno: i32,
    pub errmsg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { errno: 0, errmsg: String::new(), data: Some(data) }
    }
}

impl ApiResponse<Value> {
    pub fn ok_empty() -> Self {
        ApiResponse { errno: 0, errmsg: String::new(), data: None }
    }

    pub fn err(errno: i32, errmsg: impl Into<String>) -> Self {
        ApiResponse { errno, errmsg: errmsg.into(), data: None }
    }
}

/// Shorthand for success response with data
pub fn ok<T: Serialize>(data: T) -> axum::Json<ApiResponse<T>> {
    axum::Json(ApiResponse::ok(data))
}

/// Shorthand for success response without data
pub fn ok_empty() -> axum::Json<ApiResponse<Value>> {
    axum::Json(ApiResponse::<Value>::ok_empty())
}

/// Shorthand for error response
pub fn err(errno: i32, errmsg: impl Into<String>) -> axum::Json<ApiResponse<Value>> {
    axum::Json(ApiResponse::<Value>::err(errno, errmsg))
}
