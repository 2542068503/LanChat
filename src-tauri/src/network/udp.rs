use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::time::sleep;
use tauri::{AppHandle, Emitter};
use socket2::{Socket, Domain, Type, Protocol};

use crate::state::{AppState, PeerInfo};
use crate::protocol::envelope::Envelope;
use crate::protocol::heartbeat::HeartbeatPayload;
use crate::network::scanner;
use uuid::Uuid;

const MULTICAST_IP: Ipv4Addr = Ipv4Addr::new(239, 255, 0, 1);
const MULTICAST_PORT: u16 = 9000;

pub fn start_udp_discovery(app_handle: AppHandle, state: Arc<AppState>) {
    let app_handle_clone1 = app_handle.clone();
    let state_clone1 = state.clone();
    
    // Task 1: Multicast Listen Loop
    tauri::async_runtime::spawn(async move {
        if let Err(e) = listen_loop(app_handle_clone1, state_clone1).await {
            eprintln!("UDP listen loop error: {}", e);
        }
    });

    let state_clone2 = state.clone();
    // Task 2: Multicast Heartbeat Broadcast Loop
    tauri::async_runtime::spawn(async move {
        if let Err(e) = broadcast_loop(state_clone2).await {
            eprintln!("UDP broadcast loop error: {}", e);
        }
    });

    let app_handle_clone3 = app_handle.clone();
    let state_clone3 = state.clone();
    // Task 3: Peer Cleanup/Eviction Loop
    tauri::async_runtime::spawn(async move {
        cleanup_loop(app_handle_clone3, state_clone3).await;
    });
}

async fn listen_loop(app_handle: AppHandle, state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Construct Socket using socket2 to allow address/port reuse
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    
    // Enable SO_REUSEADDR
    socket.set_reuse_address(true)?;
    
    #[cfg(not(windows))]
    socket.set_reuse_port(true)?;

    // Bind to 0.0.0.0:MULTICAST_PORT
    let addr = SocketAddr::from(([0, 0, 0, 0], MULTICAST_PORT));
    socket.bind(&addr.into())?;

    let std_socket: std::net::UdpSocket = socket.into();
    let tokio_socket = UdpSocket::from_std(std_socket)?;

    // Join multicast group on all active IPv4 interfaces
    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        for (_name, ip) in interfaces {
            if let std::net::IpAddr::V4(ipv4) = ip {
                if !ipv4.is_loopback() {
                    let _ = tokio_socket.join_multicast_v4(MULTICAST_IP, ipv4);
                }
            }
        }
    }
    let _ = tokio_socket.join_multicast_v4(MULTICAST_IP, Ipv4Addr::new(0, 0, 0, 0));
    tokio_socket.set_multicast_ttl_v4(1)?;

    let mut buf = vec![0u8; 65536];
    println!("UDP Multicast listening on 239.255.0.1:9000");

    loop {
        let (len, src_addr) = tokio_socket.recv_from(&mut buf).await?;
        let data = &buf[..len];
        
        if let Ok(envelope) = Envelope::from_encrypted_bytes(data) {
            // Envelope verification is already done inside from_encrypted_bytes
            match envelope.msg_type.as_str() {
                    "heartbeat" => {
                        if let Ok(payload) = serde_json::from_value::<HeartbeatPayload>(envelope.payload) {
                            // Don't register ourselves
                            if payload.id == state.peer_id {
                                continue;
                            }
                            
                            let mut peers = state.online_peers.write().await;
                            let mut changed = false;
                            
                            let (existing_remark, existing_pinned) = if let Some(existing) = peers.get(&payload.id) {
                                if !existing.is_online
                                    || existing.payload.username != payload.username 
                                    || existing.payload.avatar_id != payload.avatar_id 
                                {
                                    changed = true;
                                }
                                (existing.remark.clone(), existing.is_pinned)
                            } else {
                                changed = true; // New peer
                                (None, false)
                            };
                            
                            // Deduplicate by IP: remove old offline peers from the same IP
                            let old_ids: Vec<Uuid> = peers.iter()
                                .filter(|(id, info)| **id != payload.id && info.ip == src_addr.ip().to_string() && !info.is_online)
                                .map(|(id, _)| *id)
                                .collect();
                            for id in old_ids {
                                peers.remove(&id);
                                changed = true;
                            }
                            
                            peers.insert(payload.id, PeerInfo {
                                payload: payload.clone(),
                                ip: src_addr.ip().to_string(),
                                last_seen: Instant::now(),
                                last_seen_time: chrono::Utc::now().timestamp_millis(),
                                is_online: true,
                                remark: existing_remark,
                                is_pinned: existing_pinned,
                            });
                            
                            let _ = state.save_peers(&*peers).await;
                            
                            if changed {
                                emit_peers_update(&app_handle, &*peers);
                            }

                            // Cross-subnet discovery: if the heartbeat came from
                            // a different subnet, send our own heartbeat back as
                            // a unicast reply so the peer can see us even though
                            // our multicast broadcast won't reach their subnet.
                            if let std::net::IpAddr::V4(src_v4) = src_addr.ip() {
                                if is_cross_subnet(src_v4) {
                                    let state_ref = state.clone();
                                    tauri::async_runtime::spawn(async move {
                                        scanner::send_unicast_heartbeat(&state_ref, src_v4).await;
                                    });
                                }
                            }
                        }
                    }
                    "goodbye" => {
                        if let Ok(peer_id) = serde_json::from_value::<uuid::Uuid>(envelope.payload) {
                            let mut peers = state.online_peers.write().await;
                            if let Some(info) = peers.get_mut(&peer_id) {
                                if info.is_online {
                                    info.is_online = false;
                                    
                                    // Remove from connection pool too
                                    let mut pool = state.connection_pool.write().await;
                                    pool.remove(&peer_id);
                                    
                                    let _ = state.save_peers(&*peers).await;
                                    emit_peers_update(&app_handle, &*peers);
                                }
                            }
                        }
                    }
                    "group_chat" => {
                        let _ = app_handle.emit("group-message-received", envelope.payload);
                    }
                    _ => {}
                }
        }
    }
}

