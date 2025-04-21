use std::{env, time::Duration};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tokio::net::TcpListener;

#[derive(Serialize)]
struct TaskRow {
    task_id: i32,
    name: String,
    priority: Option<i32>,
}

async fn get_tasks(
    State(pg_pool): State<Pool<Postgres>>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let rows = sqlx::query_as!(TaskRow, "SELECT * FROM tasks ORDER BY task_id")
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": e.to_string()}).to_string(),
            )
        })?;

    Ok((
        StatusCode::OK,
        json!({"success": true, "data": rows}).to_string() + "\n",
    ))
}

async fn get_task(
    State(pg_pool): State<Pool<Postgres>>,
    Path(task_id): Path<i32>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let row = sqlx::query_as!(TaskRow, "SELECT * FROM tasks WHERE task_id = $1", task_id)
        .fetch_one(&pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": e.to_string()}).to_string(),
            )
        })?;

    Ok((
        StatusCode::OK,
        json!({"success": true, "data": row}).to_string() + "\n",
    ))
}

#[derive(Deserialize)]
struct CreateTaskReq {
    name: String,
    priority: Option<i32>,
}

#[derive(Serialize)]
struct CreateTaskRow {
    task_id: i32,
}

async fn create_task(
    State(pg_pool): State<Pool<Postgres>>,
    Json(task): Json<CreateTaskReq>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let row = sqlx::query_as!(
        CreateTaskRow,
        "INSERT INTO tasks (name, priority) VALUES ($1, $2) RETURNING task_id",
        task.name,
        task.priority
    )
    .fetch_one(&pg_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": e.to_string()}).to_string(),
        )
    })?;

    Ok((
        StatusCode::OK,
        json!({"success": true, "data": row}).to_string() + "\n",
    ))
}

#[derive(Deserialize)]
struct UpdateTaskReq {
    name: Option<String>,
    priority: Option<i32>,
}

async fn update_task(
    State(pg_pool): State<Pool<Postgres>>,
    Path(task_id): Path<i32>,
    Json(task): Json<UpdateTaskReq>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    sqlx::query!(
        "UPDATE tasks SET name = $2, priority = $3 WHERE task_id = $1",
        task_id,
        task.name,
        task.priority
    )
    .execute(&pg_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": e.to_string()}).to_string(),
        )
    })?;

    Ok((StatusCode::OK, json!({"success": true}).to_string()))
}

async fn delete_task(
    State(pg_pool): State<Pool<Postgres>>,
    Path(task_id): Path<i32>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    sqlx::query!("DELETE FROM tasks WHERE task_id = $1", task_id)
        .execute(&pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"success": false, "message": e.to_string()}).to_string(),
            )
        })?;

    Ok((StatusCode::OK, json!({"success": true}).to_string()))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Unable to access .env file");

    let server_address = env::var("SERVER_ADDRESS").unwrap_or("0.0.0.0:3000".to_owned());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found in the env file");

    let db_pool = PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Could not connect to the database");

    let listener = TcpListener::bind(server_address)
        .await
        .expect("Could not create TCP Listener");

    println!("Listening on {}", listener.local_addr().unwrap());

    let app = Router::new()
        .route("/", get(|| async { "Hello, World\n" }))
        .route("/tasks", get(get_tasks).post(create_task))
        .route(
            "/tasks/:task_id",
            get(get_task).patch(update_task).delete(delete_task),
        )
        .with_state(db_pool);

    axum::serve(listener, app)
        .await
        .expect("Error serving the application")
}
