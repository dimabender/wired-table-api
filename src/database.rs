use std::env;

use sea_orm::{Database, DatabaseConnection, DbErr};

pub async fn init_db() -> Result<DatabaseConnection, DbErr> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::connect(db_url).await?;

    db.get_schema_registry("wired_table_api::entity::*")
        .sync(&db)
        .await?;

    Ok(db)
}
