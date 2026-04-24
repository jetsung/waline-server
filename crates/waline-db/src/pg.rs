use async_trait::async_trait;
use sqlx::PgPool;
use waline_common::error::{Result, WalineError};
use waline_common::models::*;
use crate::adapter::DatabaseAdapter;
use crate::schema::create_tables_postgres;

pub struct PostgresAdapter {
    pool: PgPool,
    prefix: String,
}

impl PostgresAdapter {
    pub async fn new(database_url: &str, prefix: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await.map_err(|e| {
            WalineError::Config(format!("Failed to connect to PostgreSQL: {e}"))
        })?;
        create_tables_postgres(&pool, prefix).await?;
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

    fn row_to_comment(row: &sqlx::postgres::PgRow) -> Comment {
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
            sticky: row.try_get::<bool, _>("sticky").ok(),
            like: row.try_get::<i32, _>("like").unwrap_or(0),
            created_at: row.try_get("createdAt").unwrap_or(chrono::Utc::now()),
            updated_at: row.try_get("updatedAt").unwrap_or(chrono::Utc::now()),
            avatar: None,
            level: None,
            region: None,
            browser: None,
            os: None,
            children: Vec::new(),
        }
    }

    fn row_to_counter(row: &sqlx::postgres::PgRow) -> Counter {
        use sqlx::Row;
        Counter {
            object_id: row.get::<i64, _>("id").to_string(),
            url: row.try_get::<String, _>("url").unwrap_or_default(),
            time: row.try_get::<i64, _>("time").unwrap_or(0),
            created_at: row.try_get("createdAt").unwrap_or(chrono::Utc::now()),
            updated_at: row.try_get("updatedAt").unwrap_or(chrono::Utc::now()),
        }
    }

    fn row_to_user(row: &sqlx::postgres::PgRow) -> User {
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
impl DatabaseAdapter for PostgresAdapter {
    async fn select_comments(&self, query: &CommentQuery) -> Result<Vec<Comment>> {
        let table = self.comment_table();
        let url = query.url.as_deref().or(query.path.as_deref()).unwrap_or("");
        let status = query.status.as_deref().unwrap_or("approved");

        match query.query_type.as_deref() {
            Some("count") => {
                // Not used for select, use count_comments instead
                Ok(Vec::new())
            }
            Some("recent") => {
                let limit = query.page_size.unwrap_or(50).min(50) as i64;
                let rows = sqlx::query(&format!(
                    r#"SELECT * FROM "{table}" WHERE "status" = $1 ORDER BY "insertedAt" DESC LIMIT $2"#
                ))
                .bind(status)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.iter().map(Self::row_to_comment).collect())
            }
            Some("list") => {
                // Admin list with pagination
                let page = query.page.unwrap_or(1) as i64;
                let page_size = query.page_size.unwrap_or(10) as i64;
                let offset = (page - 1) * page_size;
                let rows = sqlx::query(&format!(
                    r#"SELECT * FROM "{table}" WHERE 1=1 ORDER BY "insertedAt" DESC LIMIT $1 OFFSET $2"#
                ))
                .bind(page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.iter().map(Self::row_to_comment).collect())
            }
            _ => {
                // Default: get comments for a page
                let sortby = query.sortby.as_deref().unwrap_or("insertedAt_desc");
                let (order_col, order_dir) = match sortby {
                    "insertedAt_asc" => ("insertedAt", "ASC"),
                    "like_desc" => ("like", "DESC"),
                    _ => ("insertedAt", "DESC"),
                };
                let rows = sqlx::query(&format!(
                    r#"SELECT * FROM "{table}" WHERE "url" = $1 AND "status" = $2 ORDER BY "{order_col}" {order_dir}"#
                ))
                .bind(url)
                .bind(status)
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.iter().map(Self::row_to_comment).collect())
            }
        }
    }

    async fn count_comments(&self, urls: &[String]) -> Result<Vec<i64>> {
        let table = self.comment_table();
        let mut counts = Vec::with_capacity(urls.len());
        for url in urls {
            let count: (i64,) = sqlx::query_as(&format!(
                r#"SELECT COUNT(*) FROM "{table}" WHERE "url" = $1 AND "status" = 'approved'"#
            ))
            .bind(url)
            .fetch_one(&self.pool)
            .await?;
            counts.push(count.0);
        }
        Ok(counts)
    }

