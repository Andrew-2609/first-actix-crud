use std::{env, time::Duration};

use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Unable to access .env file");

    let server_address = env::var("SERVER_ADDRESS").unwrap_or("0.0.0.0:3000".to_owned());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found in the env file");

    let _db_pool = PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Could not connect to the database");

    let listener = TcpListener::bind(server_address)
        .await
        .expect("Could not create TCP Listener");

    println!("Listening on {}", listener.local_addr().unwrap());

    let app = Router::new().route("/", get(|| async { "Hello, World\n" }));

    axum::serve(listener, app)
        .await
        .expect("Error serving the application")
}
