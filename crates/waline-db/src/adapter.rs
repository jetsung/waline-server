use async_trait::async_trait;
use waline_common::error::Result;
use waline_common::models::*;

/// Helper to convert sqlx errors
macro_rules! db_err {
    ($result:expr) => {
        $result.map_err(|e| waline_common::error::WalineError::Database(e.to_string()))
    };
}

/// Unified database adapter trait for all database operations.
/// Implementations: PostgreSQL, MySQL, SQLite.
#[async_trait]
pub trait DatabaseAdapter: Send + Sync {
    // === Comment operations ===

    async fn select_comments(&self, query: &CommentQuery) -> Result<Vec<Comment>>;

    async fn count_comments(&self, urls: &[String]) -> Result<Vec<i64>>;

    async fn count_comments_by_status(&self, url: &str, status: &str) -> Result<i64>;

    async fn add_comment(&self, comment: &NewComment) -> Result<Comment>;

    async fn update_comment(&self, id: &str, update: &CommentUpdate) -> Result<()>;

    async fn delete_comment(&self, id: &str) -> Result<()>;

    /// Get total comment count
    async fn total_comments(&self, query: &CommentQuery) -> Result<i64>;

    // === Counter operations ===

    async fn select_counters(&self, urls: &[String]) -> Result<Vec<Counter>>;

    async fn upsert_counter(&self, url: &str, increment: i64) -> Result<Counter>;

    // === User operations ===

    async fn select_users(&self, query: &UserQuery) -> Result<Vec<User>>;

    async fn count_users(&self, query: &UserQuery) -> Result<i64>;

    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>>;

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;

    async fn get_user_by_social(&self, provider: &str, id: &str) -> Result<Option<User>>;

    async fn add_user(&self, user: &NewUser) -> Result<User>;

    async fn update_user(&self, id: &str, update: &UserUpdate) -> Result<()>;

    async fn delete_user(&self, id: &str) -> Result<()>;

    // === Data import/export ===

    async fn export_all(&self) -> Result<serde_json::Value>;

    async fn import_data(&self, data: &serde_json::Value) -> Result<()>;

    async fn update_records(&self, table: &str, data: &serde_json::Value) -> Result<()>;

    async fn clear_table(&self, table: &str) -> Result<()>;
}
