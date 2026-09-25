mod authentication;
mod database;
mod entity;
mod error;
mod extract;
mod http;
mod rate_limit;
mod state;
mod tasks;

use axum::serve;
use dotenvy::dotenv;
use http::router;
use state::AppState;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db = database::init_db().await.expect("failed to init db");

    tasks::authentication_session_cleanup::spawn(db.clone());

    let secret = env::var("SECRET").expect("SECRET must be set").into_bytes();
    let state = AppState { db, secret };

    let app = router(state.clone()).with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!(
        "Listening on {} port",
        listener.local_addr().unwrap().port()
    );
    serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
