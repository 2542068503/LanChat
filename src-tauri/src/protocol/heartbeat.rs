use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatPayload {
    pub id: Uuid,
    pub username: String,
    pub tcp_port: u16,
    pub avatar_id: u8,
    #[serde(default)]
    pub avatar_base64: Option<String>,
    pub os: String,
    #[serde(default)]
    pub app_state: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}
