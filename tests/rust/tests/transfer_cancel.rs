// tests d'annulation d'un transfert
//
// je vérifie le comportement quand l'utilisateur annule (stop = true) :
//   - l'UI reçoit bien transfert_cancel
//   - transfert_completed n'est PAS appelé
//   - aucun fichier reçu n'est notifié comme valide
//
// je démarre le sender en arrière-plan, j'annule après un court délai, puis
// je vérifie les événements. Le test e2e nécessite le port 58200 → verrou.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use toole_core::sender::start_sender;
use toole_tests::common::{
    shared_stop, start_receiver_task, temp_dir, wait_for_log, wait_for_progress, wait_until,
    write_random_file, MockUI,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn should_annuler_proprement_et_notifier_ui() {
    let _guard = toole_tests::common::PORT_LOCK.lock().await;

    let dir = temp_dir("cancel");
    let src = dir.path().join("annulable.bin");
    // fichier assez gros pour que l'envoi ne soit pas déjà terminé quand
    // j'annule (plusieurs secondes sur loopback)
    write_random_file(&src, 256 * 1024 * 1024);
    let dest = dir.path().join("dest");
    std::fs::create_dir_all(&dest).unwrap();

    let ui = Arc::new(MockUI::new());
    let stop = shared_stop();

    let (recv_task, _decisions, _registry) =
        start_receiver_task(ui.clone(), dest.clone(), stop.clone());
    wait_for_log(&ui, "Recepteur en ecoute", Duration::from_secs(5)).await;

    let send_ui = ui.clone();
    let send_stop = stop.clone();
    let sender_task = tokio::spawn(async move {
        start_sender(
            send_ui,
            "cancel-test".into(),
            vec![src.clone()],
            "127.0.0.1:58200".parse().unwrap(),
            send_stop.clone(),
        )
        .await
    });

    // j'attends que l'envoi ait vraiment démarré (au moins un événement de
// progression) avant d'annuler, pour être sûr de couper en plein transfert
    wait_for_progress(&ui, Duration::from_secs(10)).await;
    stop.store(true, Ordering::Relaxed);

    // j'attends la notification d'annulation au lieu d'un délai fixe (robuste
    // sous charge), puis je laisse les tâches se terminer
    wait_until(
        || ui.is_cancelled("cancel-test"),
        Duration::from_secs(5),
        "la notification d'annulation",
    )
    .await;
    let _ = sender_task.await;
    let _ = recv_task.await;

    // l'UI doit voir l'annulation, jamais une complétion
    assert!(
        ui.is_cancelled("cancel-test"),
        "je dois notifier transfert_cancel"
    );
    assert!(
        !ui.is_completed("cancel-test"),
        "un transfert annulé ne doit pas être notifié completed"
    );

    // aucune réception valide ne doit être annoncée
    let state = ui.state.lock().unwrap();
    assert!(
        state.received.is_empty(),
        "aucun fichier ne doit être notifié reçu après annulation: {:?}",
        state.received
    );
}
