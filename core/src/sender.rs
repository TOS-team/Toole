use crate::transfer::{
    collect_entries, io_err, make_client_endpoint, send_entry, write_json_line, ACK, BatchHeader,
};
use crate::{ToolError, UI};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;

pub async fn start_sender(
    ui: Arc<dyn UI>,
    transfer_id: String,
    paths: Vec<PathBuf>,
    peer_addr: SocketAddr,
    stop: Arc<AtomicBool>,
) -> Result<(), ToolError> {
    let endpoint = make_client_endpoint()?;
    let connecting = endpoint.connect(peer_addr, "localhost").map_err(io_err)?;
    let connection = connecting.await?;
    ui.log(&format!("Connecte a {peer_addr}"));

    let mut entries = Vec::new();
    for path in &paths {
        collect_entries(path, path, &mut entries).await?;
    }

    let mut total_bytes: u64 = 0;
    for (abs_path, _rel_path, is_dir) in &entries {
        if !is_dir {
            total_bytes += fs::metadata(abs_path).await?.len();
        }
    }

    ui.show_progress_bar(&transfer_id);
    let bytes_sent_counter = Arc::new(AtomicU64::new(0));

    // en-tête de lot : le récepteur doit connaître le total d'avance pour
    // afficher la même progression globale que nous dès le premier fichier
    if !entries.is_empty() {
        let (mut header_send, mut header_recv) = connection.open_bi().await.map_err(io_err)?;
        let header = BatchHeader {
            transfer_id: transfer_id.clone(),
            total_bytes,
        };
        write_json_line(&mut header_send, &header).await?;
        let mut ack = [0u8; 1];
        header_recv.read_exact(&mut ack).await?;
        if ack[0] != ACK {
            return Err(io_err("en-tete de lot rejete par le receveur"));
        }
        header_send.finish()?;
    }

    // on envoie au maximum 2 fichiers en parallele pour ne pas saturer la liaison
    let semaphore = Arc::new(Semaphore::new(2));

    let mut handles = Vec::new();
    for (abs_path, rel_path, is_dir) in entries {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let connection = connection.clone();
        let stop = stop.clone();
        let ui = ui.clone();
        let transfer_id = transfer_id.clone();
        let bytes_sent_counter = bytes_sent_counter.clone();
        let permit = semaphore.clone();

        handles.push(tokio::spawn(async move {
            let _guard = permit.acquire().await;
            send_entry(
                connection,
                abs_path,
                rel_path,
                is_dir,
                stop,
                ui,
                transfer_id,
                total_bytes,
                bytes_sent_counter,
            )
            .await
        }));
    }

    let mut had_error = false;
    for handle in handles {
        if let Err(e) = handle.await.map_err(io_err)? {
            eprintln!("Erreur d'envoi: {e}");
            had_error = true;
        }
    }

    connection.close(0u32.into(), b"transfert termine");
    endpoint.wait_idle().await;

    if stop.load(Ordering::Relaxed) {
        ui.transfert_cancel(&transfer_id);
    } else if had_error {
        ui.transfert_error(&transfer_id, &io_err("un ou plusieurs fichiers ont echoue"));
    } else {
        ui.transfert_completed(&transfer_id);
    }

    Ok(())
}
