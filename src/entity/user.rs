use chrono::Utc;
use sea_orm::{ActiveValue::Set, entity::prelude::*};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        default_expr = "Expr::cust(\"uuidv7()\")"
    )]
    pub id: Uuid,

    #[sea_orm(unique, column_type = "String(StringLen::N(32))")]
    pub username: String,
    #[sea_orm(unique, column_type = "String(StringLen::N(254))")]
    pub email: String,
    #[sea_orm(column_type = "String(StringLen::N(255))")]
    pub password: String,

    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,

    #[sea_orm(has_many)]
    pub session: HasMany<super::session::Entity>,
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert {
            self.updated_at = Set(Utc::now())
        }
        Ok(self)
    }
}
