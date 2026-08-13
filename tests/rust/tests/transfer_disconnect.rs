// tests de gestion des déconnexions soudaines pendant un transfert
//
// deux scénarios :
//   - le récepteur disparaît en plein transfert (fermeture brutale) → le
//     sender doit notifier une erreur, jamais une complétion
//   - l'émetteur disparaît en plein transfert (tâche avortée) → le récepteur
//     doit notifier une erreur (jamais une réception) et supprimer le fichier
//     partiel du dossier de destination

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use toole_core::sender::start_sender;
use toole_core::transfer::{
    make_client_endpoint, make_server_endpoint, read_json_line, write_json_line, ACK, BatchHeader,
    Metadata,
};
use toole_tests::common::{
    shared_stop, start_receiver_task, temp_dir, wait_for_error, wait_for_log, write_random_file,
    MockUI,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn should_signal_erreur_quand_le_recepteur_disparait() {
    let _guard = toole_tests::common::PORT_LOCK.lock().await;

    let dir = temp_dir("disconnect");
    let src = dir.path().join("gros.bin");
    write_random_file(&src, 16 * 1024 * 1024);

    // je simule un récepteur qui accepte la demande puis se déconnecte
    // brutalement au milieu du premier fichier (code de fermeture non normal,
    // sans envoyer de refus ni d'annulation)
    let server_ep = make_server_endpoint().await.unwrap();
    let server = tokio::spawn(async move {
        let connecting = server_ep.accept().await.unwrap().await.unwrap();

        // flux en-tête : j'accepte la demande d'acceptation
        let (mut send, mut recv) = connecting.accept_bi().await.unwrap();
        let _header: BatchHeader = read_json_line(&mut recv).await.unwrap();
        send.write_all(&[ACK]).await.unwrap();
        send.finish().unwrap();

        // flux du premier fichier : je lis les métadonnées puis un morceau
        let (mut fsend, mut frecv) = connecting.accept_bi().await.unwrap();
        let _meta: Metadata = read_json_line(&mut frecv).await.unwrap();
        fsend.write_all(&[ACK]).await.unwrap();
        let mut len_buf = [0u8; 4];
        let _ = frecv.read_exact(&mut len_buf).await;
        let mut data = [0u8; 1];
        let _ = frecv.read_exact(&mut data).await;

        // déconnexion brutale du récepteur
        connecting.close(0xdead_u32.into(), b"perte du recepteur");
    });

    let ui = Arc::new(MockUI::new());
    let stop = shared_stop();
    start_sender(
        ui.clone(),
        "disconnect-test".into(),
        vec![src.clone()],
        "127.0.0.1:58200".parse().unwrap(),
        stop.clone(),
    )
    .await
    .unwrap();

    let _ = server.await;

    // le sender doit signaler une erreur, jamais une complétion ni une annulation
    assert!(
        ui.has_error("disconnect-test"),
        "une déconnexion du récepteur doit notifier une erreur"
    );
    assert!(
        !ui.is_completed("disconnect-test"),
        "un transfert interrompu ne doit pas être notifié completed"
    );
    assert!(
        !ui.is_cancelled("disconnect-test"),
        "une perte de connexion n'est pas une annulation explicite"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn should_signal_erreur_et_nettoyer_quand_lemetteur_disparait() {
    let _guard = toole_tests::common::PORT_LOCK.lock().await;

    // un vrai start_sender transfère trop vite en loopback (32 Mo en <1 s)
    // pour qu'un abort arrive en plein milieu : je simule donc l'émetteur avec
    // un client QUIC brut qui fait le handshake puis disparaît brutalement
    // (connexion et endpoint droppés sans fermeture propre)
    let dest = temp_dir("sender-drop").path().join("dest");
    std::fs::create_dir_all(&dest).unwrap();
    let total: u64 = 32 * 1024 * 1024;

    let ui = Arc::new(MockUI::new());
    let stop = shared_stop();
    let (recv_task, _decisions, _registry) =
        start_receiver_task(ui.clone(), dest.clone(), stop.clone());
    wait_for_log(&ui, "Recepteur en ecoute", Duration::from_secs(5)).await;

    let client = tokio::spawn(async move {
        let ep = make_client_endpoint().unwrap();
        let connecting = ep
            .connect("127.0.0.1:58200".parse().unwrap(), "localhost")
            .unwrap();
        let connection = connecting.await.unwrap();

        // en-tête de lot : le récepteur (auto-accept) valide la demande
        let (mut hs, mut hr) = connection.open_bi().await.unwrap();
        write_json_line(
            &mut hs,
            &BatchHeader {
                transfer_id: "drop-test".into(),
                total_bytes: total,
                sender: "client-disparu".into(),
                files: vec!["gros.bin".into()],
            },
        )
        .await
        .unwrap();
        let mut decision = [0u8; 1];
        hr.read_exact(&mut decision).await.unwrap();
        assert_eq!(decision[0], ACK);
        hs.finish().unwrap();

        // métadonnées du fichier puis ack du récepteur
        let (mut fs, mut fr) = connection.open_bi().await.unwrap();
        write_json_line(
            &mut fs,
            &Metadata {
                transfer_id: "drop-test".into(),
                rel_path: "gros.bin".into(),
                size: total,
                is_dir: false,
            },
        )
        .await
        .unwrap();
        let mut ack = [0u8; 1];
        fr.read_exact(&mut ack).await.unwrap();

        // j'envoie quelques Mo puis je disparais sans fermeture propre : le
        // récepteur ne reçoit plus rien et doit détecter la perte via le
        // timeout d'idle QUIC (~15 s)
        let chunk = vec![0xABu8; 1024 * 1024];
        for _ in 0..3 {
            fs.write_all(&(chunk.len() as u32).to_be_bytes()).await.unwrap();
            fs.write_all(&chunk).await.unwrap();
        }
        drop(connection);
        drop(ep);
    });

    // le récepteur doit signaler une erreur (jamais une réception valide)
    wait_for_error(&ui, Duration::from_secs(25)).await;
    let _ = client.await;
    let state = ui.state.lock().unwrap();
    assert!(
        state.received.is_empty(),
        "un transfert interrompu ne doit pas être notifié reçu: {:?}",
        state.received
    );
    assert!(
        !state.errors.is_empty(),
        "le récepteur doit signaler une erreur quand l'émetteur disparaît"
    );
    drop(state);

    // le fichier partiel doit avoir été supprimé du dossier de destination
    assert!(
        !dest.join("gros.bin").exists(),
        "le fichier partiel doit être supprimé après une interruption"
    );

    stop.store(true, Ordering::Relaxed);
    let _ = recv_task.await;
}