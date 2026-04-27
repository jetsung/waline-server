use chrono::Utc;
use sea_orm::{ConnectionTrait, EntityTrait, Statement};
use serde_json::{Value, json};

use crate::{db::Db, error::AppError, models::{comment, counter, user}};

pub async fn export(db: &Db) -> Result<Value, AppError> {
    let comments = comment::Entity::find().all(db).await?;
    let counters = counter::Entity::find().all(db).await?;
    let users = user::Entity::find().all(db).await?;

    Ok(json!({
        "type": "waline",
        "version": 1,
        "time": Utc::now().timestamp_millis(),
        "tables": ["Comment", "Counter", "Users"],
        "data": { "Comment": comments, "Counter": counters, "Users": users }
    }))
}

pub async fn insert_record(db: &Db, table: &str, data: &Value) -> Result<i64, AppError> {
    let obj = data.as_object().ok_or(AppError::Internal("invalid data".into()))?;
    let cols: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
    let placeholders: Vec<&str> = cols.iter().map(|_| "?").collect();
    let tbl = table_name(table)?;
    let sql = format!(
        r#"INSERT INTO "{tbl}" ({}) VALUES ({})"#,
        cols.iter().map(|c| format!("\"{c}\"")).collect::<Vec<_>>().join(","),
        placeholders.join(",")
    );
    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        sql,
        cols.iter().map(|k| value_to_sea_value(&obj[*k])).collect::<Vec<_>>(),
    );
    let result = db.execute(stmt).await?;
    Ok(result.last_insert_id() as i64)
}

pub async fn update_record(db: &Db, table: &str, object_id: i64, data: &Value) -> Result<(), AppError> {
    let obj = data.as_object().ok_or(AppError::Internal("invalid data".into()))?;
    if obj.is_empty() {
        return Ok(());
    }
    let tbl = table_name(table)?;
    let sets: Vec<String> = obj.keys().map(|k| format!("\"{k}\"=?")).collect();
    let sql = format!(
        r#"UPDATE "{tbl}" SET {},"updatedAt"=CURRENT_TIMESTAMP WHERE id=?"#,
        sets.join(",")
    );
    let mut values: Vec<sea_orm::Value> = obj.keys().map(|k| value_to_sea_value(&obj[k])).collect();
    values.push(sea_orm::Value::BigInt(Some(object_id)));
    let stmt = Statement::from_sql_and_values(db.get_database_backend(), sql, values);
    db.execute(stmt).await?;
    Ok(())
}

pub async fn clear_table(db: &Db, table: &str) -> Result<(), AppError> {
    let tbl = table_name(table)?;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        format!(r#"DELETE FROM "{tbl}""#),
    ))
    .await?;
    Ok(())
}

fn table_name(table: &str) -> Result<String, AppError> {
    match table {
        "Comment" => Ok("wl_Comment".into()),
        "Counter" => Ok("wl_Counter".into()),
        "Users" | "User" => Ok("wl_Users".into()),
        _ => Err(AppError::Internal(format!("Unknown table: {table}"))),
    }
}

fn value_to_sea_value(v: &Value) -> sea_orm::Value {
    match v {
        Value::String(s) => sea_orm::Value::String(Some(Box::new(s.clone()))),
        Value::Number(n) => sea_orm::Value::BigInt(n.as_i64()),
        Value::Bool(b) => sea_orm::Value::Bool(Some(*b)),
        Value::Null => sea_orm::Value::String(None),
        _ => sea_orm::Value::String(Some(Box::new(v.to_string()))),
    }
}
