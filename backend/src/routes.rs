use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use aws_sdk_s3::Client as S3Client;

use crate::{
    config::Config,
    error::AppError,
    models::{Video, VideoResponse},
    r2
};

// - Shared state injected into every handler -----------------

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub r2_client: S3Client,
    pub config: Config,
}

// - Router -----------------------------------------------------

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/upload/init", post(upload_init))
        .route("/upload/complete/:video_id", post(upload_complete))
        .route("/video/:video_id", get(get_video))
        .route("/video/:video_id/stream", get(get_stream_url))
        .route("/health", get(health))
}

// -- POST /api/upload/init ------------------

#[derive(Deserialize)]
pub struct UploadInitRequest {
    pub filename: String,
    pub size_bytes: Option<i64>,
    pub mime_type: Option<String>,
}

#[derive(Serialize)]
pub struct UploadInitResponse {
    pub video_id: Uuid,
    pub upload_url: String, // presigned PUT URL -> browser uploads directly to R2
}

async fn upload_init(
    State(state): State<AppState>,
    Json(body): Json<UploadInitRequest>,
) -> Result<Json<UploadInitResponse>, AppError> {
    // Validate mime type - only browser-native formats
    let allowed = ["video/mp4", "video/webm", "video/ogg", "video/quicktime"];
    if let Some(ref mime) = body.mime_type {
        if !allowed.contains(&mime.as_str()) {
            return Err(AppError::BadRequest(format!(
                "Unsupported format '{mime}'. Please upload MP4, WebM, OGG, or MOV."
            )));
        }
    }

    // Validate size
    if let Some(size) = body.size_bytes {
        let max: i64 = 1024 * 1024 * 1024; // 1GB
        if size > max {
            return Err(AppError::BadRequest(
                "File exceeds 1GB limit.".into(),
            ));
        }
    }

    let video_id = Uuid::new_v4();
    let r2_key = format!("videos/{video_id}");

    // Insert pending record into Postgres
    sqlx::query(
        "INSERT into videos (id, filename, size_bytes, mime_type, r2_key, status)
        VALUES ($1, $2, $3, $4, $5, 'pending)",
    )
    .bind(video_id)
    .bind(&body.filename)
    .bind(body.size_bytes)
    .bind(&body.mime_type)
    .bind(&r2_key)
    .execute(&state.pool)
    .await?;

    // Generate presigned PUT URL (valid for 1 hour)
    let upload_url = r2::presigned_put_url(
        &state.r2_client,
        &state.config.r2_bucket,
        &r2_key,
        3600,
    )
    .await
    .map_err(AppError::R2)?;

    Ok(Json(UploadInitResponse {
        video_id,
        upload_url
    }))
}

// -- POST /api/upload/complete/:video_id --------
#[derive(Serialize)]
pub struct UploadCompleteResponse {
    pub video_id: Uuid,
    pub watch_url: String,
}

async fn upload_complete(
    State(state): State<AppState>,
    Path(video_id): Path<Uuid>,
) -> Result<Json<UploadCompleteResponse>, AppError> {
    let rows_affected = sqlx::query(
        "UPDATE videos SET status = 'ready'
         WHERE id = $1 AND status = 'pending'"
    )
    .bind(video_id)
    .execute(&state.pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound);
    }

    let watch_url = format!("{}/watch/{video_id}", state.config.public_url);

    Ok(Json(UploadCompleteResponse {
        video_id,
        watch_url,
    }))
}

// -- GET / api/video/:video_id ---------------
async fn get_video(
    State(state): State<AppState>,
    Path(video_id): Path<Uuid>,
) -> Result<Json<VideoResponse>, AppError> {
    let video = sqlx::query_as::<_, Video>(
        "SELECT * FROM videos WHERE id = $1",
    )
    .bind(video_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(video.into()))
}

// -- GET /api/video/:video_id/stream ----------------
#[derive(Serialize)]
pub struct StreamUrlResponse {
    pub stream_url: String, // presigned GET URL -> browser streams directly from R2
    pub mime_type: Option<String>,
    pub filename: String
}

async fn get_stream_url(
    State(state): State<AppState>,
    Path(video_id): Path<Uuid>,
) -> Result<Json<StreamUrlResponse>, AppError> {
    let video = sqlx::query_as::<_, Video>(
        "SELECT * FROM videos
         WHERE id = $1 AND status = 'ready'"
    )
    .bind(video_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Presigned GET URL valid for 4 hours - long enough for any streaming session
    let stream_url = r2::presigned_get_url(
        &state.r2_client,
        &state.config.r2_bucket,
        &video.r2_key,
        4 * 3600,
    )
    .await
    .map_err(AppError::R2)?;

    Ok(Json(StreamUrlResponse {
        stream_url,
        mime_type: video.mime_type,
        filename: video.filename,
    }))
}

// -- GET /api/health -------------------------
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok"}))
}