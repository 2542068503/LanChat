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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_payload_roundtrip() {
        let id = Uuid::new_v4();
        let payload = HeartbeatPayload {
            id,
            username: "TestNode".to_string(),
            tcp_port: 9001,
            avatar_id: 3,
            avatar_base64: None,
            os: "windows".to_string(),
            app_state: Some("active".to_string()),
            version: Some("2.0.1".to_string()),
        };

        let json = serde_json::to_string(&payload).expect("serialize heartbeat");
        assert!(json.contains("\"tcpPort\":9001"));
        assert!(json.contains("\"username\":\"TestNode\""));

        let deserialized: HeartbeatPayload =
            serde_json::from_str(&json).expect("deserialize heartbeat");
        assert_eq!(deserialized.id, id);
        assert_eq!(deserialized.tcp_port, 9001);
        assert_eq!(deserialized.app_state, Some("active".to_string()));
    }
}
