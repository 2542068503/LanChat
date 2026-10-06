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
    #[serde(default)]
    pub file_info: Option<FileInfo>,
    #[serde(default)]
    pub render_latex: Option<bool>,

    // Quote (Reply) support
    #[serde(default)]
    pub quote_msg_id: Option<Uuid>,
    #[serde(default)]
    pub quote_sender: Option<String>,
    #[serde(default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_payload_serialization() {
        let msg_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();
        let payload = ChatMessagePayload {
            message_id: msg_id,
            sender_id,
            content_type: "text".to_string(),
            content: "Hello peer!".to_string(),
            timestamp: 1718000000,
            file_info: None,
            render_latex: Some(true),
            quote_msg_id: None,
            quote_sender: None,
            quote_content: None,
        };

        let json = serde_json::to_string(&payload).expect("serialize payload");
        assert!(json.contains("\"contentType\":\"text\""));
        assert!(json.contains("\"renderLatex\":true"));

        let deserialized: ChatMessagePayload =
            serde_json::from_str(&json).expect("deserialize payload");
        assert_eq!(deserialized.message_id, msg_id);
        assert_eq!(deserialized.content, "Hello peer!");
    }

    #[test]
    fn test_file_info_serialization() {
        let file_id = Uuid::new_v4();
        let file_info = FileInfo {
            file_id,
            name: "document.pdf".to_string(),
            size: 1048576,
            sha256: "abcdef1234567890".to_string(),
        };

        let json = serde_json::to_string(&file_info).expect("serialize file_info");
        let deserialized: FileInfo = serde_json::from_str(&json).expect("deserialize file_info");
        assert_eq!(deserialized.file_id, file_id);
        assert_eq!(deserialized.size, 1048576);
    }
}
