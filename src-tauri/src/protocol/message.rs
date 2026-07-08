use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessagePayload {
    pub message_id: Uuid,
    pub sender_id: Uuid,
    pub content_type: String, // "text" | "file"
    pub content: String,
    pub timestamp: i64,
    pub file_info: Option<FileInfo>,
    pub render_latex: Option<bool>,

    // Quote (Reply) support
    pub quote_msg_id: Option<Uuid>,
    pub quote_sender: Option<String>,
    pub quote_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub file_id: Uuid,
    pub name: String,
    pub size: u64,
    pub sha256: String,
}