    async fn count_comments_by_status(&self, url: &str, status: &str) -> Result<i64> {
        let table = self.comment_table();
        let count: (i64,) = sqlx::query_as(&format!(
            r#"SELECT COUNT(*) FROM "{table}" WHERE "url" = $1 AND "status" = $2"#
        ))
        .bind(url)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;
        Ok(count.0)
    }

    async fn add_comment(&self, comment: &NewComment) -> Result<Comment> {
        let table = self.comment_table();
        let row = sqlx::query(&format!(
            r#"INSERT INTO "{table}" ("user_id", "comment", "ip", "link", "mail", "nick", "pid", "rid", "status", "ua", "url", "sticky")
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *"#
        ))
        .bind(&comment.user_id)
        .bind(&comment.comment)
        .bind(&comment.ip)
        .bind(&comment.link)
        .bind(&comment.mail)
        .bind(&comment.nick)
        .bind(&comment.pid)
        .bind(&comment.rid)
        .bind("approved")
        .bind(&comment.ua)
        .bind(&comment.url)
        .bind(comment.sticky)
        .fetch_one(&self.pool)
        .await?;
        Ok(Self::row_to_comment(&row))
    }

    async fn update_comment(&self, id: &str, update: &CommentUpdate) -> Result<()> {
        let table = self.comment_table();
        let id: i64 = id.parse().map_err(|_| WalineError::BadRequest("Invalid comment id".to_string()))?;

        if let Some(ref status) = update.status {
            sqlx::query(&format!(r#"UPDATE "{table}" SET "status" = $1, "updatedAt" = NOW() WHERE "id" = $2"#))
                .bind(status)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(like) = update.like {
            sqlx::query(&format!(r#"UPDATE "{table}" SET "like" = $1, "updatedAt" = NOW() WHERE "id" = $2"#))
                .bind(like)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(sticky) = update.sticky {
            sqlx::query(&format!(r#"UPDATE "{table}" SET "sticky" = $1, "updatedAt" = NOW() WHERE "id" = $2"#))
                .bind(sticky)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(ref comment) = update.comment {
            sqlx::query(&format!(r#"UPDATE "{table}" SET "comment" = $1, "updatedAt" = NOW() WHERE "id" = $2"#))
                .bind(comment)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn delete_comment(&self, id: &str) -> Result<()> {
        let table = self.comment_table();
        let id: i64 = id.parse().map_err(|_| WalineError::BadRequest("Invalid comment id".to_string()))?;
        // Delete children (pid or rid matches)
        sqlx::query(&format!(r#"DELETE FROM "{table}" WHERE "pid" = $1 OR "rid" = $1"#))
            .bind(id)
            .execute(&self.pool)
            .await?;
        // Delete the comment itself
        sqlx::query(&format!(r#"DELETE FROM "{table}" WHERE "id" = $1"#))
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn total_comments(&self, query: &CommentQuery) -> Result<i64> {
        let table = self.comment_table();
        let url = query.url.as_deref().or(query.path.as_deref()).unwrap_or("");
        let status = query.status.as_deref().unwrap_or("approved");

        let count: (i64,) = if url.is_empty() {
            sqlx::query_as(&format!(r#"SELECT COUNT(*) FROM "{table}""#))
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as(&format!(
                r#"SELECT COUNT(*) FROM "{table}" WHERE "url" = $1 AND "status" = $2"#
            ))
            .bind(url)
            .bind(status)
            .fetch_one(&self.pool)
            .await?
        };
        Ok(count.0)
    }

    async fn select_counters(&self, urls: &[String]) -> Result<Vec<Counter>> {
        let table = self.counter_table();
        let mut result = Vec::with_capacity(urls.len());
        for url in urls {
            let row = sqlx::query(&format!(r#"SELECT * FROM "{table}" WHERE "url" = $1"#))
                .bind(url)
                .fetch_optional(&self.pool)
                .await?;
            if let Some(row) = row {
                result.push(Self::row_to_counter(&row));
            }
        }
        Ok(result)
    }

    async fn upsert_counter(&self, url: &str, increment: i64) -> Result<Counter> {
        let table = self.counter_table();
        let row = sqlx::query(&format!(
            r#"INSERT INTO "{table}" ("url", "time") VALUES ($1, $2)
            ON CONFLICT ("url") DO UPDATE SET "time" = "{table}"."time" + $2, "updatedAt" = NOW()
            RETURNING *"#
        ))
        .bind(url)
        .bind(increment)
        .fetch_one(&self.pool)
        .await?;
        Ok(Self::row_to_counter(&row))
    }

    async fn select_users(&self, query: &UserQuery) -> Result<Vec<User>> {
        let table = self.users_table();
        match query.query_type.as_deref() {
            Some("admin") => {
                let page = query.page.unwrap_or(1) as i64;
                let page_size = query.page_size.unwrap_or(10) as i64;
                let offset = (page - 1) * page_size;
                let rows = sqlx::query(&format!(
                    r#"SELECT * FROM "{table}" ORDER BY "createdAt" DESC LIMIT $1 OFFSET $2"#
                ))
                .bind(page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.iter().map(Self::row_to_user).collect())
            }
            _ => {
                // Public: top commenters
                let limit = query.page_size.unwrap_or(10) as i64;
                let rows = sqlx::query(&format!(
                    r#"SELECT u.*, COUNT(c."id") as comment_count FROM "{table}" u
                    LEFT JOIN "{}Comment" c ON c."user_id" = u."id"::TEXT AND c."status" = 'approved'
                    WHERE u."type" NOT LIKE 'verify%' AND u."type" != 'banned'
                    GROUP BY u."id"
                    ORDER BY comment_count DESC LIMIT $1"#,
                    self.prefix
                ))
                .bind(limit)
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.iter().map(Self::row_to_user).collect())
            }
        }
    }

    async fn count_users(&self, _query: &UserQuery) -> Result<i64> {
        let table = self.users_table();
        let count: (i64,) = sqlx::query_as(&format!(r#"SELECT COUNT(*) FROM "{table}""#))
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0)
    }

    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        let table = self.users_table();
        let id: i64 = id.parse().map_err(|_| WalineError::BadRequest("Invalid user id".to_string()))?;
        let row = sqlx::query(&format!(r#"SELECT * FROM "{table}" WHERE "id" = $1"#))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| Self::row_to_user(&r)))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let table = self.users_table();
        let row = sqlx::query(&format!(r#"SELECT * FROM "{table}" WHERE "email" = $1"#))
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| Self::row_to_user(&r)))
    }

    async fn get_user_by_social(&self, provider: &str, id: &str) -> Result<Option<User>> {
        let table = self.users_table();
        let row = sqlx::query(&format!(r#"SELECT * FROM "{table}" WHERE "{provider}" = $1"#))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| Self::row_to_user(&r)))
    }

    async fn add_user(&self, user: &NewUser) -> Result<User> {
        let table = self.users_table();
        let row = sqlx::query(&format!(
            r#"INSERT INTO "{table}" ("display_name", "email", "password", "type")
            VALUES ($1, $2, $3, $4)
            RETURNING *"#
        ))
        .bind(&user.display_name)
        .bind(&user.email)
        .bind(&user.password)
        .bind("guest")
        .fetch_one(&self.pool)
        .await?;
        Ok(Self::row_to_user(&row))
    }

    async fn update_user(&self, id: &str, update: &UserUpdate) -> Result<()> {
        let table = self.users_table();
        let id: i64 = id.parse().map_err(|_| WalineError::BadRequest("Invalid user id".to_string()))?;
        let mut sets = Vec::new();
        let mut param_idx = 1u32;

        macro_rules! add_field {
            ($field:expr, $val:expr) => {
                if let Some(ref v) = $val {
                    sets.push(format!(r#""{}" = ${}"#, $field, param_idx));
                    param_idx += 1;
                }
            };
        }

        add_field!("display_name", update.display_name);
        add_field!("email", update.email);
        add_field!("password", update.password);
        add_field!("url", update.url);
        add_field!("avatar", update.avatar);
        add_field!("label", update.label);
        add_field!("type", update.user_type);

        if sets.is_empty() {
            return Ok(());
        }

        sets.push(r#""updatedAt" = NOW()"#.to_string());
        let sql = format!(r#"UPDATE "{table}" SET {} WHERE "id" = ${param_idx}"#, sets.join(", "));
        let mut query = sqlx::query(&sql);

        macro_rules! bind_field {
            ($val:expr) => {
                if let Some(ref v) = $val {
                    query = query.bind(v);
                }
            };
        }

        bind_field!(update.display_name);
        bind_field!(update.email);
        bind_field!(update.password);
        bind_field!(update.url);
        bind_field!(update.avatar);
        bind_field!(update.label);
        bind_field!(update.user_type);

        query = query.bind(id);
        query.execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        let table = self.users_table();
        let id: i64 = id.parse().map_err(|_| WalineError::BadRequest("Invalid user id".to_string()))?;
        sqlx::query(&format!(r#"DELETE FROM "{table}" WHERE "id" = $1"#))
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn export_all(&self) -> Result<serde_json::Value> {
        let comment_table = self.comment_table();
        let counter_table = self.counter_table();
        let users_table = self.users_table();

        let comments: Vec<serde_json::Value> = sqlx::query(&format!(r#"SELECT * FROM "{comment_table}""#))
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|r| {
                let c = Self::row_to_comment(r);
                serde_json::to_value(&c).unwrap_or_default()
            })
            .collect();

        let counters: Vec<serde_json::Value> = sqlx::query(&format!(r#"SELECT * FROM "{counter_table}""#))
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|r| {
                let c = Self::row_to_counter(r);
                serde_json::to_value(&c).unwrap_or_default()
            })
            .collect();

        let users: Vec<serde_json::Value> = sqlx::query(&format!(r#"SELECT * FROM "{users_table}""#))
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|r| {
                let u = Self::row_to_user(r);
                serde_json::to_value(&u).unwrap_or_default()
            })
            .collect();

        Ok(serde_json::json!({
            "Comment": comments,
            "Counter": counters,
            "Users": users,
        }))
    }

    async fn import_data(&self, data: &serde_json::Value) -> Result<()> {
        // For each table in the data, insert rows
        if let Some(comments) = data.get("Comment").and_then(|v| v.as_array()) {
            for comment_data in comments {
                let c: Comment = serde_json::from_value(comment_data.clone())
                    .map_err(|e| WalineError::BadRequest(format!("Invalid comment data: {e}")))?;
                let table = self.comment_table();
                sqlx::query(&format!(
                    r#"INSERT INTO "{table}" ("id", "user_id", "comment", "insertedAt", "ip", "link", "mail", "nick", "pid", "rid", "status", "ua", "url", "like", "sticky", "createdAt", "updatedAt")
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)"#
                ))
                .bind(c.object_id.parse::<i64>().unwrap_or(0))
                .bind(&c.user_id)
                .bind(&c.comment)
                .bind(c.inserted_at)
                .bind(&c.ip)
                .bind(&c.link)
                .bind(&c.mail)
                .bind(&c.nick)
                .bind(&c.pid)
                .bind(&c.rid)
                .bind(&c.status)
                .bind(&c.ua)
                .bind(&c.url)
                .bind(c.like)
                .bind(c.sticky)
                .bind(c.created_at)
                .bind(c.updated_at)
                .execute(&self.pool)
                .await?;
            }
        }
        Ok(())
    }

    async fn update_records(&self, _table: &str, _data: &serde_json::Value) -> Result<()> {
        // TODO: implement generic record update
        Ok(())
    }

    async fn clear_table(&self, table_name: &str) -> Result<()> {
        let table = match table_name {
            "Comment" => self.comment_table(),
            "Counter" => self.counter_table(),
            "Users" => self.users_table(),
            _ => return Err(WalineError::BadRequest(format!("Unknown table: {table_name}"))),
        };
        sqlx::query(&format!(r#"DELETE FROM "{table}""#))
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
