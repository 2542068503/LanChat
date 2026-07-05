pub mod state;
pub mod protocol;
pub mod crypto;
pub mod network;

use std::sync::Arc;
use std::path::PathBuf;
use tauri::{AppHandle, State, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code};
use uuid::Uuid;
use chrono::Utc;

use crate::state::{AppState, SharedFile};
use crate::protocol::message::{ChatMessagePayload, FileInfo};
use crate::crypto::sha;
use tokio::io::AsyncReadExt;
use sha2::Digest;

#[tauri::command]
async fn get_self_info(state: State<'_, Arc<AppState>>) -> Result<serde_json::Value, String> {
    let username = state.username.read().await.clone();
    let avatar_id = *state.avatar_id.read().await;
    let avatar_base64 = state.avatar_base64.read().await.clone();
    let local_ip = state.local_ip.read().await.clone();
    let tcp_port = *state.tcp_port.read().await;
    
    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "UnknownHost".to_string());
        
    let mut interfaces_json = Vec::new();
    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        for (name, ip) in interfaces {
            interfaces_json.push(serde_json::json!({
                "name": name,
                "ip": ip.to_string()
            }));
        }
    }
    
    Ok(serde_json::json!({
        "id": state.peer_id,
        "username": username,
        "avatarId": avatar_id,
        "avatarBase64": avatar_base64,
        "localIp": local_ip,
        "tcpPort": tcp_port,
        "hostname": hostname,
        "interfaces": interfaces_json
    }))
}

#[tauri::command]
async fn update_profile(
    state: State<'_, Arc<AppState>>,
    username: String,
    avatar_id: u8,
    avatar_base64: Option<String>
) -> Result<(), String> {
    if username.trim().is_empty() {
        return Err("用户名不能为空".into());
    }
    
    let mut u_guard = state.username.write().await;
    *u_guard = username.clone();
    
    let mut a_guard = state.avatar_id.write().await;
    *a_guard = avatar_id;
    
    let mut ab_guard = state.avatar_base64.write().await;
    *ab_guard = avatar_base64.clone();
    
    // Save updated configuration to file
    let config = crate::state::ProfileConfig {
        peer_id: state.peer_id,
        username,
        avatar_id,
        avatar_base64,
    };
    let profile_file = std::env::var("LANCHAT_PROFILE")
        .unwrap_or_else(|_| "lanchat_profile.json".to_string());
    let config_path = if std::path::PathBuf::from(&profile_file).is_absolute() {
        std::path::PathBuf::from(profile_file)
    } else {
        state.config_dir.join(profile_file)
    };
    
    if let Ok(content) = serde_json::to_string_pretty(&config) {
        let _ = std::fs::write(&config_path, content);
    }
    
    // Explicitly drop write guards to prevent deadlock during broadcast_heartbeat
    drop(u_guard);
    drop(a_guard);
    drop(ab_guard);
    
    // Broadcast the profile update immediately to all online peers
    let state_inner = state.inner().clone();
    crate::network::broadcast_heartbeat(&state_inner).await;
    
    Ok(())
}

#[tauri::command]
async fn send_message(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    peer_id: String,
    content_type: String,
    content: String,
    file_info: Option<FileInfo>,
    render_latex: Option<bool>,
    quote_msg_id: Option<String>,
    quote_sender: Option<String>,
    quote_content: Option<String>
) -> Result<ChatMessagePayload, String> {
    let peer_uuid = Uuid::parse_str(&peer_id).map_err(|e| e.to_string())?;
    
    let quote_uuid = if let Some(q_id) = quote_msg_id {
        Some(Uuid::parse_str(&q_id).map_err(|e| e.to_string())?)
    } else {
        None
    };
    
    let msg = ChatMessagePayload {
        message_id: Uuid::new_v4(),
        sender_id: state.peer_id,
        content_type,
        content,
        timestamp: Utc::now().timestamp_millis(),
        file_info,
        render_latex,
        quote_msg_id: quote_uuid,
        quote_sender,
        quote_content,
    };

    let state_inner = state.inner().clone();
    network::send_chat_message(state_inner, app, peer_uuid, msg.clone())
        .await
        .map_err(|e| e.to_string())?;
        
    Ok(msg)
}

