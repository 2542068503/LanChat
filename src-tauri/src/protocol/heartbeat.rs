use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatPayload {
    pub id: Uuid,
    pub username: String,
    pub tcp_port: u16,
    pub avatar_id: u8,
    pub avatar_base64: Option<String>,
    pub os: String,
    pub app_state: Option<String>,
}
