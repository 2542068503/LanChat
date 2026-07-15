use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Envelope {
    pub v: u8,
    #[serde(default)]
    pub app_version: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub payload: serde_json::Value,
    pub token: Option<String>,
}

impl Envelope {
    pub fn new<T: Serialize>(msg_type: &str, payload: &T) -> Result<Self, serde_json::Error> {
        let payload_val = serde_json::to_value(payload)?;
        let payload_str = payload_val.to_string();
        let input = format!("{}:{}:LANCHAT_SIGNATURE_SALT_V2", msg_type, payload_str);
        let token = crate::crypto::sha::compute_sha256_bytes(input.as_bytes());
        Ok(Self {
            v: 2,
            app_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            msg_type: msg_type.to_string(),
            payload: payload_val,
            token: Some(token),
        })
    }

    pub fn new_system_old<T: Serialize>(msg_type: &str, payload: &T) -> Result<Self, serde_json::Error> {
        let payload_val = serde_json::to_value(payload)?;
        let payload_str = payload_val.to_string();
        let input = format!("{}:{}:LANCHAT_SIGNATURE_SALT_2026", msg_type, payload_str);
        let token = crate::crypto::sha::compute_sha256_bytes(input.as_bytes());
        Ok(Self {
            v: 1,
            app_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            msg_type: msg_type.to_string(),
            payload: payload_val,
            token: Some(token),
        })
    }

    pub fn verify(&self) -> bool {
        if let Some(ref t) = self.token {
            let payload_str = self.payload.to_string();
            let input = format!(
                "{}:{}:LANCHAT_SIGNATURE_SALT_V2",
                self.msg_type, payload_str
            );
            let computed = crate::crypto::sha::compute_sha256_bytes(input.as_bytes());
            computed == *t
        } else {
            false
        }
    }

    pub fn verify_old(&self) -> bool {
        if let Some(ref t) = self.token {
            let payload_str = self.payload.to_string();
            let input = format!(
                "{}:{}:LANCHAT_SIGNATURE_SALT_2026",
                self.msg_type, payload_str
            );
            let computed = crate::crypto::sha::compute_sha256_bytes(input.as_bytes());
            computed == *t
        } else {
            false
        }
    }

    pub fn to_encrypted_bytes(&self) -> Result<Vec<u8>, String> {
        let json_bytes = serde_json::to_vec(self).map_err(|e| e.to_string())?;
        crate::crypto::aes::encrypt(&json_bytes)
    }

    pub fn from_encrypted_bytes(data: &[u8]) -> Result<(Self, bool), String> {
        let json_bytes = crate::crypto::aes::decrypt(data)?;
        let env: Self = serde_json::from_slice(&json_bytes).map_err(|e| e.to_string())?;
        
        let is_v2 = env.verify();
        let is_v1 = env.verify_old();
        
        if !is_v2 && !is_v1 {
            return Err("Envelope verification failed".into());
        }
        
        Ok((env, is_v1))
    }
}