async fn validate_path(state: &AppState, path_str: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path_str);
    
    // Check for directory traversal attempts
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err("Access denied: path traversal attempt".to_string());
        }
    }
    
    // Check if the path is in the authorized config directory or cache directory
    if path.starts_with(&state.config_dir) || path.starts_with(&state.cache_dir) {
        return Ok(path);
    }
    
    // Check if the path is explicitly registered in allowed_paths
    let allowed = state.allowed_paths.read().await;
    let mut current = path.as_path();
    while let Some(parent) = current.parent() {
        if allowed.contains(current) {
            return Ok(path);
        }
        current = parent;
    }
    if allowed.contains(current) {
        return Ok(path);
    }
    
    Err("Access denied: path is not authorized".to_string())
}

#[tauri::command]
async fn share_file(
    state: State<'_, Arc<AppState>>,
    file_path: String
) -> Result<FileInfo, String> {
    let path = validate_path(&state, &file_path).await?;
    if !path.exists() {
        return Err("文件不存在".into());
    }
    
    let name = path
        .file_name()
        .ok_or("无效的文件名")?
        .to_string_lossy()
        .into_owned();
        
    let size = tokio::fs::metadata(&path)
        .await
        .map_err(|e| e.to_string())?
        .len();
        
    let path_clone = path.clone();
    let sha256 = tokio::task::spawn_blocking(move || {
        sha::compute_sha256_file(&path_clone)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;
        
    let file_id = Uuid::new_v4();
    
    let shared = SharedFile {
        file_id,
        file_path: path,
        name: name.clone(),
        size,
        sha256: sha256.clone(),
    };
    
    let mut shared_files = state.shared_files.write().await;
    shared_files.insert(file_id, shared);
    
    Ok(FileInfo {
        file_id,
        name,
        size,
        sha256,
    })
}

#[tauri::command]
async fn calculate_file_hash(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    file_path: String,
) -> Result<String, String> {
    use tauri::{Emitter, Listener};
    let path = validate_path(&state, &file_path).await?;
    if !path.exists() {
        return Err("文件不存在".into());
    }
    
    let total_bytes = tokio::fs::metadata(&path)
        .await
        .map_err(|e| e.to_string())?
        .len();
        
    let mut file = tokio::fs::File::open(&path)
        .await
        .map_err(|e| e.to_string())?;
        
    let mut hasher = sha2::Sha256::new();
    let mut buffer = vec![0u8; 8 * 1024 * 1024]; // 8MB buffer
    let mut bytes_processed = 0u64;
    let mut last_emit = std::time::Instant::now();
    
    let hash_action = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0)); // 0=run, 1=skip, 2=cancel
    
    let hash_action_skip = hash_action.clone();
    let id_skip = app.listen("skip-hash", move |_| {
        hash_action_skip.store(1, std::sync::atomic::Ordering::SeqCst);
    });

    let hash_action_cancel = hash_action.clone();
    let id_cancel = app.listen("cancel-hash", move |_| {
        hash_action_cancel.store(2, std::sync::atomic::Ordering::SeqCst);
    });
    
    loop {
        if hash_action.load(std::sync::atomic::Ordering::SeqCst) == 1 {
            app.unlisten(id_skip);
            app.unlisten(id_cancel);
            return Ok("SKIPPED".to_string());
        }
        if hash_action.load(std::sync::atomic::Ordering::SeqCst) == 2 {
            app.unlisten(id_skip);
            app.unlisten(id_cancel);
            return Err("Cancelled".to_string());
        }

        let count = file.read(&mut buffer)
            .await
            .map_err(|e| e.to_string())?;
            
        if count == 0 {
            break;
        }
        
        sha2::Digest::update(&mut hasher, &buffer[..count]);
        bytes_processed += count as u64;
        
        if last_emit.elapsed().as_millis() >= 150 {
            let _ = app.emit("hash-progress", serde_json::json!({
                "filePath": file_path.clone(),
                "bytesProcessed": bytes_processed,
                "totalBytes": total_bytes
            }));
            last_emit = std::time::Instant::now();
        }
    }
    
    app.unlisten(id_skip);
    app.unlisten(id_cancel);
    
    let hash = hex::encode(sha2::Digest::finalize(hasher));
    
    // 发送最后的 100% 进度
    let _ = app.emit("hash-progress", serde_json::json!({
        "filePath": file_path.clone(),
        "bytesProcessed": total_bytes,
        "totalBytes": total_bytes
    }));
    
    Ok(hash)
}

