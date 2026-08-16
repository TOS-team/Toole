// tests de la demande d'acceptation / refus d'un transfert entrant
//
// je vérifie que le récepteur présente bien la demande (transfert_incoming),
// attend la décision de l'utilisateur, et que :
//   - refuser → transfert_refused notifié des deux côtés
//   - transfert_completed N'EST PAS appelé
//   - aucun fichier n'est écrit chez le récepteur

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use toole_core::sender::start_sender;
use toole_tests::common::{
    shared_stop, start_receiver_task_ex, temp_dir, wait_for_incoming, wait_for_log,
    wait_for_pending, wait_for_refused, MockUI,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn should_refuser_et_notifier_ui_des_deux_cotes() {
    let _guard = toole_tests::common::PORT_LOCK.lock().await;

    let dir = temp_dir("refuse");
    let src = dir.path().join("refusable.bin");
    std::fs::write(&src, b"contenu a ne pas recevoir").unwrap();
    let dest = dir.path().join("dest");
    std::fs::create_dir_all(&dest).unwrap();

    let ui = Arc::new(MockUI::new());
    let stop = shared_stop();

    // récepteur SANS auto-accept : il doit attendre une vraie décision
    let (recv_task, decisions, _registry) =
        start_receiver_task_ex(ui.clone(), dest.clone(), stop.clone(), false);
    wait_for_log(&ui, "Recepteur en ecoute", Duration::from_secs(5)).await;

    let send_ui = ui.clone();
    let send_stop = stop.clone();
    let sender_task = tokio::spawn(async move {
        start_sender(
            send_ui,
            "refuse-test".into(),
            vec![src.clone()],
            "127.0.0.1:58200".parse().unwrap(),
            "test-refuse".into(),
            send_stop.clone(),
        )
        .await
    });

    // j'attends que la demande d'acceptation soit présentée au récepteur
    wait_for_incoming(&ui, Duration::from_secs(5)).await;

    // le récepteur est en attente de décision : je refuse
    wait_for_pending(&decisions, "refuse-test", Duration::from_secs(5)).await;
    assert!(
        decisions.resolve("refuse-test", false),
        "la décision doit être résolue"
    );

    let _ = sender_task.await;
    stop.store(true, Ordering::Relaxed);
    let _ = recv_task.await;

    // le refus doit être notifié (émetteur ET récepteur utilisent le même MockUI)
    wait_for_refused(&ui, "refuse-test", Duration::from_secs(5)).await;
    assert!(
        ui.is_refused("refuse-test"),
        "un transfert refusé doit notifier transfert_refused"
    );
    assert!(
        !ui.is_completed("refuse-test"),
        "un transfert refusé ne doit pas être notifié completed"
    );

    // aucune réception valide ni fichier écrit chez le récepteur
    let state = ui.state.lock().unwrap();
    assert!(
        state.received.is_empty(),
        "aucun fichier ne doit être notifié reçu: {:?}",
        state.received
    );
    drop(state);
    assert!(
        !dest.join("refusable.bin").exists(),
        "le fichier refusé ne doit pas exister dans le dossier de destination"
    );
}