use async_trait::async_trait;
use sqlx::SqlitePool;
use waline_common::error::{Result, WalineError};
use waline_common::models::*;
use crate::adapter::DatabaseAdapter;
use crate::schema::create_tables_sqlite;

pub struct SqliteAdapter {
    pool: SqlitePool,
    prefix: String,
}

impl SqliteAdapter {
    pub async fn new(path: &str, prefix: &str) -> Result<Self> {
        let database_url = format!("sqlite:{path}?mode=rwc");
        let pool = SqlitePool::connect(&database_url).await.map_err(|e| {
            WalineError::Config(format!("Failed to connect to SQLite: {e}"))
        })?;
        create_tables_sqlite(&pool, prefix).await?;
        Ok(Self {
            pool,
            prefix: prefix.to_string(),
        })
    }

    fn comment_table(&self) -> String {
        format!("{}Comment", self.prefix)
    }
    fn counter_table(&self) -> String {
        format!("{}Counter", self.prefix)
    }
    fn users_table(&self) -> String {
        format!("{}Users", self.prefix)
    }

    fn row_to_comment(row: &sqlx::sqlite::SqliteRow) -> Comment {
        use sqlx::Row;
        Comment {
            object_id: row.get::<i64, _>("id").to_string(),
            user_id: row.try_get::<String, _>("user_id").ok(),
            comment: row.try_get::<String, _>("comment").unwrap_or_default(),
            inserted_at: row.try_get("insertedAt").unwrap_or(chrono::Utc::now()),
            ip: row.try_get::<String, _>("ip").ok(),
            link: row.try_get::<String, _>("link").ok(),
            mail: row.try_get::<String, _>("mail").ok(),
            nick: row.try_get::<String, _>("nick").ok(),
            pid: row.try_get::<String, _>("pid").ok(),
            rid: row.try_get::<String, _>("rid").ok(),
            status: row.try_get::<String, _>("status").unwrap_or_else(|_| "approved".to_string()),
            ua: row.try_get::<String, _>("ua").ok(),
            url: row.try_get::<String, _>("url").ok(),
            sticky: row.try_get::<i32, _>("sticky").ok().map(|v| v != 0),
            like: row.try_get::<i32, _>("like").unwrap_or(0),
            created_at: row.try_get("createdAt").unwrap_or(chrono::Utc::now()),
            updated_at: row.try_get("updatedAt").unwrap_or(chrono::Utc::now()),
            avatar: None, level: None, region: None, browser: None, os: None,
            children: Vec::new(),
        }
    }

    fn row_to_counter(row: &sqlx::sqlite::SqliteRow) -> Counter {
        use sqlx::Row;
        Counter {
            object_id: row.get::<i64, _>("id").to_string(),
            url: row.try_get::<String, _>("url").unwrap_or_default(),
            time: row.try_get::<i64, _>("time").unwrap_or(0),
            created_at: row.try_get("createdAt").unwrap_or(chrono::Utc::now()),
            updated_at: row.try_get("updatedAt").unwrap_or(chrono::Utc::now()),
        }
    }

    fn row_to_user(row: &sqlx::sqlite::SqliteRow) -> User {
        use sqlx::Row;
        User {
            object_id: row.get::<i64, _>("id").to_string(),
            display_name: row.try_get::<String, _>("display_name").ok(),
            email: row.try_get::<String, _>("email").ok(),
            password: row.try_get::<String, _>("password").ok(),
            user_type: row.try_get::<String, _>("type").unwrap_or_else(|_| "guest".to_string()),
            url: row.try_get::<String, _>("url").ok(),
            avatar: row.try_get::<String, _>("avatar").ok(),
            label: row.try_get::<String, _>("label").ok(),
            github: row.try_get::<String, _>("github").ok(),
            twitter: row.try_get::<String, _>("twitter").ok(),
            facebook: row.try_get::<String, _>("facebook").ok(),
            google: row.try_get::<String, _>("google").ok(),
            weibo: row.try_get::<String, _>("weibo").ok(),
            qq: row.try_get::<String, _>("qq").ok(),
            oidc: row.try_get::<String, _>("oidc").ok(),
            two_fa: row.try_get::<String, _>("2fa").ok(),
            created_at: row.try_get("createdAt").unwrap_or(chrono::Utc::now()),
            updated_at: row.try_get("updatedAt").unwrap_or(chrono::Utc::now()),
            level: None,
        }
    }
}

