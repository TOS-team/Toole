use crate::transfer::{handle_incoming_connection, make_server_endpoint};
use crate::{ToolError, UI};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
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
                    let peer = connection.remote_address().ip().to_string();
                    ui.log(&format!(
                        "Connexion entrante depuis {:?}",
                        connection.remote_address()
                    ));

                    // l'id du transfert est donne par l'emetteur (lu dans le metadata)
                    let transfer_id: Arc<Mutex<Option<String>>> =
                        Arc::new(Mutex::new(None));
                    let total = Arc::new(AtomicU64::new(0));
                    let files = Arc::new(Mutex::new(Vec::new()));
                    let bytes = Arc::new(AtomicU64::new(0));

                    let res = handle_incoming_connection(
                        connection,
                        dest_dir,
                        stop,
                        files.clone(),
                        bytes.clone(),
                        ui.clone(),
                        transfer_id.clone(),
                        total.clone(),
                    )
                    .await;

                    let received: Vec<String> = files.lock().unwrap().clone();
                    let total = bytes.load(Ordering::Relaxed);
                    let tid = transfer_id
                        .lock()
                        .unwrap()
                        .clone()
                        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                    if let Err(e) = res {
                        eprintln!("Erreur connexion receveur: {e}");
                        let err: ToolError = crate::transfer::io_err(format!(
                            "reception: {e}"
                        ));
                        ui.tranfert_error(&tid, &err);
                    } else {
                        ui.transfert_received(&tid, &peer, total, received);
                    }
                }
                Err(e) => eprintln!("Handshake QUIC echoue: {e}"),
            }
        });
    }

    endpoint.close(0u32.into(), b"arret");
    Ok(())
}