#[tauri::command]
async fn share_file_with_hash(
    state: State<'_, Arc<AppState>>,
    file_path: String,
    sha256: String,
) -> Result<FileInfo, String> {
    let path = validate_path(&state, &file_path).await?;
    if !path.exists() {
        return Err("文件不存在".into());
    }
    
    let name = path
        .file_name()
        .ok_or("无效的文件名")?
        .to_string_lossy()
        .into_owned();
        
    let size = tokio::fs::metadata(&path)
        .await
        .map_err(|e| e.to_string())?
        .len();
        
    let file_id = Uuid::new_v4();
    
    let shared = SharedFile {
        file_id,
        file_path: path,
        name: name.clone(),
        size,
        sha256: sha256.clone(),
    };
    
    let mut shared_files = state.shared_files.write().await;
    shared_files.insert(file_id, shared);
    
    Ok(FileInfo {
        file_id,
        name,
        size,
        sha256,
    })
}

#[tauri::command]
async fn write_text_file(
    state: State<'_, Arc<AppState>>,
    file_path: String,
    content: String
) -> Result<(), String> {
    let path = validate_path(&state, &file_path).await?;
    tokio::fs::write(&path, content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_text_file(
    state: State<'_, Arc<AppState>>,
    file_path: String
) -> Result<String, String> {
    let path = validate_path(&state, &file_path).await?;
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_as_file(
    source: String,
    dest: String
) -> Result<(), String> {
    tokio::fs::copy(&source, &dest)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn download_file(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    peer_id: String,
    file_id: String,
    file_name: String,
    file_size: u64,
    save_path: String
) -> Result<(), String> {
    let _path = validate_path(&state, &save_path).await?;
    let peer_uuid = Uuid::parse_str(&peer_id).map_err(|e| e.to_string())?;
    let file_uuid = Uuid::parse_str(&file_id).map_err(|e| e.to_string())?;
    
    let state_inner = state.inner().clone();
    network::start_download(app, state_inner, peer_uuid, file_uuid, file_name, file_size, save_path);
    
    Ok(())
}

#[tauri::command]
async fn select_save_path(
    state: State<'_, Arc<AppState>>,
    default_name: String
) -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .set_file_name(&default_name)
        .save_file()
        .await;
        
    if let Some(ref f) = file {
        let path = f.path().to_path_buf();
        state.allowed_paths.write().await.insert(path);
    }
        
    Ok(file.map(|f| f.path().to_string_lossy().into_owned()))
}

#[tauri::command]
async fn select_share_file(
    state: State<'_, Arc<AppState>>
) -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .pick_file()
        .await;
        
    if let Some(ref f) = file {
        let path = f.path().to_path_buf();
        state.allowed_paths.write().await.insert(path);
    }
        
    Ok(file.map(|f| f.path().to_string_lossy().into_owned()))
}

#[tauri::command]
async fn read_file_base64(
    state: State<'_, Arc<AppState>>,
    file_path: String
) -> Result<String, String> {
    let path = validate_path(&state, &file_path).await?;
    use base64::Engine;
    let bytes = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

pub fn start_tray_flashing(app_handle: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        // Check if already flashing
        {
            let mut flashing = state.is_flashing.lock().await;
            if *flashing {
                return; // Already flashing
            }
            *flashing = true;
        }

        let default_icon = app_handle.default_window_icon().cloned();
        
        // Load custom message icon
        let msg_icon_bytes = include_bytes!("../icons/chat_msg.png");
        let msg_icon = tauri::image::Image::from_bytes(msg_icon_bytes).ok();

        let mut show_msg_icon = true;

        loop {
            // Check if we should stop flashing
            {
                let flashing = state.is_flashing.lock().await;
                if !*flashing {
                    // Restore default icon before exiting
                    if let Some(tray) = app_handle.tray_by_id("main-tray") {
                        let _ = tray.set_icon(default_icon);
                    }
                    break;
                }
            }

            if let Some(tray) = app_handle.tray_by_id("main-tray") {
                if show_msg_icon {
                    let _ = tray.set_icon(msg_icon.clone());
                } else {
                    let _ = tray.set_icon(default_icon.clone());
                }
            }

            show_msg_icon = !show_msg_icon;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });
}

pub fn stop_tray_flashing(app_handle: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        let mut flashing = state.is_flashing.lock().await;
        *flashing = false;
        
        // Restore default icon
        if let Some(tray) = app_handle.tray_by_id("main-tray") {
            let default_icon = app_handle.default_window_icon().cloned();
            let _ = tray.set_icon(default_icon);
        }
    });
}

#[tauri::command]
async fn refresh_discovery(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let state_inner = state.inner().clone();
    crate::network::broadcast_heartbeat(&state_inner).await;
    Ok(())
}