#[async_trait]
impl DatabaseAdapter for SqliteAdapter {
    async fn select_comments(&self, query: &CommentQuery) -> Result<Vec<Comment>> {
        let table = self.comment_table();
        let url = query.url.as_deref().or(query.path.as_deref()).unwrap_or("");
        let status = query.status.as_deref().unwrap_or("approved");

        match query.query_type.as_deref() {
            Some("recent") => {
                let limit = query.page_size.unwrap_or(50).min(50) as i64;
                let rows = sqlx::query(&format!(
                    r#"SELECT * FROM "{table}" WHERE "status" = ? ORDER BY "insertedAt" DESC LIMIT ?"#
                ))
                .bind(status).bind(limit)
                .fetch_all(&self.pool).await?;
                Ok(rows.iter().map(Self::row_to_comment).collect())
            }
            Some("list") => {
                let page = query.page.unwrap_or(1) as i64;
                let page_size = query.page_size.unwrap_or(10) as i64;
                let offset = (page - 1) * page_size;
                let rows = sqlx::query(&format!(
                    r#"SELECT * FROM "{table}" ORDER BY "insertedAt" DESC LIMIT ? OFFSET ?"#
                ))
                .bind(page_size).bind(offset)
                .fetch_all(&self.pool).await?;
                Ok(rows.iter().map(Self::row_to_comment).collect())
            }
            _ => {
                let sortby = query.sortby.as_deref().unwrap_or("insertedAt_desc");
                let (order_col, order_dir) = match sortby {
                    "insertedAt_asc" => ("insertedAt", "ASC"),
                    "like_desc" => ("like", "DESC"),
                    _ => ("insertedAt", "DESC"),
                };
                let rows = sqlx::query(&format!(
                    r#"SELECT * FROM "{table}" WHERE "url" = ? AND "status" = ? ORDER BY "{order_col}" {order_dir}"#
                ))
                .bind(url).bind(status)
                .fetch_all(&self.pool).await?;
                Ok(rows.iter().map(Self::row_to_comment).collect())
            }
        }
    }

    async fn count_comments(&self, urls: &[String]) -> Result<Vec<i64>> {
        let table = self.comment_table();
        let mut counts = Vec::with_capacity(urls.len());
        for url in urls {
            let count: (i64,) = sqlx::query_as(&format!(
                r#"SELECT COUNT(*) FROM "{table}" WHERE "url" = ? AND "status" = 'approved'"#
            )).bind(url).fetch_one(&self.pool).await?;
            counts.push(count.0);
        }
        Ok(counts)
    }

    async fn count_comments_by_status(&self, _url: &str, status: &str) -> Result<i64> {
        let table = self.comment_table();
        let count: (i64,) = sqlx::query_as(&format!(
            r#"SELECT COUNT(*) FROM "{table}" WHERE "status" = ?"#
        )).bind(status).fetch_one(&self.pool).await?;
        Ok(count.0)
    }

