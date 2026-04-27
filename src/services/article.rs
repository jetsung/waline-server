use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::{Value, json};

use crate::{db::Db, error::AppError, models::counter};

pub async fn get_counters(db: &Db, paths: &[String], types: &[String]) -> Result<Value, AppError> {
    if paths.is_empty() {
        return Ok(json!([]));
    }

    // Fetch all matching rows
    let rows = counter::Entity::find()
        .filter(counter::Column::Url.is_in(paths.iter().map(|s| s.as_str()).collect::<Vec<_>>()))
        .all(db)
        .await?;

    let resp_map: std::collections::HashMap<&str, &counter::Model> =
        rows.iter().map(|r| (r.url.as_str(), r)).collect();

    let data: Vec<Value> = paths
        .iter()
        .map(|path| {
            let mut obj = serde_json::Map::new();
            for t in types {
                let val = resp_map
                    .get(path.as_str())
                    .and_then(|r| counter_field(r, t))
                    .unwrap_or(0);
                obj.insert(t.clone(), json!(val));
            }
            Value::Object(obj)
        })
        .collect();

    Ok(json!(data))
}

pub async fn update_counter(
    db: &Db,
    path: &str,
    field: &str,
    action: &str,
) -> Result<Value, AppError> {
    use crate::models::counter::ActiveModel;

    let existing = counter::Entity::find()
        .filter(counter::Column::Url.eq(path))
        .one(db)
        .await?;

    let new_val = if let Some(ref row) = existing {
        let cur = counter_field(row, field).unwrap_or(0);
        if action == "desc" {
            (cur - 1).max(0)
        } else {
            cur + 1
        }
    } else {
        if action == "desc" {
            return Ok(json!([{field: 0}]));
        }
        1
    };

    if let Some(row) = existing {
        let mut active: ActiveModel = row.into();
        set_counter_field(&mut active, field, new_val);
        active.update(db).await?;
    } else {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut active = ActiveModel {
            url: Set(path.to_string()),
            created_at: Set(Some(now.clone())),
            updated_at: Set(Some(now)),
            ..Default::default()
        };
        set_counter_field(&mut active, field, new_val);
        active.insert(db).await?;
    }

    Ok(json!([{field: new_val}]))
}

fn counter_field(row: &counter::Model, field: &str) -> Option<i64> {
    match field {
        "time" => row.time,
        "reaction0" => row.reaction0,
        "reaction1" => row.reaction1,
        "reaction2" => row.reaction2,
        "reaction3" => row.reaction3,
        "reaction4" => row.reaction4,
        "reaction5" => row.reaction5,
        "reaction6" => row.reaction6,
        "reaction7" => row.reaction7,
        "reaction8" => row.reaction8,
        _ => None,
    }
}

fn set_counter_field(active: &mut counter::ActiveModel, field: &str, val: i64) {
    match field {
        "time" => active.time = Set(Some(val)),
        "reaction0" => active.reaction0 = Set(Some(val)),
        "reaction1" => active.reaction1 = Set(Some(val)),
        "reaction2" => active.reaction2 = Set(Some(val)),
        "reaction3" => active.reaction3 = Set(Some(val)),
        "reaction4" => active.reaction4 = Set(Some(val)),
        "reaction5" => active.reaction5 = Set(Some(val)),
        "reaction6" => active.reaction6 = Set(Some(val)),
        "reaction7" => active.reaction7 = Set(Some(val)),
        "reaction8" => active.reaction8 = Set(Some(val)),
        _ => {}
    }
}
