use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tauri::AppHandle;

use crate::state::AppState;
use crate::protocol::envelope::Envelope;
use crate::protocol::heartbeat::HeartbeatPayload;

/// Maximum number of concurrent scan tasks
const MAX_CONCURRENT_SCANS: usize = 50;
/// How often the background scanner runs (in seconds)
const SCAN_INTERVAL_SECS: u64 = 30;
/// Timeout for TCP connect probes
const TCP_TIMEOUT_MS: u64 = 500;
/// The well-known LanChat UDP port used for discovery
const DISCOVERY_PORT: u16 = 9000;

/// Start the background subnet scanner that probes adjacent subnets for other
/// LanChat instances. Runs every SCAN_INTERVAL_SECS and also responds to
/// unicast heartbeats from cross-subnet peers.
pub fn start_subnet_scanner(app_handle: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            if let Err(e) = run_scan(app_handle.clone(), state.clone()).await {
                eprintln!("Subnet scanner error: {}", e);
            }
            sleep(Duration::from_secs(SCAN_INTERVAL_SECS)).await;
        }
    });
}

/// Run a single scan cycle: detect local subnets, generate targets, probe them.
async fn run_scan(_app_handle: AppHandle, state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let local_ips = get_local_ipv4s();
    if local_ips.is_empty() {
        return Ok(());
    }

    let targets = generate_scan_targets(&local_ips);
    if targets.is_empty() {
        return Ok(());
    }

    // Build the heartbeat envelope once for this scan cycle
    let heartbeat_bytes = build_heartbeat_bytes(&state).await;
    let heartbeat_bytes = match heartbeat_bytes {
        Some(b) => b,
        None => return Ok(()),
    };

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_SCANS));
    let mut handles = Vec::new();

    for target in targets {
        let permit = semaphore.clone().acquire_owned().await;
        let bytes = heartbeat_bytes.clone();
        let state_clone = state.clone();

        handles.push(tauri::async_runtime::spawn(async move {
            // Keep the permit for the duration of this task
            let _permit = permit;
            probe_target(target, bytes, &state_clone).await;
        }));
    }

    // Wait for all probes to complete
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

/// Build the encrypted heartbeat envelope bytes for sending to peers.
async fn build_heartbeat_bytes(state: &AppState) -> Option<Vec<u8>> {
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
            return Some(bytes);
        }
    }
    None
}

/// Probe a single target IP:
/// 1. Send UDP unicast heartbeat to port DISCOVERY_PORT
/// 2. Try TCP connect to port DISCOVERY_PORT (fallback)
async fn probe_target(target_ip: Ipv4Addr, heartbeat_bytes: Vec<u8>, _state: &AppState) {
    // --- Method 1: UDP unicast heartbeat ---
    // Bind to any available port and send our heartbeat directly to the target
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
        let dest = SocketAddr::from((target_ip, DISCOVERY_PORT));
        let _ = socket.send_to(&heartbeat_bytes, &dest).await;
        // Drop the socket so the port is freed immediately
        drop(socket);
    }

    // --- Method 2: TCP connect probe (fallback when UDP is blocked) ---
    // We try to connect to port DISCOVERY_PORT with a short timeout.
    // If it succeeds, there's a host listening — but we don't get identity info
    // from a bare TCP probe. The purpose is to verify liveness; actual peer
    // discovery still relies on UDP heartbeat exchange.
    let tcp_addr = SocketAddr::from((target_ip, DISCOVERY_PORT));
    if let Ok(stream) = tokio::time::timeout(
        Duration::from_millis(TCP_TIMEOUT_MS),
        tokio::net::TcpStream::connect(tcp_addr),
    )
    .await
    {
        if let Ok(stream) = stream {
            // TCP connected — host is alive. We can't get LanChat identity this way,
            // but we note it. The real discovery happens via UDP heartbeat exchange.
            // Close the connection immediately.
            drop(stream);
        }
    }
}

/// Get all non-loopback IPv4 addresses on this machine.
fn get_local_ipv4s() -> Vec<Ipv4Addr> {
    let mut result = Vec::new();
    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        for (_name, ip) in interfaces {
            if let std::net::IpAddr::V4(ipv4) = ip {
                if !ipv4.is_loopback() {
                    result.push(ipv4);
                }
            }
        }
    }
    result
}

/// Generate candidate IP addresses on adjacent subnets to scan.
///
/// For each local IPv4 address, scan the 5 nearest /24 subnets on each side
/// (±5 on the third octet) to discover peers on neighbouring subnets.
///
/// Examples:
///   - 6.101.88.136 → also scan 6.101.83-93.1-254
///   - 192.168.1.5  → also scan 192.168.0-2.1-254 (subnet 1 is itself, skipped)
///   - 10.0.5.10    → also scan 10.0.0-4,6-10.1-254
fn generate_scan_targets(local_ips: &[Ipv4Addr]) -> Vec<Ipv4Addr> {
    let mut targets = Vec::new();
    const RANGE: u8 = 5;

    for ip in local_ips {
        let octets = ip.octets();

        // Scan ±RANGE subnets (±RANGE on the third octet)
        for offset in 1..=RANGE {
            for &third in &[
                octets[2].wrapping_sub(offset),
                octets[2].wrapping_add(offset),
            ] {
                // Generate host IPs 1..254 in that subnet
                for host in 1..=254u8 {
                    targets.push(Ipv4Addr::new(octets[0], octets[1], third, host));
                }
            }
        }
    }

    targets
}

/// Send a unicast heartbeat reply to a specific IP address.
/// This is called when the listen loop receives a heartbeat from a non-multicast
/// source, so the remote peer can discover us even across subnet boundaries.
pub async fn send_unicast_heartbeat(state: &AppState, target_ip: Ipv4Addr) {
    let bytes = match build_heartbeat_bytes(state).await {
        Some(b) => b,
        None => return,
    };

    let dest = SocketAddr::from((target_ip, DISCOVERY_PORT));
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
        let _ = socket.send_to(&bytes, &dest).await;
    }
}

/// Immediate one-shot scan, triggered by user action (e.g., "refresh" button).
/// This is non-blocking: spawns a task and returns immediately.
pub fn trigger_scan(app_handle: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_scan(app_handle, state).await {
            eprintln!("Manual subnet scan error: {}", e);
        }
    });
}
