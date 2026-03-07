use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Video {
    pub id: Uuid,
    pub filename: String,
    pub size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub r2_key: String,
    pub status: String, // "pending" | "ready"
    pub created_at: DateTime<Utc>,
}

/// What we return to the client for a ready video
#[derive(Debug, Serialize)]
pub struct VideoResponse {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<Video> for VideoResponse {
    fn from(v: Video) -> Self {
        Self {
            id: v.id,
            filename: v.filename,
            mime_type: v.mime_type,
            status: v.status,
            created_at: v.created_at
        }
    }
}