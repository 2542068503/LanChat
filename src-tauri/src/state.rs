use crate::protocol::heartbeat::HeartbeatPayload;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

pub const MIN_ALLOWED_VERSION: &str = "2.0.0";

pub fn is_version_allowed(client_version: &str) -> bool {
    // Parse version like "1.5.0" into tuple (1, 5, 0)
    let parse_ver = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<&str> = v.split('.').collect();
        let major = parts.get(0).unwrap_or(&"0").parse().unwrap_or(0);
        let minor = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
        let patch = parts.get(2).unwrap_or(&"0").parse().unwrap_or(0);
        (major, minor, patch)
    };

    let client_ver = parse_ver(client_version);
    let min_ver = parse_ver(MIN_ALLOWED_VERSION);

    client_ver >= min_ver
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub payload: HeartbeatPayload,
    pub ip: String,
    pub last_seen: Instant,
    pub last_seen_time: i64,
    pub is_online: bool,
    pub remark: Option<String>,
    pub is_pinned: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerInfoSaved {
    pub payload: HeartbeatPayload,
    pub ip: String,
    pub last_seen_time: i64,
    pub remark: Option<String>,
    pub is_pinned: bool,
}

#[derive(Debug, Clone)]
pub struct SharedFile {
    pub file_id: Uuid,
    pub file_path: PathBuf,
    pub name: String,
    pub size: u64,
    pub sha256: String,
}

pub struct ConnectionEntry {
    pub connection_id: Uuid,
    pub write_half: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    pub is_outbound: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfileConfig {
    pub peer_id: Uuid,
    pub username: String,
    pub avatar_id: u8,
    pub avatar_base64: Option<String>,
}

pub struct AppState {
    pub peer_id: Uuid,
    pub username: RwLock<String>,
    pub avatar_id: RwLock<u8>,
    pub avatar_base64: RwLock<Option<String>>,
    pub tcp_port: RwLock<u16>,
    pub local_ip: RwLock<String>,
    pub online_peers: RwLock<HashMap<Uuid, PeerInfo>>,
    pub connection_pool: RwLock<HashMap<Uuid, ConnectionEntry>>,
    pub shared_files: RwLock<HashMap<Uuid, SharedFile>>,
    pub allowed_paths: RwLock<std::collections::HashSet<PathBuf>>,
    pub is_flashing: Mutex<bool>,
    pub is_focused: RwLock<bool>,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub masquerade_icon: Mutex<Option<tauri::image::Image<'static>>>,
}

impl AppState {
    pub fn new(config_dir: PathBuf, cache_dir: PathBuf) -> Self {
        let profile_file =
            std::env::var("LANCHAT_PROFILE").unwrap_or_else(|_| "lanchat_profile.json".to_string());
        let config_path = if PathBuf::from(&profile_file).is_absolute() {
            PathBuf::from(profile_file)
        } else {
            config_dir.join(profile_file)
        };

        let config = if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                serde_json::from_str::<ProfileConfig>(&content)
                    .unwrap_or_else(|_| Self::default_config())
            } else {
                Self::default_config()
            }
        } else {
            let default = Self::default_config();
            if let Ok(content) = serde_json::to_string_pretty(&default) {
                let _ = std::fs::write(&config_path, content);
            }
            default
        };

        let initial_peers = Self::load_peers(&config_dir);

        Self {
            peer_id: config.peer_id,
            username: RwLock::new(config.username),
            avatar_id: RwLock::new(config.avatar_id),
            avatar_base64: RwLock::new(config.avatar_base64),
            tcp_port: RwLock::new(0),
            local_ip: RwLock::new("127.0.0.1".to_string()),
            online_peers: RwLock::new(initial_peers),
            connection_pool: RwLock::new(HashMap::new()),
            shared_files: RwLock::new(HashMap::new()),
            allowed_paths: RwLock::new(std::collections::HashSet::new()),
            is_flashing: Mutex::new(false),
            is_focused: RwLock::new(true),
            config_dir,
            cache_dir,
            masquerade_icon: Mutex::new(None),
        }
    }

    pub async fn save_peers(&self, peers: &HashMap<Uuid, PeerInfo>) -> Result<(), String> {
        let saved_peers: Vec<PeerInfoSaved> = peers
            .values()
            .map(|info| PeerInfoSaved {
                payload: info.payload.clone(),
                ip: info.ip.clone(),
                last_seen_time: info.last_seen_time,
                remark: info.remark.clone(),
                is_pinned: info.is_pinned,
            })
            .collect();

        let peers_file =
            std::env::var("LANCHAT_PEERS").unwrap_or_else(|_| "lanchat_peers.json".to_string());
        let config_path = if PathBuf::from(&peers_file).is_absolute() {
            PathBuf::from(peers_file)
        } else {
            self.config_dir.join(peers_file)
        };

        let content = serde_json::to_string_pretty(&saved_peers)
            .map_err(|e| format!("Failed to serialize peers: {}", e))?;
        std::fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write peers file: {}", e))?;
        Ok(())
    }

    pub fn load_peers(config_dir: &PathBuf) -> HashMap<Uuid, PeerInfo> {
        let peers_file =
            std::env::var("LANCHAT_PEERS").unwrap_or_else(|_| "lanchat_peers.json".to_string());
        let config_path = if PathBuf::from(&peers_file).is_absolute() {
            PathBuf::from(peers_file)
        } else {
            config_dir.join(peers_file)
        };

        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(saved) = serde_json::from_str::<Vec<PeerInfoSaved>>(&content) {
                    let mut map = HashMap::new();
                    for item in saved {
                        map.insert(
                            item.payload.id,
                            PeerInfo {
                                payload: item.payload,
                                ip: item.ip,
                                last_seen: Instant::now(),
                                last_seen_time: item.last_seen_time,
                                is_online: false, // Default to offline on startup
                                remark: item.remark,
                                is_pinned: item.is_pinned,
                            },
                        );
                    }
                    return map;
                }
            }
        }
        HashMap::new()
    }

    fn default_config() -> ProfileConfig {
        let unique_id = Uuid::new_v4();
        ProfileConfig {
            peer_id: unique_id,
            username: format!("用户-{}", &unique_id.to_string()[..4]),
            avatar_id: 1,
            avatar_base64: None,
        }
    }
}
