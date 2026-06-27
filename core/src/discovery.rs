use crate::{Peer, ToolError};
use tokio::net::UdpSocket;
use tokio::time::{interval, Duration};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};


const BROADCAST_ADDR: &str = "255.255.255.255:58199";
const BIND_ADDR: &str = "0.0.0.0:58199";
const DISCOVERY_MSG: &[u8] = b"TOOLE_DISCOVERY";
const HERE_PREFIX: &str = "TOOLE_HERE:";

pub async fn start_discovery(local_ip: String,stop: Arc<AtomicBool>,) -> Result<(), ToolError> {
    // j'attache là le socket à une addresse qui permet de d'ecouter tout le reseau
    let socket = UdpSocket::bind(BIND_ADDR).await?;
    socket.set_broadcast(true)?;

    println!("Discovery start on {}...", BIND_ADDR);

    //on initialise un buffer pour recevoir les messages
    let mut buf = vec![0u8; 1024];
    let mut ticker = interval(Duration::from_secs(3));

    while !stop.load(Ordering::Relaxed) {
        tokio::select! {
            _ = ticker.tick() => {
                // j'envoie le message de discovery sur le reseau
                if let Err(e) = socket.send_to(DISCOVERY_MSG, BROADCAST_ADDR).await {
                    eprintln!("Erreur envoi broadcast : {}", e);
                }
            }
            Ok((len, addr)) = socket.recv_from(&mut buf) => {
                // je recupere le message recu et je le converti en string
                let msg = String::from_utf8_lossy(&buf[..len]);
                if msg.as_bytes() == DISCOVERY_MSG{
                    let reponse = format!("TOOLE_HERE:{}", get_hostname());
                    if let Err(e) = socket.send_to(reponse.as_bytes(), addr).await {
                        eprintln!("Erreur envoi réponse : {}", e);
                    }
                }
                // je recupere le message recu et je le converti en string
                else if let Ok(text) = std::str::from_utf8(msg.as_bytes()){
                    // je verifie si le message commence par "TOOLE_HERE:"
                    if let Some(h) = text.strip_prefix(HERE_PREFIX){
                        if h != get_hostname() && addr.ip().to_string() != local_ip{
                            let peer = Peer{
                                hostname: h.to_string(),
                                addr: addr.ip().to_string(),
                            };
                            println!("Peer found: {} ({})", peer.hostname, peer.addr);
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
