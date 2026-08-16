use crate::transfer::{
    handle_incoming_connection, make_server_endpoint, DecisionBoard,
};
use crate::{ToolError, TransferRegistry, UI};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const PORT: u16 = 58200;

pub async fn start_receiver(
    ui: Arc<dyn UI>,
    dest_dir: PathBuf,
    stop: Arc<AtomicBool>,
    decisions: Arc<DecisionBoard>,
    registry: Arc<dyn TransferRegistry>,
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
        let ui = ui.clone();
        let decisions = decisions.clone();
        let registry = registry.clone();

        tokio::spawn(async move {
            match connecting.await {
                Ok(connection) => {
                    let peer = connection.remote_address().ip().to_string();
                    ui.log(&format!(
                        "Connexion entrante depuis {:?}",
                        connection.remote_address()
                    ));

                    // l'id du transfert est donne par l'emetteur (lu dans le metadata)
                    let transfer_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
                    let total = Arc::new(AtomicU64::new(0));
                    let files = Arc::new(Mutex::new(Vec::new()));
                    let bytes = Arc::new(AtomicU64::new(0));

                    // chaque connexion a son propre drapeau d'arrêt : annuler un
                    // transfert ne doit pas arrêter le récepteur global
                    let conn_stop = Arc::new(AtomicBool::new(false));

                    let res = handle_incoming_connection(
                        connection,
                        dest_dir,
                        conn_stop,
                        files.clone(),
                        bytes.clone(),
                        ui.clone(),
                        transfer_id.clone(),
                        total.clone(),
                        decisions,
                        registry.clone(),
                    )
                    .await;

                    // je désenregistre le transfert quel que soit le résultat
                    let tid = transfer_id
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .clone()
                        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                    registry.unregister(&tid);

                    match res {
                        Err(ToolError::Cancelled) | Err(ToolError::RemoteCancel) => {
                            ui.transfert_cancel(&tid);
                        }
                        Err(ToolError::Refused) => {
                            ui.transfert_refused(&tid);
                        }
                        Err(e) => {
                            eprintln!("Erreur connexion receveur: {e}");
                            let err: ToolError = crate::transfer::io_err(format!("reception: {e}"));
                            ui.transfert_error(&tid, &err);
                        }
                        Ok(()) => {
                            let received: Vec<String> = files.lock().unwrap_or_else(|e| e.into_inner()).clone();
                            let total = bytes.load(Ordering::Relaxed);
                            ui.transfert_received(&tid, &peer, total, received);
                        }
                    }
                }
                Err(e) => eprintln!("Handshake QUIC echoue: {e}"),
            }
        });
    }

    endpoint.close(0u32.into(), b"arret");
    Ok(())
}