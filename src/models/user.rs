use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "wl_Users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub display_name: String,
    pub email: String,
    pub password: String,
    #[sea_orm(column_name = "type")]
    pub user_type: String,
    pub label: Option<String>,
    pub url: Option<String>,
    pub avatar: Option<String>,
    pub github: Option<String>,
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub google: Option<String>,
    pub weibo: Option<String>,
    pub qq: Option<String>,
    pub oidc: Option<String>,
    pub huawei: Option<String>,
    #[sea_orm(column_name = "2fa")]
    pub two_factor_auth: Option<String>,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: Option<String>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn is_admin(&self) -> bool {
        self.user_type == "administrator"
    }

    pub fn is_verified(&self) -> bool {
        !self.user_type.starts_with("verify:") && self.user_type != "banned"
    }
}
