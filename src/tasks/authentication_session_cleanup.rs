use crate::entity::session;
use chrono::Utc;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use tokio::time;

pub fn spawn(db: DatabaseConnection) {
    tokio::spawn(async move {
        let mut interval = time::interval(time::Duration::from_mins(60));
        interval.tick().await;

        loop {
            interval.tick().await;
            run_cleanup(&db).await;
        }
    });
}

async fn run_cleanup(db: &DatabaseConnection) {
    let cutoff = Utc::now() - chrono::Duration::days(30);

    session::Entity::delete_many()
        .filter(
            Condition::any()
                .add(session::Column::ExpiresAt.lt(cutoff))
                .add(session::Column::RevokedAt.lt(cutoff)),
        )
        .exec(db)
        .await
        .unwrap();
}
