use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;
use local_ip_address::local_ip;

use crate::state::AppState;
use crate::protocol::envelope::Envelope;
use crate::protocol::message::ChatMessagePayload;
use crate::network::framing;
use crate::network::file::handle_file_request;

fn handle_incoming_message(app_handle: &AppHandle, state: &Arc<AppState>, chat_payload: ChatMessagePayload) {
    let _ = app_handle.emit("message-received", chat_payload.clone());
    
    // Check if main window is focused. If not, trigger tray flashing and taskbar warning.
    if let Some(window) = app_handle.get_webview_window("main") {
        if let Ok(focused) = window.is_focused() {
            if !focused {
                crate::start_tray_flashing(app_handle.clone(), state.clone());
                // Flash taskbar icon (Request user attention)
                let _ = window.request_user_attention(Some(tauri::UserAttentionType::Critical));
            }
        }
    }
}

pub fn start_tcp_server(app_handle: AppHandle, state: Arc<AppState>) {
    let state_clone = state.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = tcp_server_loop(app_handle, state_clone).await {
            eprintln!("TCP server error: {}", e);
        }
    });
}

async fn tcp_server_loop(app_handle: AppHandle, state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Bind to random port
    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let local_addr = listener.local_addr()?;
    let assigned_port = local_addr.port();
    
    // Save port and local IP to state
    {
        let mut port_guard = state.tcp_port.write().await;
        *port_guard = assigned_port;
        
        if let Ok(ip) = local_ip() {
            let mut ip_guard = state.local_ip.write().await;
            *ip_guard = ip.to_string();
        }
    }
    
    println!("TCP Server listening on port {}", assigned_port);

    loop {
        let (stream, src_addr) = listener.accept().await?;
        let app_handle_clone = app_handle.clone();
        let state_clone = state.clone();
        
        tauri::async_runtime::spawn(async move {
            if let Err(e) = handle_new_connection(stream, src_addr, app_handle_clone, state_clone).await {
                eprintln!("Error handling connection from {}: {}", src_addr, e);
            }
        });
    }
}

async fn register_connection(
    state: Arc<AppState>,
    peer_id: Uuid,
    write_half: tokio::net::tcp::OwnedWriteHalf,
    is_outbound: bool,
) -> Result<(Uuid, Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>), Box<dyn std::error::Error + Send + Sync>> {
    let mut pool = state.connection_pool.write().await;
    
    if let Some(existing) = pool.get(&peer_id) {
        if existing.is_outbound != is_outbound {
            // Tie-breaker: smaller peer ID is the dedicated connector.
            let we_are_connector = state.peer_id < peer_id;
            
            if we_are_connector {
                if !is_outbound {
                    return Err("Declining inbound connection due to outbound preference tie-breaker".into());
                }
            } else {
                if is_outbound {
                    return Err("Declining outbound connection due to inbound preference tie-breaker".into());
                }
            }
        }
    }
    
    let connection_id = Uuid::new_v4();
    let write_half_arc = Arc::new(Mutex::new(write_half));
    let entry = crate::state::ConnectionEntry {
        connection_id,
        write_half: write_half_arc.clone(),
        is_outbound,
    };
    pool.insert(peer_id, entry);
    
    Ok((connection_id, write_half_arc))
}