pub async fn broadcast_heartbeat(state: &AppState) {
    let username = state.username.read().await.clone();
    let avatar_id = *state.avatar_id.read().await;
    let avatar_base64 = state.avatar_base64.read().await.clone();
    let tcp_port = *state.tcp_port.read().await;
    
    let payload = HeartbeatPayload {
        id: state.peer_id,
        username,
        tcp_port,
        avatar_id,
        avatar_base64,
        os: std::env::consts::OS.to_string(),
    };

    if let Ok(envelope) = Envelope::new("heartbeat", &payload) {
        if let Ok(bytes) = envelope.to_encrypted_bytes() {
            let dest_addr = SocketAddr::from((MULTICAST_IP, MULTICAST_PORT));
            
            // Broadcast on all active IPv4 interfaces
            if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
                for (_name, ip) in interfaces {
                    if let std::net::IpAddr::V4(ipv4) = ip {
                        if !ipv4.is_loopback() {
                            if let Ok(socket) = UdpSocket::bind(format!("{}:0", ipv4)).await {
                                let _ = socket.send_to(&bytes, &dest_addr).await;
                            }
                        }
                    }
                }
            }
            // Fallback
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                let _ = socket.send_to(&bytes, &dest_addr).await;
            }
        }
    }
}

async fn broadcast_loop(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dest_addr = SocketAddr::from((MULTICAST_IP, MULTICAST_PORT));

    loop {
        let username = state.username.read().await.clone();
        let avatar_id = *state.avatar_id.read().await;
        let avatar_base64 = state.avatar_base64.read().await.clone();
        let tcp_port = *state.tcp_port.read().await;
        
        let payload = HeartbeatPayload {
            id: state.peer_id,
            username,
            tcp_port,
            avatar_id,
            avatar_base64,
            os: std::env::consts::OS.to_string(),
        };

        if let Ok(envelope) = Envelope::new("heartbeat", &payload) {
            if let Ok(bytes) = envelope.to_encrypted_bytes() {
                // Broadcast on all active IPv4 interfaces
                if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
                    for (_name, ip) in interfaces {
                        if let std::net::IpAddr::V4(ipv4) = ip {
                            if !ipv4.is_loopback() {
                                if let Ok(socket) = UdpSocket::bind(format!("{}:0", ipv4)).await {
                                    let _ = socket.send_to(&bytes, &dest_addr).await;
                                }
                            }
                        }
                    }
                }
                // Fallback
                if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                    let _ = socket.send_to(&bytes, &dest_addr).await;
                }
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}

async fn cleanup_loop(app_handle: AppHandle, state: Arc<AppState>) {
    loop {
        sleep(Duration::from_secs(5)).await;
        
        let mut peers = state.online_peers.write().await;
        let mut changed = false;
        let now = Instant::now();
        
        for (id, info) in peers.iter_mut() {
            if info.is_online && now.duration_since(info.last_seen) > Duration::from_secs(15) {
                info.is_online = false;
                changed = true;
                
                // Clean up connection pool
                let state_ref = state.clone();
                let peer_id = *id;
                tauri::async_runtime::spawn(async move {
                    let mut pool = state_ref.connection_pool.write().await;
                    pool.remove(&peer_id);
                });
            }
        }

        if changed {
            let _ = state.save_peers(&*peers).await;
            emit_peers_update(&app_handle, &*peers);
        }
    }
}

pub async fn send_goodbye(state: &AppState) {
    if let Ok(envelope) = Envelope::new("goodbye", &state.peer_id) {
        if let Ok(bytes) = envelope.to_encrypted_bytes() {
            let dest_addr = SocketAddr::from((MULTICAST_IP, MULTICAST_PORT));
            
            if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
                for (_name, ip) in interfaces {
                    if let std::net::IpAddr::V4(ipv4) = ip {
                        if !ipv4.is_loopback() {
                            if let Ok(socket) = UdpSocket::bind(format!("{}:0", ipv4)).await {
                                let _ = socket.send_to(&bytes, &dest_addr).await;
                            }
                        }
                    }
                }
            }
            // Fallback
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                let _ = socket.send_to(&bytes, &dest_addr).await;
            }
        }
    }
}