#[tauri::command]
async fn set_peer_remark(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    peer_id: String,
    remark: Option<String>
) -> Result<(), String> {
    let peer_uuid = Uuid::parse_str(&peer_id).map_err(|e| e.to_string())?;
    let mut peers = state.online_peers.write().await;
    let found = if let Some(info) = peers.get_mut(&peer_uuid) {
        info.remark = remark;
        true
    } else {
        false
    };

    if found {
        state.save_peers(&peers).await.map_err(|e| e.to_string())?;
        crate::network::emit_peers_update(&app, &*peers);
        Ok(())
    } else {
        Err("Peer not found".into())
    }
}

#[tauri::command]
async fn delete_peer(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    peer_id: String
) -> Result<(), String> {
    let peer_uuid = Uuid::parse_str(&peer_id).map_err(|e| e.to_string())?;
    let mut peers = state.online_peers.write().await;
    if peers.remove(&peer_uuid).is_some() {
        // Also remove from connection pool if any
        let mut pool = state.connection_pool.write().await;
        pool.remove(&peer_uuid);
        
        state.save_peers(&peers).await.map_err(|e| e.to_string())?;
        crate::network::emit_peers_update(&app, &*peers);
        Ok(())
    } else {
        Err("Peer not found".into())
    }
}

#[tauri::command]
async fn send_group_message(
    state: State<'_, Arc<AppState>>,
    content_type: String,
    content: String,
    file_info: Option<crate::protocol::message::FileInfo>,
    render_latex: Option<bool>,
    quote_msg_id: Option<String>,
    quote_sender: Option<String>,
    quote_content: Option<String>
) -> Result<serde_json::Value, String> {
    let msg = serde_json::json!({
        "messageId": Uuid::new_v4(),
        "senderId": state.peer_id,
        "senderUsername": state.username.read().await.clone(),
        "contentType": content_type,
        "content": content,
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "fileInfo": file_info,
        "renderLatex": render_latex,
        "quoteMsgId": quote_msg_id,
        "quoteSender": quote_sender,
        "quoteContent": quote_content,
    });

    if let Ok(socket) = tokio::net::UdpSocket::bind("0.0.0.0:0").await {
        let dest_addr = std::net::SocketAddr::from(([239, 255, 0, 1], 9000));
        if let Ok(envelope) = crate::protocol::envelope::Envelope::new("group_chat", &msg) {
            if let Ok(bytes) = serde_json::to_vec(&envelope) {
                let _ = socket.send_to(&bytes, &dest_addr).await;
            }
        }
    }

    Ok(msg)
}

#[tauri::command]
async fn get_peers(state: State<'_, Arc<AppState>>) -> Result<Vec<serde_json::Value>, String> {
    let peers = state.online_peers.read().await;
    let list: Vec<serde_json::Value> = peers.values().map(|info| {
        serde_json::json!({
            "id": info.payload.id,
            "username": info.payload.username,
            "tcpPort": info.payload.tcp_port,
            "avatarId": info.payload.avatar_id,
            "avatarBase64": null,
            "os": info.payload.os,
            "ip": info.ip,
            "isOnline": info.is_online,
            "lastSeen": info.last_seen_time,
            "remark": info.remark,
            "isPinned": info.is_pinned
        })
    }).collect();
    Ok(list)
}

#[tauri::command]
async fn toggle_peer_pin(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    peer_id: String
) -> Result<bool, String> {
    let peer_uuid = Uuid::parse_str(&peer_id).map_err(|e| e.to_string())?;
    let mut peers = state.online_peers.write().await;
    let is_pinned = if let Some(info) = peers.get_mut(&peer_uuid) {
        info.is_pinned = !info.is_pinned;
        info.is_pinned
    } else {
        return Err("Peer not found".into());
    };

    state.save_peers(&peers).await.map_err(|e| e.to_string())?;
    crate::network::emit_peers_update(&app, &*peers);
    Ok(is_pinned)
}

