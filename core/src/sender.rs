use crate::transfer::{collect_entries,io_err,make_client_endpoint,send_entry};
use crate::{ToolError,UI};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::path::PathBuf;


pub async fn start_sender(
    ui:Arc< &dyn UI>,
    paths: Vec<PathBuf>,
    peer_addr: SocketAddr,
    stop: Arc<AtomicBool>,
) -> Result<(), ToolError> {
    let endpoint = make_client_endpoint()?;

    // "localhost" : nom SNI attendu par le certificat auto-signe (voir SanType::DnsName
    // dans certificat()). La verification etant desactivee (SkipServerVerification),
    // ce nom n'a pas besoin de correspondre a une vraie resolution DNS.
    let connecting = endpoint.connect(peer_addr, "localhost").map_err(io_err)?;
    let connection = connecting.await?;
    ui.log(&format!("Connecte a {peer_addr}"));

    // Parcours recursif prealable : on liste toutes les entrees (fichiers et
    // dossiers vides) avant d'ouvrir les streams, pour pouvoir les envoyer
    // en parallele.
    let mut entries = Vec::new();
    for path in &paths {
        collect_entries(path, path, &mut entries).await?;
    }

    let mut handles = Vec::new();
    for (abs_path, rel_path, is_dir) in entries {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let connection = connection.clone();
        let stop = stop.clone();
        handles.push(tokio::spawn(async move {
            send_entry(connection, abs_path, rel_path, is_dir, stop).await
        }));
    }

    for handle in handles {
        if let Err(e) = handle.await.map_err(io_err)? {
            eprintln!("Erreur d'envoi: {e}");
        }
    }

    connection.close(0u32.into(), b"transfert termine");
    endpoint.wait_idle().await;
    Ok(())
}