/// Check if the given IPv4 address is on a different subnet from all local
/// IPv4 addresses. Uses /24 subnet mask (most common for LANs).
fn is_cross_subnet(remote: Ipv4Addr) -> bool {
    let remote_octets = remote.octets();
    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        for (_name, ip) in interfaces {
            if let std::net::IpAddr::V4(local) = ip {
                if local.is_loopback() {
                    continue;
                }
                let local_octets = local.octets();
                // Compare first 3 octets (/24 subnet)
                if local_octets[0] == remote_octets[0]
                    && local_octets[1] == remote_octets[1]
                    && local_octets[2] == remote_octets[2]
                {
                    return false; // Same subnet
                }
            }
        }
    }
    true // Different subnet or no interfaces found
}

pub fn emit_peers_update(app_handle: &AppHandle, peers: &HashMap<uuid::Uuid, PeerInfo>) {
    // Collect active online peers list in serialized format for frontend
    let list: Vec<serde_json::Value> = peers.values().map(|info| {
        serde_json::json!({
            "id": info.payload.id,
            "username": info.payload.username,
            "tcpPort": info.payload.tcp_port,
            "avatarId": info.payload.avatar_id,
            "avatarBase64": info.payload.avatar_base64,
            "os": info.payload.os,
            "ip": info.ip,
            "isOnline": info.is_online,
            "lastSeen": info.last_seen_time,
            "remark": info.remark,
            "isPinned": info.is_pinned
        })
    }).collect();

    let _ = app_handle.emit("peers-updated", list);
}
