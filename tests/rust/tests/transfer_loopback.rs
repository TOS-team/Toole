// tests de transfert end-to-end en loopback (sender + receiver réels)
//
// je couvre les cas nominaux :
//   - un seul fichier
//   - plusieurs fichiers dans un même lot
//   - un dossier (y compris vide)
//
// chaque test lance un vrai récepteur sur 127.0.0.1:58200 et un vrai émetteur,
// puis vérifie que le contenu est identique et que l'UI a bien été notifiée
// (completed + received). Le verrou PORT_LOCK sérialise l'accès au port.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use toole_core::sender::start_sender;
use toole_core::recever::start_receiver;
use toole_tests::common::{
    files_equal, shared_stop, temp_dir, write_random_file, MockUI,
};

/// lance un récepteur + un émetteur sur le même port, envoie les fichiers
/// donnés, puis attend la fin des deux tâches. Je renvoie l'UI partagée pour
/// vérifier les événements.
async fn run_transfer(files: Vec<(String, Vec<u8>)>) -> Arc<MockUI> {
    let _guard = toole_tests::common::PORT_LOCK.lock().unwrap();
    let dir = temp_dir("loopback");

    // je matérialise les fichiers dans le répertoire temp
    let mut paths = Vec::new();
    for (name, data) in &files {
        let p = dir.join(name);
        std::fs::write(&p, data).unwrap();
        paths.push(p);
    }

    let dest = dir.join("dest");
    std::fs::create_dir_all(&dest).unwrap();

    let ui = Arc::new(MockUI::new());
    let stop = shared_stop();

    let recv_ui = ui.clone();
    let recv_stop = stop.clone();
    let recv_dest = dest.clone();
    let recv_task = tokio::spawn(async move {
        let _ = start_receiver(recv_ui, recv_dest, recv_stop).await;
    });
    // je laisse le récepteur se binder avant de me connecter
    tokio::time::sleep(Duration::from_millis(300)).await;

    let send_ui = ui.clone();
    let send_stop = stop.clone();
    start_sender(
        send_ui,
        "transfer-loopback".into(),
        paths,
        "127.0.0.1:58200".parse().unwrap(),
        send_stop.clone(),
    )
    .await
    .unwrap();

    // j'arrête proprement le récepteur une fois l'envoi terminé
    stop.store(true, Ordering::Relaxed);
    let _ = recv_task.await;

    // je vérifie le contenu écrit chez le récepteur
    for (name, data) in &files {
        let written = std::fs::read(dest.join(name)).unwrap();
        assert_eq!(
            &written, data,
            "le fichier '{name}' doit être identique à la source"
        );
    }

    ui
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn should_transferer_un_seul_fichier() {
    let data = write_random_file(
        &temp_dir("single").join("probe.bin"),
        2 * 1024 * 1024,
    );
    let ui = run_transfer(vec![("probe.bin".into(), data)]).await;

    assert!(ui.is_completed("transfer-loopback"), "le sender doit notifier completed");
    let state = ui.state.lock().unwrap();
    assert!(
        state.received.len() == 1,
        "le receiver doit notifier 1 fichier reçu, j'ai {:?}",
        state.received
    );
    assert_eq!(state.received[0].2, 2 * 1024 * 1024, "octets reçus = taille du fichier");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn should_transferer_plusieurs_fichiers_dans_un_lot() {
    let mut files = Vec::new();
    for i in 0..3 {
        let data = write_random_file(&temp_dir("multi").join(format!("f{i}.bin")), 1024 * 1024);
        files.push((format!("f{i}.bin"), data));
    }
    let ui = run_transfer(files).await;

    let state = ui.state.lock().unwrap();
    assert!(
        state.received.len() == 1 && state.received[0].3.len() == 3,
        "je dois recevoir 3 fichiers dans la même notification, j'ai {:?}",
        state.received
    );
    // je vérifie que les 3 noms sont présents
    let names = &state.received[0].3;
    for n in ["f0.bin", "f1.bin", "f2.bin"] {
        assert!(names.contains(&n.to_string()), "nom {n} manquant dans {:?}", names);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn should_transferer_un_dossier() {
    let _guard = toole_tests::common::PORT_LOCK.lock().unwrap();

    // j'isole le dossier à envoyer et le dossier de réception dans deux
    // répertoires distincts (le récepteur ne doit jamais écrire dans la source)
    let root = temp_dir("folder");
    let src_folder = root.join("src");
    let sub = src_folder.join("sous");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("inner.bin"), b"contenu du dossier").unwrap();
    let empty = src_folder.join("vide");
    std::fs::create_dir_all(&empty).unwrap();
    let dest = root.join("dest");
    std::fs::create_dir_all(&dest).unwrap();

    let ui = Arc::new(MockUI::new());
    let stop = shared_stop();

    let recv_ui = ui.clone();
    let recv_stop = stop.clone();
    let recv_dest = dest.clone();
    let recv_task = tokio::spawn(async move {
        let _ = start_receiver(recv_ui, recv_dest, recv_stop).await;
    });
    tokio::time::sleep(Duration::from_millis(300)).await;

    start_sender(
        ui.clone(),
        "folder-test".into(),
        vec![src_folder.clone()],
        "127.0.0.1:58200".parse().unwrap(),
        stop.clone(),
    )
    .await
    .unwrap();

    stop.store(true, Ordering::Relaxed);
    let _ = recv_task.await;

    // le contenu doit être reçu sous dest/{nom du dossier}/...
    let name = src_folder.file_name().unwrap();
    let dest_dir = dest.join(name);
    assert!(
        files_equal(&sub.join("inner.bin"), &dest_dir.join("sous/inner.bin")),
        "le contenu du dossier doit être copié à l'identique"
    );
    assert!(
        dest_dir.join("vide").is_dir(),
        "le dossier vide doit aussi être recréé chez le récepteur"
    );
    assert!(ui.is_completed("folder-test"), "le sender doit notifier completed");
}
