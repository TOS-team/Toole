// tests de la découverte réseau (discovery.rs du core)
//
// je couvre :
//   - broadcast_targets : la déduction des adresses broadcast par interface
//   - la découverte e2e : je simule un pair distant en UDP loopback et je
//     vérifie que le core le détecte via peer_found
//
// attention : le test e2e bind le port 58199 (découverte), il ne doit pas
// tourner en parallèle d'autres tests qui utilisent ce port

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use toole_core::discovery::{broadcast_targets, start_discovery};
use toole_tests::common::{is_ci, shared_stop, wait_for_log, wait_for_peer, MockUI};

#[test]
fn should_collect_au_moins_le_broadcast_universel() {
    // je vérifie que broadcast_targets renvoie toujours le broadcast global
    // 255.255.255.255 en dernier recours, même sans interface réseau montée
    let targets = broadcast_targets();
    assert!(
        targets
            .iter()
            .any(|t| t.ip().is_ipv4()
                && t.ip() == std::net::IpAddr::from(std::net::Ipv4Addr::BROADCAST)),
        "je dois trouver 255.255.255.255 parmi les cibles: {targets:?}"
    );
    // toutes les cibles doivent pointer sur le port de découverte
    for t in &targets {
        assert_eq!(t.port(), 58199, "chaque cible doit viser le port 58199");
    }
    // pas de doublon (la fonction déduit les broadcast puis déduplique)
    let mut seen = Vec::new();
    for t in &targets {
        assert!(!seen.contains(t), "cible en double: {t}");
        seen.push(*t);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_detecter_un_pair_emettant_toole_here() {
    if is_ci() {
        return; // la découverte UDP peut être indisponible sur la CI
    }

    // je bloque le port 58199 pour ce test uniquement
    let _guard = toole_tests::common::PORT_LOCK.lock().await;

    // UI factice qui mémorise les pairs trouvés
    let ui = Arc::new(MockUI::new());
    let stop = shared_stop();

    // je passe une IP factice comme "moi" pour que le filtre anti-boucle
    // (addr.ip() == me) ne rejette pas le paquet que je vais envoyer depuis
    // 127.0.0.1
    let discovery_ui = ui.clone();
    let discovery_stop = stop.clone();
    let task = tokio::spawn(async move {
        let _ = start_discovery("10.0.0.1".to_string(), discovery_stop, discovery_ui).await;
    });

    // j'attends que la découverte ait bindé sa socket avant de simuler un pair
    wait_for_log(&ui, "Decouverte demarree", Duration::from_secs(5)).await;

    // je simule un pair en envoyant TOOLE_HERE:{id} sur la socket de découverte
    let sock = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    sock.send_to(b"TOOLE_HERE:pc-test-ABC12", "127.0.0.1:58199")
        .await
        .unwrap();

    // j'attends que le core traite le paquet (pas de délai fixe)
    wait_for_peer(&ui, Duration::from_secs(5)).await;

    stop.store(true, Ordering::Relaxed);
    let _ = task.await;

    let state = ui.state.lock().unwrap();
    // je vérifie que le pair simulé est bien présent. Je ne peux pas exiger
    // `len == 1` : de vrais appareils Toolé sur le LAN répondent au même
    // broadcast, et ce n'est pas ce que ce test veut valider
    assert!(
        state.peers_found.iter().any(|p| p.id == "pc-test-ABC12"),
        "je dois détecter le pair simulé: {:?}",
        state.peers_found
    );
    let peer = state
        .peers_found
        .iter()
        .find(|p| p.id == "pc-test-ABC12")
        .unwrap();
    assert_eq!(peer.addr, "127.0.0.1");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_redemarrer_sans_race_sur_le_port() {
    if is_ci() {
        return; // la découverte UDP peut être indisponible sur la CI
    }

    let _guard = toole_tests::common::PORT_LOCK.lock().await;

    let ui = Arc::new(MockUI::new());

    // première instance de la découverte, puis arrêt immédiat
    let stop = shared_stop();
    let discovery_ui = ui.clone();
    let discovery_stop = stop.clone();
    let mut task = tokio::spawn(async move {
        let _ = start_discovery("10.0.0.1".to_string(), discovery_stop, discovery_ui).await;
    });
    wait_for_log(&ui, "Decouverte demarree", Duration::from_secs(5)).await;
    stop.store(true, Ordering::Relaxed);
    let _ = task.await;

    // je relance immédiatement, comme le bouton Rafraîchir : la socket doit
    // avoir été libérée, sinon le bind échoue (port déjà utilisé) et le pair
    // simulé ci-dessous ne sera jamais détecté
    let stop2 = shared_stop();
    let discovery_ui2 = ui.clone();
    let discovery_stop2 = stop2.clone();
    task = tokio::spawn(async move {
        let _ = start_discovery("10.0.0.1".to_string(), discovery_stop2, discovery_ui2).await;
    });
    tokio::time::sleep(Duration::from_millis(300)).await;

    // le redémarrage doit redétecter un pair simulé
    let sock = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    sock.send_to(b"TOOLE_HERE:pc-restart-ZZ99", "127.0.0.1:58199")
        .await
        .unwrap();
    wait_for_peer(&ui, Duration::from_secs(5)).await;

    stop2.store(true, Ordering::Relaxed);
    let _ = task.await;

    let state = ui.state.lock().unwrap();
    assert!(
        state.peers_found.iter().any(|p| p.id == "pc-restart-ZZ99"),
        "je dois redétecter le pair après le redémarrage: {:?}",
        state.peers_found
    );
}