    async fn add_comment(&self, comment: &NewComment) -> Result<Comment> {
        let table = self.comment_table();
        sqlx::query(&format!(
            r#"INSERT INTO "{table}" ("user_id","comment","ip","link","mail","nick","pid","rid","status","ua","url","sticky") VALUES (?,?,?,?,?,?,?,?,?,?,?,?)"#
        ))
        .bind(&comment.user_id).bind(&comment.comment).bind(&comment.ip)
        .bind(&comment.link).bind(&comment.mail).bind(&comment.nick)
        .bind(&comment.pid).bind(&comment.rid).bind("approved")
        .bind(&comment.ua).bind(&comment.url).bind(comment.sticky)
        .execute(&self.pool).await?;

        let row = sqlx::query(&format!(r#"SELECT * FROM "{table}" WHERE "id" = last_insert_rowid()"#))
            .fetch_one(&self.pool).await?;
        Ok(Self::row_to_comment(&row))
    }

    async fn update_comment(&self, id: &str, update: &CommentUpdate) -> Result<()> {
        let table = self.comment_table();
        let id: i64 = id.parse().map_err(|_| WalineError::BadRequest("Invalid id".into()))?;
        if let Some(ref s) = update.status {
            sqlx::query(&format!(r#"UPDATE "{table}" SET "status"=?,"updatedAt"=CURRENT_TIMESTAMP WHERE "id"=?"#))
                .bind(s).bind(id).execute(&self.pool).await?;
        }
        if let Some(l) = update.like {
            sqlx::query(&format!(r#"UPDATE "{table}" SET "like"=?,"updatedAt"=CURRENT_TIMESTAMP WHERE "id"=?"#))
                .bind(l).bind(id).execute(&self.pool).await?;
        }
        if let Some(ref c) = update.comment {
            sqlx::query(&format!(r#"UPDATE "{table}" SET "comment"=?,"updatedAt"=CURRENT_TIMESTAMP WHERE "id"=?"#))
                .bind(c).bind(id).execute(&self.pool).await?;
        }
        Ok(())
    }

    async fn delete_comment(&self, id: &str) -> Result<()> {
        let table = self.comment_table();
        let id: i64 = id.parse().map_err(|_| WalineError::BadRequest("Invalid id".into()))?;
        sqlx::query(&format!(r#"DELETE FROM "{table}" WHERE "pid"=? OR "rid"=?"#))
            .bind(id).bind(id).execute(&self.pool).await?;
        sqlx::query(&format!(r#"DELETE FROM "{table}" WHERE "id"=?"#))
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn total_comments(&self, query: &CommentQuery) -> Result<i64> {
        let table = self.comment_table();
        let url = query.url.as_deref().or(query.path.as_deref()).unwrap_or("");
        let status = query.status.as_deref().unwrap_or("approved");
        let count: (i64,) = if url.is_empty() {
            sqlx::query_as(&format!(r#"SELECT COUNT(*) FROM "{table}""#)).fetch_one(&self.pool).await?
        } else {
            sqlx::query_as(&format!(r#"SELECT COUNT(*) FROM "{table}" WHERE "url"=? AND "status"=?"#))
                .bind(url).bind(status).fetch_one(&self.pool).await?
        };
        Ok(count.0)
    }

    async fn select_counters(&self, urls: &[String]) -> Result<Vec<Counter>> {
        let table = self.counter_table();
        let mut result = Vec::with_capacity(urls.len());
        for url in urls {
            if let Some(row) = sqlx::query(&format!(r#"SELECT * FROM "{table}" WHERE "url"=?"#))
                .bind(url).fetch_optional(&self.pool).await? {
                result.push(Self::row_to_counter(&row));
            }
        }
        Ok(result)
    }

    async fn upsert_counter(&self, url: &str, increment: i64) -> Result<Counter> {
        let table = self.counter_table();
        sqlx::query(&format!(
            r#"INSERT INTO "{table}" ("url","time") VALUES (?,?)
            ON CONFLICT("url") DO UPDATE SET "time"="time"+?,"updatedAt"=CURRENT_TIMESTAMP"#
        )).bind(url).bind(increment).bind(increment).execute(&self.pool).await?;

        let row = sqlx::query(&format!(r#"SELECT * FROM "{table}" WHERE "url"=?"#))
            .bind(url).fetch_one(&self.pool).await?;
        Ok(Self::row_to_counter(&row))
    }

    async fn select_users(&self, query: &UserQuery) -> Result<Vec<User>> {
        let table = self.users_table();
        let page = query.page.unwrap_or(1) as i64;
        let page_size = query.page_size.unwrap_or(10) as i64;
        let offset = (page - 1) * page_size;
        let rows = sqlx::query(&format!(
            r#"SELECT * FROM "{table}" ORDER BY "createdAt" DESC LIMIT ? OFFSET ?"#
        )).bind(page_size).bind(offset).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(Self::row_to_user).collect())
    }

    async fn count_users(&self, _query: &UserQuery) -> Result<i64> {
        let table = self.users_table();
        let count: (i64,) = sqlx::query_as(&format!(r#"SELECT COUNT(*) FROM "{table}""#))
            .fetch_one(&self.pool).await?;
        Ok(count.0)
    }

    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        let table = self.users_table();
        let id: i64 = id.parse().map_err(|_| WalineError::BadRequest("Invalid id".into()))?;
        let row = sqlx::query(&format!(r#"SELECT * FROM "{table}" WHERE "id"=?"#))
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| Self::row_to_user(&r)))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let table = self.users_table();
        let row = sqlx::query(&format!(r#"SELECT * FROM "{table}" WHERE "email"=?"#))
            .bind(email).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| Self::row_to_user(&r)))
    }

    async fn get_user_by_social(&self, provider: &str, id: &str) -> Result<Option<User>> {
        let table = self.users_table();
        let row = sqlx::query(&format!(r#"SELECT * FROM "{table}" WHERE "{provider}"=?"#))
            .bind(id).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| Self::row_to_user(&r)))
    }

    async fn add_user(&self, user: &NewUser) -> Result<User> {
        let table = self.users_table();
        sqlx::query(&format!(
            r#"INSERT INTO "{table}" ("display_name","email","password","type") VALUES (?,?,?,?)"#
        )).bind(&user.display_name).bind(&user.email).bind(&user.password).bind("guest")
        .execute(&self.pool).await?;

        let row = sqlx::query(&format!(r#"SELECT * FROM "{table}" WHERE "id"=last_insert_rowid()"#))
            .fetch_one(&self.pool).await?;
        Ok(Self::row_to_user(&row))
    }

    async fn update_user(&self, id: &str, update: &UserUpdate) -> Result<()> {
        let table = self.users_table();
        let id: i64 = id.parse().map_err(|_| WalineError::BadRequest("Invalid id".into()))?;
        let mut sets = Vec::new();
        if update.display_name.is_some() { sets.push(r#""display_name"=?"#); }
        if update.email.is_some() { sets.push(r#""email"=?"#); }
        if update.password.is_some() { sets.push(r#""password"=?"#); }
        if update.url.is_some() { sets.push(r#""url"=?"#); }
        if update.avatar.is_some() { sets.push(r#""avatar"=?"#); }
        if update.label.is_some() { sets.push(r#""label"=?"#); }
        if update.user_type.is_some() { sets.push(r#""type"=?"#); }
        if sets.is_empty() { return Ok(()); }
        sets.push(r#""updatedAt"=CURRENT_TIMESTAMP"#);

        let sql = format!(r#"UPDATE "{table}" SET {} WHERE "id"=?"#, sets.join(","));
        let mut q = sqlx::query(&sql);
        if let Some(ref v) = update.display_name { q = q.bind(v); }
        if let Some(ref v) = update.email { q = q.bind(v); }
        if let Some(ref v) = update.password { q = q.bind(v); }
        if let Some(ref v) = update.url { q = q.bind(v); }
        if let Some(ref v) = update.avatar { q = q.bind(v); }
        if let Some(ref v) = update.label { q = q.bind(v); }
        if let Some(ref v) = update.user_type { q = q.bind(v); }
        q = q.bind(id);
        q.execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        let table = self.users_table();
        let id: i64 = id.parse().map_err(|_| WalineError::BadRequest("Invalid id".into()))?;
        sqlx::query(&format!(r#"DELETE FROM "{table}" WHERE "id"=?"#))
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn export_all(&self) -> Result<serde_json::Value> {
        let ct = self.comment_table();
        let cnt = self.counter_table();
        let ut = self.users_table();
        let comments: Vec<_> = sqlx::query(&format!(r#"SELECT * FROM "{ct}""#)).fetch_all(&self.pool).await?
            .iter().map(|r| serde_json::to_value(&Self::row_to_comment(r)).unwrap_or_default()).collect();
        let counters: Vec<_> = sqlx::query(&format!(r#"SELECT * FROM "{cnt}""#)).fetch_all(&self.pool).await?
            .iter().map(|r| serde_json::to_value(&Self::row_to_counter(r)).unwrap_or_default()).collect();
        let users: Vec<_> = sqlx::query(&format!(r#"SELECT * FROM "{ut}""#)).fetch_all(&self.pool).await?
            .iter().map(|r| serde_json::to_value(&Self::row_to_user(r)).unwrap_or_default()).collect();
        Ok(serde_json::json!({"Comment": comments, "Counter": counters, "Users": users}))
    }

    async fn import_data(&self, _data: &serde_json::Value) -> Result<()> { Ok(()) }
    async fn update_records(&self, _table: &str, _data: &serde_json::Value) -> Result<()> { Ok(()) }

    async fn clear_table(&self, table_name: &str) -> Result<()> {
        let table = match table_name {
            "Comment" => self.comment_table(),
            "Counter" => self.counter_table(),
            "Users" => self.users_table(),
            _ => return Err(WalineError::BadRequest(format!("Unknown table: {table_name}"))),
        };
        sqlx::query(&format!(r#"DELETE FROM "{table}""#)).execute(&self.pool).await?;
        Ok(())
    }
}
