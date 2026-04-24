use waline_common::models::*;
use waline_common::error::{Result, WalineError};
use sqlx::{Row, FromRow, PgPool, MySqlPool, SqlitePool};
use sqlx::postgres::PgRow;
use sqlx::mysql::MySqlRow;
use sqlx::sqlite::SqliteRow;

/// SQL schema creation for all supported databases
pub async fn create_tables_postgres(pool: &PgPool, prefix: &str) -> Result<()> {
    let comment_table = format!("{prefix}Comment");
    let counter_table = format!("{prefix}Counter");
    let users_table = format!("{prefix}Users");

    sqlx::query(&format!(
        r#"
        CREATE TABLE IF NOT EXISTS "{comment_table}" (
            "id" SERIAL PRIMARY KEY,
            "user_id" TEXT,
            "comment" TEXT NOT NULL DEFAULT '',
            "insertedAt" TIMESTAMP NOT NULL DEFAULT NOW(),
            "ip" TEXT,
            "link" TEXT,
            "mail" TEXT,
            "nick" TEXT,
            "pid" TEXT,
            "rid" TEXT,
            "status" TEXT NOT NULL DEFAULT 'approved',
            "ua" TEXT,
            "url" TEXT,
            "like" INTEGER NOT NULL DEFAULT 0,
            "sticky" BOOLEAN,
            "createdAt" TIMESTAMP NOT NULL DEFAULT NOW(),
            "updatedAt" TIMESTAMP NOT NULL DEFAULT NOW()
        )
        "#
    ))
    .execute(pool)
    .await?;

    sqlx::query(&format!(
        r#"
        CREATE TABLE IF NOT EXISTS "{counter_table}" (
            "id" SERIAL PRIMARY KEY,
            "url" TEXT NOT NULL,
            "time" INTEGER NOT NULL DEFAULT 0,
            "createdAt" TIMESTAMP NOT NULL DEFAULT NOW(),
            "updatedAt" TIMESTAMP NOT NULL DEFAULT NOW()
        )
        "#
    ))
    .execute(pool)
    .await?;

    sqlx::query(&format!(
        r#"
        CREATE TABLE IF NOT EXISTS "{users_table}" (
            "id" SERIAL PRIMARY KEY,
            "display_name" TEXT,
            "email" TEXT,
            "password" TEXT,
            "type" TEXT NOT NULL DEFAULT 'guest',
            "url" TEXT,
            "avatar" TEXT,
            "label" TEXT,
            "github" TEXT,
            "twitter" TEXT,
            "facebook" TEXT,
            "google" TEXT,
            "weibo" TEXT,
            "qq" TEXT,
            "oidc" TEXT,
            "2fa" TEXT,
            "createdAt" TIMESTAMP NOT NULL DEFAULT NOW(),
            "updatedAt" TIMESTAMP NOT NULL DEFAULT NOW()
        )
        "#
    ))
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_tables_mysql(pool: &MySqlPool, prefix: &str) -> Result<()> {
    let comment_table = format!("{prefix}Comment");
    let counter_table = format!("{prefix}Counter");
    let users_table = format!("{prefix}Users");

    sqlx::query(&format!(
        r#"
        CREATE TABLE IF NOT EXISTS `{comment_table}` (
            `id` INT AUTO_INCREMENT PRIMARY KEY,
            `user_id` VARCHAR(255),
            `comment` TEXT NOT NULL,
            `insertedAt` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `ip` VARCHAR(255),
            `link` VARCHAR(255),
            `mail` VARCHAR(255),
            `nick` VARCHAR(255),
            `pid` VARCHAR(255),
            `rid` VARCHAR(255),
            `status` VARCHAR(255) NOT NULL DEFAULT 'approved',
            `ua` VARCHAR(255),
            `url` VARCHAR(255),
            `like` INT NOT NULL DEFAULT 0,
            `sticky` TINYINT(1),
            `createdAt` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updatedAt` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        "#
    ))
    .execute(pool)
    .await?;

    sqlx::query(&format!(
        r#"
        CREATE TABLE IF NOT EXISTS `{counter_table}` (
            `id` INT AUTO_INCREMENT PRIMARY KEY,
            `url` VARCHAR(255) NOT NULL,
            `time` INT NOT NULL DEFAULT 0,
            `createdAt` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updatedAt` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        "#
    ))
    .execute(pool)
    .await?;

    sqlx::query(&format!(
        r#"
        CREATE TABLE IF NOT EXISTS `{users_table}` (
            `id` INT AUTO_INCREMENT PRIMARY KEY,
            `display_name` VARCHAR(255),
            `email` VARCHAR(255),
            `password` VARCHAR(255),
            `type` VARCHAR(255) NOT NULL DEFAULT 'guest',
            `url` VARCHAR(255),
            `avatar` VARCHAR(255),
            `label` VARCHAR(255),
            `github` VARCHAR(255),
            `twitter` VARCHAR(255),
            `facebook` VARCHAR(255),
            `google` VARCHAR(255),
            `weibo` VARCHAR(255),
            `qq` VARCHAR(255),
            `oidc` VARCHAR(255),
            `2fa` VARCHAR(255),
            `createdAt` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updatedAt` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        "#
    ))
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_tables_sqlite(pool: &SqlitePool, prefix: &str) -> Result<()> {
    let comment_table = format!("{prefix}Comment");
    let counter_table = format!("{prefix}Counter");
    let users_table = format!("{prefix}Users");

    sqlx::query(&format!(
        r#"
        CREATE TABLE IF NOT EXISTS "{comment_table}" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "user_id" TEXT,
            "comment" TEXT NOT NULL DEFAULT '',
            "insertedAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "ip" TEXT,
            "link" TEXT,
            "mail" TEXT,
            "nick" TEXT,
            "pid" TEXT,
            "rid" TEXT,
            "status" TEXT NOT NULL DEFAULT 'approved',
            "ua" TEXT,
            "url" TEXT,
            "like" INTEGER NOT NULL DEFAULT 0,
            "sticky" INTEGER,
            "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "updatedAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    ))
    .execute(pool)
    .await?;

    sqlx::query(&format!(
        r#"
        CREATE TABLE IF NOT EXISTS "{counter_table}" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "url" TEXT NOT NULL,
            "time" INTEGER NOT NULL DEFAULT 0,
            "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "updatedAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    ))
    .execute(pool)
    .await?;

    sqlx::query(&format!(
        r#"
        CREATE TABLE IF NOT EXISTS "{users_table}" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "display_name" TEXT,
            "email" TEXT,
            "password" TEXT,
            "type" TEXT NOT NULL DEFAULT 'guest',
            "url" TEXT,
            "avatar" TEXT,
            "label" TEXT,
            "github" TEXT,
            "twitter" TEXT,
            "facebook" TEXT,
            "google" TEXT,
            "weibo" TEXT,
            "qq" TEXT,
            "oidc" TEXT,
            "2fa" TEXT,
            "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "updatedAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    ))
    .execute(pool)
    .await?;

    Ok(())
}

/// Helper to map a DB row `id` to `objectId` in the API response
pub fn map_id_to_object_id(id: i64) -> String {
    id.to_string()
}
