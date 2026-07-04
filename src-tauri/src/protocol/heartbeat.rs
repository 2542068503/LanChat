use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatPayload {
    pub id: Uuid,
    pub username: String,
    pub tcp_port: u16,
    pub avatar_id: u8,
    pub os: String,
}