async fn handle_new_connection(
    mut stream: TcpStream,
    _src_addr: SocketAddr,
    app_handle: AppHandle,
    state: Arc<AppState>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read the first frame to determine the connection type
    let first_frame = framing::read_frame(&mut stream).await?;
    let envelope = Envelope::from_encrypted_bytes(&first_frame)?;
    
    if envelope.v != 1 {
        return Err("Unsupported protocol version".into());
    }

    if !envelope.verify() {
        return Err("Security signature verification failed".into());
    }

    if envelope.msg_type == "file_request" {
        // This is a file transfer channel, delegate to file handler
        handle_file_request(stream, envelope.payload, state, app_handle).await?;
    } else if envelope.msg_type == "chat" {
        // This is a chat connection, handle and store in pool
        let chat_payload: ChatMessagePayload = serde_json::from_value(envelope.payload)?;
        let peer_id = chat_payload.sender_id;
        
        // Emit message to frontend & handle tray flashing
        handle_incoming_message(&app_handle, &state, chat_payload.clone());
        
        // Split stream to support full-duplex communication
        let (read_half, write_half) = stream.into_split();
        
        // Register connection using the tie-breaker helper
        let (connection_id, _write_half_arc) = match register_connection(state.clone(), peer_id, write_half, false).await {
            Ok(res) => res,
            Err(e) => {
                println!("Inbound connection from {} rejected by tie-breaker: {}", peer_id, e);
                return Ok(());
            }
        };
        
        // Spawn reader task
        let app_handle_clone = app_handle.clone();
        let state_clone = state.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = chat_read_loop(read_half, peer_id, connection_id, app_handle_clone, state_clone).await {
                eprintln!("Chat read loop error for peer {}: {}", peer_id, e);
            }
        });
    }

    Ok(())
}

async fn chat_read_loop(
    mut read_half: OwnedReadHalf,
    peer_id: Uuid,
    connection_id: Uuid,
    app_handle: AppHandle,
    state: Arc<AppState>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        match framing::read_frame(&mut read_half).await {
            Ok(frame_bytes) => {
                let envelope = Envelope::from_encrypted_bytes(&frame_bytes)?;
                if envelope.v == 1 && envelope.verify() && envelope.msg_type == "chat" {
                    let chat_payload: ChatMessagePayload = serde_json::from_value(envelope.payload)?;
                    handle_incoming_message(&app_handle, &state, chat_payload);
                }
            }
            Err(e) => {
                // Connection closed or error, clean up only if connection ID matches
                let mut pool = state.connection_pool.write().await;
                if let Some(entry) = pool.get(&peer_id) {
                    if entry.connection_id == connection_id {
                        pool.remove(&peer_id);
                    }
                }
                return Err(e.into());
            }
        }
    }
}

/// Sends a chat message to a peer, using an existing connection or establishing a new one
pub async fn send_chat_message(
    state: Arc<AppState>,
    app_handle: AppHandle,
    peer_id: Uuid,
    message: ChatMessagePayload
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let envelope = Envelope::new("chat", &message)?;
    let frame_bytes = envelope.to_encrypted_bytes()?;
    
    // 1. Try to use existing connection from pool
    let existing_conn = {
        let pool = state.connection_pool.read().await;
        pool.get(&peer_id).map(|e| e.write_half.clone())
    };
    
    if let Some(write_half_arc) = existing_conn {
        let mut write_half = write_half_arc.lock().await;
        if framing::write_frame(&mut *write_half, &frame_bytes).await.is_ok() {
            return Ok(());
        }
    }
    
    // 2. If no connection or write failed, resolve IP and port to establish new connection
    let (peer_ip, peer_port) = {
        let peers = state.online_peers.read().await;
        if let Some(info) = peers.get(&peer_id) {
            if !info.is_online {
                return Err("Peer is offline".into());
            }
            (info.ip.clone(), info.payload.tcp_port)
        } else {
            return Err("Peer is offline or not found".into());
        }
    };
    
    let dest_addr: SocketAddr = format!("{}:{}", peer_ip, peer_port).parse()?;
    let stream = TcpStream::connect(dest_addr).await?;
    
    let (read_half, write_half) = stream.into_split();
    
    // Register outbound connection
    let (connection_id, write_half_arc) = register_connection(state.clone(), peer_id, write_half, true).await?;
    
    // Spawn reader task
    let app_handle_clone = app_handle.clone();
    let state_clone = state.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = chat_read_loop(read_half, peer_id, connection_id, app_handle_clone, state_clone).await {
            eprintln!("Chat read loop error for peer {}: {}", peer_id, e);
        }
    });
    
    // Write frame
    let mut write_half = write_half_arc.lock().await;
    framing::write_frame(&mut *write_half, &frame_bytes).await?;
    
    Ok(())
}
