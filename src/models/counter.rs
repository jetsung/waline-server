use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "wl_Counter")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub time: Option<i64>,
    pub reaction0: Option<i64>,
    pub reaction1: Option<i64>,
    pub reaction2: Option<i64>,
    pub reaction3: Option<i64>,
    pub reaction4: Option<i64>,
    pub reaction5: Option<i64>,
    pub reaction6: Option<i64>,
    pub reaction7: Option<i64>,
    pub reaction8: Option<i64>,
    pub url: String,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: Option<String>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
