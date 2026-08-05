use crate::transfer::{handle_incoming_connection, make_server_endpoint};
use crate::{ToolError, UI};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const PORT: u16 = 58200;

pub async fn start_receiver(
    ui: Arc<dyn UI>,
    dest_dir: PathBuf,
    stop: Arc<AtomicBool>,
) -> Result<(), ToolError> {
    let endpoint = make_server_endpoint().await?;
    ui.log(&format!("Recepteur en ecoute sur le port {PORT}"));

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        let incoming = tokio::select! {
            conn = endpoint.accept() => conn,
            _ = tokio::time::sleep(Duration::from_millis(500)) => continue,
        };

        let Some(connecting) = incoming else {
            break;
        };

        let dest_dir = dest_dir.clone();
        let stop = stop.clone();
        let ui = ui.clone();

        tokio::spawn(async move {
            match connecting.await {
                Ok(connection) => {
                    ui.log(&format!(
                        "Connexion entrante depuis {:?}",
                        connection.remote_address()
                    ));
                    if let Err(e) = handle_incoming_connection(connection, dest_dir, stop).await {
                        eprintln!("Erreur connexion receveur: {e}");
                    }
                }
                Err(e) => eprintln!("Handshake QUIC echoue: {e}"),
            }
        });
    }

    endpoint.close(0u32.into(), b"arret");
    Ok(())
}