#[tauri::command]
async fn get_network_details(state: State<'_, Arc<AppState>>) -> Result<serde_json::Value, String> {
    let interfaces = match local_ip_address::list_afinet_netifas() {
        Ok(list) => {
            list.into_iter()
                .filter(|(_name, ip)| ip.is_ipv4())
                .map(|(name, ip)| {
                    serde_json::json!({
                        "name": name,
                        "ip": ip.to_string()
                    })
                })
                .collect::<Vec<serde_json::Value>>()
        }
        Err(_) => Vec::new(),
    };

    let tcp_conn_count = state.connection_pool.read().await.len();
    let tcp_port = *state.tcp_port.read().await;
    let local_ip = state.local_ip.read().await.clone();

    Ok(serde_json::json!({
        "interfaces": interfaces,
        "tcpConnCount": tcp_conn_count,
        "tcpPort": tcp_port,
        "localIp": local_ip,
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app, shortcut, event| {
            if shortcut.clone().into_string() == "ctrl+shift+alt+p" && event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }).build())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            
            // Resolve AppData configuration directory and ensure it exists
            let config_dir = app_handle.path().app_config_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
            let _ = std::fs::create_dir_all(&config_dir);
            let cache_dir = app_handle.path().app_cache_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
            let _ = std::fs::create_dir_all(&cache_dir);
            
            let state = Arc::new(AppState::new(config_dir, cache_dir));
            app.manage(state.clone());

            // Start UDP discovery and TCP server
            network::start_udp_discovery(app_handle.clone(), state.clone());
            network::start_tcp_server(app_handle.clone(), state.clone());

            // Build System Tray Menu
            let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let show_i = tauri::menu::MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&show_i, &quit_i])?;

            let state_tray = state.clone();
            
            // Build Tray Icon with safe icon loading
            let icon = app.default_window_icon().cloned();
            let mut tray_builder = tauri::tray::TrayIconBuilder::with_id("main-tray")
                .menu(&menu);
                
            if let Some(i) = icon {
                tray_builder = tray_builder.icon(i);
            }

            let _tray = tray_builder
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            let state_goodbye = state_tray.clone();
                            tauri::async_runtime::block_on(async move {
                                network::send_goodbye(&state_goodbye).await;
                            });
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Down,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let app_handle = app.handle().clone();
            
            // Check if multiple instances are allowed (via Debug assertions/Dev mode, CLI, Env, or Config JSON)
            let mut allow_multiple = cfg!(debug_assertions);

            // 1. Check command line arguments
            if !allow_multiple && std::env::args().any(|arg| arg == "--multi" || arg == "-m" || arg == "--multiple") {
                allow_multiple = true;
            }

            // 2. Check environment variable
            if !allow_multiple && std::env::var("LANCHAT_MULTI").map(|val| val == "1" || val.to_lowercase() == "true").unwrap_or(false) {
                allow_multiple = true;
            }

            // 3. Check configuration file (lanchat_profile.json)
            if !allow_multiple {
                let check_config_dir = app_handle.path().app_config_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
                let profile_file = std::env::var("LANCHAT_PROFILE")
                    .unwrap_or_else(|_| "lanchat_profile.json".to_string());
                let config_path = if std::path::PathBuf::from(&profile_file).is_absolute() {
                    std::path::PathBuf::from(profile_file)
                } else {
                    check_config_dir.join(profile_file)
                };
                if config_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&config_path) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(multi) = json.get("allow_multiple").and_then(|v| v.as_bool()) {
                                allow_multiple = multi;
                            }
                        }
                    }
                }
            }

            if !allow_multiple {
                // 单实例防多开机制 (绑定环回端口 127.0.0.1:37284)
                match std::net::TcpListener::bind("127.0.0.1:37284") {
                    Ok(listener) => {
                        // 第一个实例启动：启动监听
                        tauri::async_runtime::spawn(async move {
                            if let Ok(tokio_listener) = tokio::net::TcpListener::from_std(listener) {
                                while let Ok((_stream, _)) = tokio_listener.accept().await {
                                    // 收到其它尝试多开的实例发送的连接，唤醒主窗口
                                    if let Some(window) = app_handle.get_webview_window("main") {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                        });
                    }
                    Err(_) => {
                        // 已有实例在运行：向其发送唤醒信号，随后强制退出
                        if let Ok(mut stream) = std::net::TcpStream::connect("127.0.0.1:37284") {
                            use std::io::Write;
                            let _ = stream.write_all(b"WAKE");
                        }
                        std::process::exit(0);
                    }
                }
            }

            // 注册全局快捷键 Ctrl+Shift+Alt+P 唤醒主窗口
            let shortcut = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::ALT),
                Code::KeyP
            );
            let _ = app.global_shortcut().register(shortcut);

            Ok(())
        })
        .on_window_event(move |window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                tauri::WindowEvent::Focused(true) => {
                    let state_ref = window.state::<Arc<AppState>>();
                    stop_tray_flashing(window.app_handle().clone(), state_ref.inner().clone());
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_self_info,
            update_profile,
            send_message,
            share_file,
            download_file,
            select_save_path,
            select_share_file,
            read_file_base64,
            refresh_discovery,
            set_peer_remark,
            delete_peer,
            send_group_message,
            get_peers,
            toggle_peer_pin,
            get_network_details,
            calculate_file_hash,
            share_file_with_hash,
            write_text_file,
            read_text_file,
            save_as_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
