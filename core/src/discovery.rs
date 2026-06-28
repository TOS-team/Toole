use crate::{Peer, ToolError, UI};
use std::collections::HashMap;
use tokio::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{interval, Duration, Instant};
use socket2::{Domain, Type, Socket};

const BROADCAST_ADDR: &str = "255.255.255.255:58199";
const BIND_ADDR: &str = "0.0.0.0:58199";
const DISCOVERY_MSG: &[u8] = b"TOOLE_DISCOVERY";
const HERE_PREFIX: &str = "TOOLE_HERE:";
const PEER_TTL: Duration = Duration::from_secs(9);

pub async fn start_discovery(
    local_ip: String,
    stop: Arc<AtomicBool>,
    ui: Arc<dyn UI>,
) -> Result<(), ToolError> {
    // j'utilise socket2 pour pouvoir passer SO_REUSEADDR et SO_REUSEPORT avant le bind
    // ca evite le conflit quand l'ancienne tache n'a pas encore libere le port
    let s = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    s.set_reuse_address(true)?;
    s.set_reuse_port(true)?;
    s.set_broadcast(true)?;
    let addr: std::net::SocketAddr = BIND_ADDR.parse().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;
    s.bind(&addr.into())?;
    s.set_nonblocking(true)?;
    let socket = UdpSocket::from_std(s.into())?;

    ui.log(&format!("Discovery start on {}...", BIND_ADDR));

    //on initialise un buffer pour recevoir les messages
    let mut buf = vec![0u8; 1024];
    let mut ticker = interval(Duration::from_secs(3));
    let mut peers_seen_at: HashMap<String, Instant> = HashMap::new();

    while !stop.load(Ordering::Relaxed) {
        tokio::select! {
            _ = ticker.tick() => {
                // j'envoie le message de discovery sur le reseau
                if let Err(e) = socket.send_to(DISCOVERY_MSG, BROADCAST_ADDR).await {
                    ui.log(&format!("Broadcast send error: {}", e));
                }

                // Expire les pairs silencieux.
                let now = Instant::now();
                let lost_peers: Vec<String> = peers_seen_at
                    .iter()
                    .filter_map(|(hostname, seen_at)| {
                        if now.duration_since(*seen_at) > PEER_TTL {
                            Some(hostname.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                for hostname in lost_peers {
                    peers_seen_at.remove(&hostname);
                    ui.peer_lost(&hostname);
                }
            }
            Ok((len, addr)) = socket.recv_from(&mut buf) => {
                // je recupere le message recu et je le converti en string
                let msg = String::from_utf8_lossy(&buf[..len]);
                if msg.as_bytes() == DISCOVERY_MSG{
                    let reponse = format!("TOOLE_HERE:{}", get_hostname());
                    if let Err(e) = socket.send_to(reponse.as_bytes(), addr).await {
                        ui.log(&format!("Reponse send error: {}", e));
                    }
                }
                // je recupere le message recu et je le converti en string
                else if let Ok(text) = std::str::from_utf8(msg.as_bytes()){
                    // je verifie si le message commence par "TOOLE_HERE:"
                    if let Some(h) = text.strip_prefix(HERE_PREFIX){
                        if h != get_hostname() && addr.ip().to_string() != local_ip{
                            let hostname = h.to_string();
                            let peer = Peer{
                                hostname: hostname.clone(),
                                addr: addr.ip().to_string(),
                            };
                            let is_new_peer = peers_seen_at.insert(hostname, Instant::now()).is_none();
                            if is_new_peer {
                                ui.peer_found(&peer);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn get_hostname() -> String {
    crate::utils::current_hostname()
}
