use crate::transfer::{
    collect_entries, io_err, make_client_endpoint, send_entry, write_json_line, ACK, BatchHeader,
    CLOSE_CANCEL, CLOSE_OK, DECISION_TIMEOUT, REFUSE,
};
use crate::{ToolError, UI};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::sync::Semaphore;

pub async fn start_sender(
    ui: Arc<dyn UI>,
    transfer_id: String,
    paths: Vec<PathBuf>,
    peer_addr: SocketAddr,
    peer_id: String,
    stop: Arc<AtomicBool>,
) -> Result<(), ToolError> {
    // je vérifie l'identité du pair au handshake : l'empreinte attendue vient
    // de l'épingle (premier contact = aucune, j'épingle après coup)
    let expected = crate::file_certif::pin_for(&peer_id);
    let endpoint = make_client_endpoint(expected.as_deref())?;
    let connecting = endpoint.connect(peer_addr, "localhost").map_err(io_err)?;
    let connection = connecting.await?;
    ui.log(&format!("Connecte a {peer_addr}"));

    // handshake réussi : au premier contact j'épingle l'empreinte du
    // certificat reçu (TOFU). Si une épingle existait mais ne correspondait
    // pas, le handshake aurait déjà échoué à l'étape ci-dessus
    if let Some(fp) = crate::file_certif::peer_fingerprint(&connection) {
        if crate::file_certif::pin_for(&peer_id) != Some(fp.clone()) {
            match crate::file_certif::save_pin(&peer_id, &fp) {
                Ok(()) => ui.log(&format!("Empreinte du pair {peer_id} epinglee ({fp})")),
                Err(e) => ui.log(&format!("epingle impossible: {e}")),
            }
        }
    }

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

    let files: Vec<String> = entries
        .iter()
        .map(|(_abs, rel, _is_dir)| rel.clone())
        .collect();

    // en-tête de lot : le récepteur doit connaître le transfer_id, le total et
    // la liste des fichiers à l'avance, pour afficher la demande d'acceptation
    // puis la même progression globale que nous dès le premier fichier
    if !entries.is_empty() {
        let (mut header_send, mut header_recv) = connection.open_bi().await.map_err(io_err)?;
        let header = BatchHeader {
            transfer_id: transfer_id.clone(),
            total_bytes,
            sender: crate::utils::device_id(),
            files: files.clone(),
        };
        write_json_line(&mut header_send, &header).await?;

        // j'attends la décision du destinataire (accepter / refuser), en
        // surveillant l'annulation utilisateur pendant l'attente
        let mut decision = [0u8; 1];
        let deadline = Instant::now() + DECISION_TIMEOUT;
        let decision_result = loop {
            if stop.load(Ordering::Relaxed) {
                connection.close(CLOSE_CANCEL.into(), b"annulation utilisateur");
                endpoint.wait_idle().await;
                ui.transfert_cancel(&transfer_id);
                return Ok(());
            }
            if Instant::now() >= deadline {
                break Err(io_err("le destinataire n'a pas repondu"));
            }
            match tokio::time::timeout(
                Duration::from_millis(100),
                header_recv.read_exact(&mut decision),
            )
            .await
            {
                Ok(Ok(())) => break Ok(()),
                Ok(Err(e)) => break Err(crate::transfer::quinn_to_err(e)),
                Err(_) => continue,
            }
        };

        match decision_result {
            Ok(()) if decision[0] == ACK => {
                header_send.finish()?;
            }
            Ok(()) if decision[0] == REFUSE => {
                connection.close(CLOSE_CANCEL.into(), b"transfert refuse");
                endpoint.wait_idle().await;
                ui.transfert_refused(&transfer_id);
                return Ok(());
            }
            Ok(()) => {
                connection.close(CLOSE_CANCEL.into(), b"reponse invalide");
                endpoint.wait_idle().await;
                ui.transfert_error(
                    &transfer_id,
                    &io_err("reponse invalide du destinataire"),
                );
                return Ok(());
            }
            Err(ToolError::RemoteCancel) => {
                // le destinataire a annulé pendant l'attente
                connection.close(CLOSE_CANCEL.into(), b"annule par le destinataire");
                endpoint.wait_idle().await;
                ui.transfert_cancel(&transfer_id);
                return Ok(());
            }
            Err(e) => {
                connection.close(CLOSE_CANCEL.into(), b"pas de reponse");
                endpoint.wait_idle().await;
                ui.transfert_error(&transfer_id, &e);
                return Ok(());
            }
        }
    }

    // une fois la décision acceptée, j'affiche la barre de progression du lot
    ui.show_progress_bar(&transfer_id);
    let bytes_sent_counter = Arc::new(AtomicU64::new(0));

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
    let mut remote_cancelled = false;
    for handle in handles {
        if let Err(e) = handle.await.map_err(io_err)? {
            eprintln!("Erreur d'envoi: {e}");
            if matches!(e, ToolError::RemoteCancel) {
                remote_cancelled = true;
            } else {
                had_error = true;
            }
        }
    }

    if stop.load(Ordering::Relaxed) || remote_cancelled {
        connection.close(CLOSE_CANCEL.into(), b"annulation");
        endpoint.wait_idle().await;
        ui.transfert_cancel(&transfer_id);
    } else if had_error {
        // je ferme avec CLOSE_OK et non CLOSE_CANCEL : le récepteur distingue
        // une annulation (CLOSE_CANCEL/reset → transfert_cancel) d'un échec
        // (autre code → transfert_error). Ici c'est bien un échec, pas une
        // annulation ; le contrôle de complétude `done < expected` côté
        // récepteur confirme la perte de données.
        connection.close(CLOSE_OK.into(), b"erreur");
        endpoint.wait_idle().await;
        ui.transfert_error(&transfer_id, &io_err("un ou plusieurs fichiers ont echoue"));
    } else {
        connection.close(CLOSE_OK.into(), b"transfert termine");
        endpoint.wait_idle().await;
        ui.transfert_completed(&transfer_id);
    }

    Ok(())
}
