use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileRequestPayload {
    pub file_id: Uuid,
    pub start_offset: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileResponsePayload {
    pub status: String, // "ok" | "error"
    pub start_offset: u64,
    pub file_size: u64,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileChunkPayload {
    pub file_id: Uuid,
    pub index: u64,
    pub offset: u64,
    pub size: u32,
    pub chunk_sha256: String,
}
