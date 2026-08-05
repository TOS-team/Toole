use crate::{Peer, ToolError, UI};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;

const DISCOVERY_PORT: u16 = 58199;
const BROADCAST_INTERVAL: Duration = Duration::from_secs(3);
const PEER_TIMEOUT: Duration = Duration::from_secs(9);

pub async fn start_discovery(
    local_ip: String,
    stop: Arc<AtomicBool>,
    ui: Arc<dyn UI>,
) -> Result<(), ToolError> {
    let socket = Arc::new(UdpSocket::bind(format!("0.0.0.0:{DISCOVERY_PORT}")).await?);
    socket.set_broadcast(true)?;

    let hostname = crate::utils::current_hostname();
    let mut last_seen: HashMap<String, Instant> = HashMap::new();
    let mut interval = tokio::time::interval(BROADCAST_INTERVAL);
    let mut buf = [0u8; 1024];

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        let now = Instant::now();
        let expired: Vec<String> = last_seen
            .iter()
            .filter(|(_, t)| now.duration_since(**t) > PEER_TIMEOUT)
            .map(|(h, _)| h.clone())
            .collect();
        for h in expired {
            last_seen.remove(&h);
            ui.peer_lost(&h);
        }

        tokio::select! {
            _ = interval.tick() => {
                let msg = b"TOOLE_DISCOVERY";
                let addr = format!("255.255.255.255:{DISCOVERY_PORT}");
                let _ = socket.send_to(msg, &addr).await;
            }
            result = socket.recv_from(&mut buf) => {
                if let Ok((len, addr)) = result {
                    let msg = String::from_utf8_lossy(&buf[..len]);

                    if msg == "TOOLE_DISCOVERY" {
                        if addr.ip().to_string() != local_ip {
                            let response = format!("TOOLE_HERE:{}", hostname);
                            let _ = socket.send_to(response.as_bytes(), addr).await;
                        }
                    } else if let Some(h) = msg.strip_prefix("TOOLE_HERE:") {
                        if h != hostname && addr.ip().to_string() != local_ip {
                            let peer = Peer {
                                hostname: h.to_string(),
                                addr: addr.ip().to_string(),
                            };
                            last_seen.insert(h.to_string(), Instant::now());
                            ui.peer_found(&peer);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
