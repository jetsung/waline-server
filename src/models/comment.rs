use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "wl_Comment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_name = "user_id")]
    pub user_id: Option<i64>,
    pub comment: Option<String>,
    #[sea_orm(column_name = "insertedAt")]
    pub inserted_at: Option<String>,
    pub ip: Option<String>,
    pub link: Option<String>,
    pub mail: Option<String>,
    pub nick: Option<String>,
    pub pid: Option<i64>,
    pub rid: Option<i64>,
    pub sticky: Option<i64>,
    pub status: String,
    #[sea_orm(column_name = "like")]
    pub like_count: Option<i64>,
    pub ua: Option<String>,
    pub url: Option<String>,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: Option<String>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Return millisecond timestamp for insertedAt (or createdAt fallback).
    pub fn time_ms(&self) -> i64 {
        parse_ts(self.inserted_at.as_deref())
            .or_else(|| parse_ts(self.created_at.as_deref()))
            .unwrap_or(0)
    }
}

fn parse_ts(s: Option<&str>) -> Option<i64> {
    let s = s?;
    // Try RFC3339 first, then SQLite local datetime format
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp_millis());
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(dt.and_utc().timestamp_millis());
    }
    None
}
