use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tokio::net::UdpSocket;
use tokio::time::sleep;

use crate::protocol::envelope::Envelope;
use crate::protocol::heartbeat::HeartbeatPayload;
use crate::state::AppState;

/// How often the background scanner runs (in seconds)
const SCAN_INTERVAL_SECS: u64 = 30;
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
async fn run_scan(
    _app_handle: AppHandle,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let local_ips = get_local_ipv4s();
    if local_ips.is_empty() {
        return Ok(());
    }

    let heartbeat_bytes = match build_heartbeat_bytes(&state).await {
        Some(b) => Arc::new(b),
        None => return Ok(()),
    };

    use socket2::{Domain, Protocol, Socket, Type};
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    let _ = socket.set_send_buffer_size(4 * 1024 * 1024); // 4MB send buffer
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;
    socket.bind(&"0.0.0.0:0".parse::<SocketAddr>().unwrap().into())?;

    let std_socket: std::net::UdpSocket = socket.into();
    let udp = Arc::new(UdpSocket::from_std(std_socket)?);

    let mut tasks = Vec::new();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(16)); // Limit concurrent subnet scans to 16

    for ip in local_ips {
        let octets = ip.octets();

        // Scan the current /24 subnet and +/- 3 subnets (total 7 subnets)
        let mut thirds: Vec<u8> = Vec::new();
        thirds.push(octets[2]);

        let start = octets[2].saturating_sub(3);
        let end = octets[2].saturating_add(3);

        for t in start..=end {
            if t != octets[2] {
                thirds.push(t);
            }
        }

        // To avoid dropping packets and network congestion, we serialize the scan
        // for each interface, putting a 15ms delay between each packet.
        // 7 subnets * 254 hosts = 1778 packets. 1778 * 15ms = 26.6 seconds.
        // This perfectly fits inside the 30-second scan interval.
        let udp_clone = udp.clone();
        let bytes_clone = heartbeat_bytes.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        tasks.push(tokio::spawn(async move {
            let _permit = permit;

            for third in thirds {
                // Send directed broadcast to this /24 subnet first
                let bcast_ip = Ipv4Addr::new(octets[0], octets[1], third, 255);
                let bcast_dest = SocketAddr::from((bcast_ip, DISCOVERY_PORT));
                let _ = udp_clone.send_to(&bytes_clone, &bcast_dest).await;

                for host in 1..=254u8 {
                    let target_ip = Ipv4Addr::new(octets[0], octets[1], third, host);
                    let dest = SocketAddr::from((target_ip, DISCOVERY_PORT));

                    // Fixed 15ms delay for EVERY packet to perfectly pace the 1778 packets over ~26s
                    tokio::time::sleep(std::time::Duration::from_millis(15)).await;

                    let mut retries = 0;
                    while let Err(e) = udp_clone.send_to(&bytes_clone, &dest).await {
                        // Yield if OS buffer is full or would block
                        if e.raw_os_error() == Some(10055) || e.raw_os_error() == Some(10035) {
                            if retries >= 3 {
                                break; // Max retries exceeded
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                            retries += 1;
                        } else {
                            // For other errors like NetworkUnreachable, we break out of the while loop and continue to the next IP
                            break;
                        }
                    }
                }
            }
        }));
    }

    // Wait for all /24 subnet sweep tasks to complete
    for t in tasks {
        let _ = t.await;
    }

    Ok(())
}

/// Build the encrypted heartbeat envelope bytes for sending to peers.
async fn build_heartbeat_bytes(state: &AppState) -> Option<Vec<u8>> {
    let is_focused = *state.is_focused.read().await;
    let payload = HeartbeatPayload {
        id: state.peer_id,
        username: state.username.read().await.clone(),
        tcp_port: *state.tcp_port.read().await,
        avatar_id: *state.avatar_id.read().await,
        avatar_base64: state.avatar_base64.read().await.clone(),
        os: std::env::consts::OS.to_string(),
        app_state: Some(if is_focused { "active".to_string() } else { "background".to_string() }),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
    };

    if let Ok(envelope) = Envelope::new("heartbeat", &payload) {
        if let Ok(bytes) = envelope.to_encrypted_bytes() {
            return Some(bytes);
        }
    }
    None
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
