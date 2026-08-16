// tests d'intégrité du transfert
//
// je vérifie que le pipeline envoie exactement les octets annoncés, sans
// perte ni corruption, sur un fichier plus gros (64 Mo) pour passer plusieurs
// chunks de 1 Mo. Je vérifie aussi que le throttle 50ms réduit bien le nombre
// d'événements de progression (sinon le webview serait noyé).

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use toole_core::sender::start_sender;
use toole_tests::common::{
    files_equal, shared_stop, start_receiver_task, temp_dir, wait_for_log, write_random_file,
    MockUI,
};

const SIZE: usize = 64 * 1024 * 1024; // 64 Mo = 64 chunks de 1 Mo

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn should_transferer_sans_perte_ni_corruption() {
    let _guard = toole_tests::common::PORT_LOCK.lock().await;

    let dir = temp_dir("integrity");
    let src = dir.path().join("gros.bin");
    let _original = write_random_file(&src, SIZE);
    let dest = dir.path().join("dest");
    std::fs::create_dir_all(&dest).unwrap();

    let ui = Arc::new(MockUI::new());
    let stop = shared_stop();

    let (recv_task, _decisions, _registry) =
        start_receiver_task(ui.clone(), dest.clone(), stop.clone());
    wait_for_log(&ui, "Recepteur en ecoute", Duration::from_secs(5)).await;

    let t0 = std::time::Instant::now();
    start_sender(
        ui.clone(),
        "integrity".into(),
        vec![src.clone()],
        "127.0.0.1:58200".parse().unwrap(),
        "test-integrity".into(),
        stop.clone(),
    )
    .await
    .unwrap();
    let transfer_duration = t0.elapsed();

    stop.store(true, Ordering::Relaxed);
    let _ = recv_task.await;

    // intégrité : fichier identique octet à octet, et taille exacte
    assert!(
        files_equal(&src, &dest.join("gros.bin")),
        "le fichier reçu doit être strictement identique à la source"
    );
    assert_eq!(
        std::fs::metadata(dest.join("gros.bin")).unwrap().len() as usize,
        SIZE,
        "la taille reçue doit être exactement {SIZE} octets"
    );

    // l'UI doit avoir été notifiée du succès des deux côtés
    assert!(
        ui.is_completed("integrity"),
        "le sender doit notifier completed"
    );
    let state = ui.state.lock().unwrap();
    assert_eq!(
        state.received.len(),
        1,
        "le receiver doit notifier 1 réception"
    );
    assert_eq!(
        state.received[0].2, SIZE as u64,
        "octets reçus annoncés = taille"
    );
    assert!(
        state.errors.is_empty(),
        "aucune erreur attendue, j'ai {:?}",
        state.errors
    );
    drop(state);

    // le throttle : sans throttle on aurait ~64 (sender) + ~64 (receiver)
    // événements pour 64 chunks. Avec le throttle (1 événement / 50 ms / côté),
    // le total dépend de la durée du transfert : je borne par cette durée
    // mesurée plutôt que par un seuil fixe, pour ne pas devenir flaky sur une
    // machine lente (le throttle reste bien le seul facteur limitant)
    let max_expected = (transfer_duration.as_millis() as usize / 50) * 2 + 8;
    let events = ui.progress_events();
    assert!(
        events <= max_expected.max(16),
        "le throttle doit limiter les événements de progression: {events} pour une durée de {transfer_duration:?} (max attendu {max_expected})"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn should_emettre_des_progressions_par_fichier() {
    let _guard = toole_tests::common::PORT_LOCK.lock().await;

    let dir = temp_dir("fileprog");
    let src = dir.path().join("avec_progres.bin");
    let _original = write_random_file(&src, 2 * 1024 * 1024);
    let dest = dir.path().join("dest");
    std::fs::create_dir_all(&dest).unwrap();

    let ui = Arc::new(MockUI::new());
    let stop = shared_stop();

    let (recv_task, _decisions, _registry) =
        start_receiver_task(ui.clone(), dest.clone(), stop.clone());
    wait_for_log(&ui, "Recepteur en ecoute", Duration::from_secs(5)).await;

    start_sender(
        ui.clone(),
        "fileprog".into(),
        vec![src.clone()],
        "127.0.0.1:58200".parse().unwrap(),
        "test-integrity".into(),
        stop.clone(),
    )
    .await
    .unwrap();

    stop.store(true, Ordering::Relaxed);
    let _ = recv_task.await;

    // je vérifie que file_progress_bar a bien été appelée pour ce fichier,
    // avec un total correspondant à la taille réelle
    let state = ui.state.lock().unwrap();
    let entries = &state.file_progress;
    assert!(
        !entries.is_empty(),
        "je dois avoir des progressions par fichier"
    );
    let last = entries.last().unwrap();
    assert_eq!(last.0, "avec_progres.bin");
    assert_eq!(
        last.2,
        2 * 1024 * 1024,
        "le total par fichier doit être la taille"
    );
}
