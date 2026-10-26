use crate::entity::session;
use chrono::Utc;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use tokio::time::{self, MissedTickBehavior};

pub fn spawn(db: DatabaseConnection) {
    tokio::spawn(async move {
        let mut interval = time::interval(time::Duration::from_hours(24));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let _ = run_cleanup(&db).await;
        }
    });
}

async fn run_cleanup(db: &DatabaseConnection) -> Result<(), DbErr> {
    let now = Utc::now();

    session::Entity::delete_many()
        .filter(Condition::any().add(session::Column::ExpiresAt.lte(now)))
        .exec(db)
        .await?;

    Ok(())
}
