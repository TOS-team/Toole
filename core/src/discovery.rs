use crate::{Peer, ToolError, UI};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;

const DISCOVERY_PORT: u16 = 58199;
const BROADCAST_INTERVAL: Duration = Duration::from_secs(3);
const PEER_TIMEOUT: Duration = Duration::from_secs(9);

// collecte les adresses broadcast de chaque interface réseau montée,
// puis le broadcast illimité 255.255.255.255 en secours (souvent filtré en WiFi)
// je l'expose en pub pour pouvoir la tester unitairement dans le crate tests/
pub fn broadcast_targets() -> Vec<SocketAddr> {
    let mut ips: Vec<Ipv4Addr> = Vec::new();

    if let Ok(ifaces) = if_addrs::get_if_addrs() {
        for iface in ifaces {
            if iface.is_loopback() {
                continue;
            }
            if let if_addrs::IfAddr::V4(v4) = iface.addr {
                if let Some(bcast) = v4.broadcast {
                    ips.push(bcast);
                }
                // si l'OS ne fournit pas le broadcast, on le déduit ip | ~masque
                let bcast = Ipv4Addr::from(u32::from(v4.ip) | !u32::from(v4.netmask));
                if bcast != Ipv4Addr::BROADCAST {
                    ips.push(bcast);
                }
            }
        }
    }

    ips.push(Ipv4Addr::BROADCAST);
    ips.dedup();

    ips.into_iter()
        .map(|ip| SocketAddr::new(IpAddr::V4(ip), DISCOVERY_PORT))
        .collect()
}

pub async fn start_discovery(
    local_ip: String,
    stop: Arc<AtomicBool>,
    ui: Arc<dyn UI>,
) -> Result<(), ToolError> {
    let socket = Arc::new(UdpSocket::bind(format!("0.0.0.0:{DISCOVERY_PORT}")).await?);
    socket.set_broadcast(true)?;
    ui.log(&format!("Decouverte demarree sur le port {DISCOVERY_PORT}"));

    let targets = broadcast_targets();
    let me: IpAddr = local_ip.parse().unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

    let my_id = crate::utils::device_id();
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
                for dest in &targets {
                    let _ = socket.send_to(msg, dest).await;
                }
            }
            result = socket.recv_from(&mut buf) => {
                if let Ok((len, addr)) = result {
                    if addr.ip() == me {
                        continue;
                    }
                    let msg = String::from_utf8_lossy(&buf[..len]);

                    if msg == "TOOLE_DISCOVERY" {
                        let response = format!("TOOLE_HERE:{}", my_id);
                        let _ = socket.send_to(response.as_bytes(), addr).await;
                    } else if let Some(id) = msg.strip_prefix("TOOLE_HERE:") {
                        if id != my_id {
                            let peer = Peer {
                                id: id.to_string(),
                                addr: addr.ip().to_string(),
                            };
                            last_seen.insert(id.to_string(), Instant::now());
                            ui.peer_found(&peer);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
