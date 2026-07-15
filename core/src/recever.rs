use crate::transfert::{make_server_endpoint,handle_incoming_connection,};
use std::net::SocketAddr;


pub async fn start_receiver(
    ui: &dyn UI,
    dest_dir: PathBuf,
    stop: Arc<AtomicBool>,
) -> Result<(), ToolError> {
    let endpoint = make_server_endpoint().await?;
    ui.log(&format!("Recepteur en ecoute sur le port {PORT}"));

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        // On attend soit une connexion entrante, soit un tick de 500ms pour
        // rester reactif au signal d'arret (evite de bloquer indefiniment
        // sur endpoint.accept()).
        let incoming = tokio::select! {
            conn = endpoint.accept() => conn,
            _ = tokio::time::sleep(Duration::from_millis(500)) => continue,
        };

        let Some(connecting) = incoming else {
            break; // endpoint ferme cote local
        };

        let dest_dir = dest_dir.clone();
        let stop = stop.clone();

        tokio::spawn(async move {

          match connecting.await {

                Ok(connection) => {
                    if let Err(e) = handle_incoming_connection(connection, dest_dir, stop).await {
                        eprintln!("Erreur de connexion receveur: {e}");
                    }
                }
                Err(e) => eprintln!("Handshake QUIC echoue😭:  {e}"),
            }
        });
    }

    endpoint.close(0u32.into(), b"arret");
    Ok(())
}


