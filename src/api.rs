use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::db;
use crate::import;
use crate::sm2;

pub type AppState = Arc<Mutex<rusqlite::Connection>>;

#[derive(Deserialize)]
pub struct AddWordRequest {
    pub english: String,
    pub russian: String,
}

#[derive(Deserialize)]
pub struct ReviewRequest {
    pub quality: u8,
}

#[derive(Deserialize)]
pub struct ImportRequest {
    pub content: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

pub fn build_router(conn: rusqlite::Connection) -> Router {
    let state: AppState = Arc::new(Mutex::new(conn));

    Router::new()
        .route("/api/words", get(get_words))
        .route("/api/add", post(add_word))
        .route("/api/update/{id}", put(update_word_handler))
        .route("/api/del/{id}", delete(delete_word))
        .route("/api/rev", get(get_review_words))
        .route("/api/review/{id}", post(review_word))
        .route("/api/import", post(import_words_handler))
        .with_state(state)
}

async fn get_words(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let conn = state
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let words = db::get_all_words(&conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(words))
}

async fn add_word(
    State(state): State<AppState>,
    body: String,
) -> Result<impl IntoResponse, StatusCode> {
    let payload: AddWordRequest =
        serde_json::from_str(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let conn = state
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let word = db::add_word(&conn, &payload.english, &payload.russian)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(word)))
}

async fn update_word_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    body: String,
) -> Result<impl IntoResponse, StatusCode> {
    let payload: AddWordRequest =
        serde_json::from_str(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let conn = state
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let word = db::update_word(&conn, id, &payload.english, &payload.russian)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(word))
}

async fn delete_word(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, StatusCode> {
    let conn = state
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    db::delete_word(&conn, id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        message: format!("Word {id} deleted"),
    }))
}

async fn get_review_words(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let conn = state
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let words = db::get_words_for_review(&conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(words))
}

async fn review_word(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    body: String,
) -> Result<impl IntoResponse, StatusCode> {
    let payload: ReviewRequest =
        serde_json::from_str(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let conn = state
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let word = db::get_word(&conn, id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let (new_interval, new_ef, new_reps) = sm2::calculate(
        payload.quality,
        word.repetitions,
        word.ease_factor,
        word.interval,
    );
    db::save_review(&conn, id, payload.quality, new_interval, new_ef, new_reps)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        message: format!("Reviewed: interval={new_interval} days, ef={new_ef:.2}, reps={new_reps}"),
    }))
}

async fn import_words_handler(
    State(state): State<AppState>,
    body: String,
) -> Result<impl IntoResponse, StatusCode> {
    let payload: ImportRequest =
        serde_json::from_str(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let conn = state
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let words = import::parse_markdown(&payload.content);
    let count = db::import_words(&conn, &words).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            message: format!("Imported {count} words"),
        }),
    ))
}
