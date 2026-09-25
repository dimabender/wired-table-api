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
    let port = env::var("API_PORT").expect("API_PORT must be set");

    let state = AppState { db, secret };

    let app = router(state.clone()).with_state(state);

    let address = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&address).await.unwrap();

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
