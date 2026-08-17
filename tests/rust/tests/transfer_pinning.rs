// tests du pinning TOFU des certificats (file_certif.rs du core)
//
// je couvre le cycle complet de la confiance à la première connexion :
//   1. premier contact : aucune empreinte attendue, le handshake passe et
//      j'épingle l'empreinte du certificat reçu
//   2. contact suivant : l'empreinte épinglée correspond, le handshake passe
//   3. certificat différent de l'épingle (attaquant de l'homme du milieu) :
//      le handshake est refusé

use toole_core::file_certif;
use toole_core::transfer::{make_client_endpoint, make_server_endpoint};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_epinguer_au_premier_contact_et_detecter_une_identite_changee() {
    let _guard = toole_tests::common::PORT_LOCK.lock().await;

    // ids distincts pour ne jamais entrer en collision avec de vraies
    // épingles (les miennes restent des artefacts de test dans pins.json)
    let peer = "pin-test-device";

    let server_ep = make_server_endpoint().await.unwrap();
    let server = tokio::spawn(async move {
        // j'accepte jusqu'à 3 tentatives (2 réussies + 1 refusée par le client)
        for _ in 0..3 {
            if let Some(connecting) = server_ep.accept().await {
                let _ = connecting.await;
            }
        }
    });

    // 1. premier contact : pas d'épingle attendue → handshake accepté
    let ep = make_client_endpoint(None).unwrap();
    let connection = ep
        .connect("127.0.0.1:58200".parse().unwrap(), "localhost")
        .unwrap()
        .await
        .expect("le premier contact doit etre accepte");
    let fp = file_certif::peer_fingerprint(&connection)
        .expect("le serveur doit presenter un certificat");
    file_certif::save_pin(peer, &fp).unwrap();
    assert_eq!(
        file_certif::pin_for(peer).as_deref(),
        Some(fp.as_str()),
        "l'empreinte du premier contact doit etre epinglee"
    );

    // 2. contact suivant : l'empreinte épinglée correspond → handshake accepté
    let ep = make_client_endpoint(Some(&fp)).unwrap();
    ep.connect("127.0.0.1:58200".parse().unwrap(), "localhost")
        .unwrap()
        .await
        .expect("un certificat correspondant a l'epingle doit etre accepte");

    // 3. certificat différent de l'épingle : attaque MITM → handshake refusé
    let ep = make_client_endpoint(Some("empreinte-inconnue")).unwrap();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ep.connect("127.0.0.1:58200".parse().unwrap(), "localhost")
            .unwrap(),
    )
    .await;
    assert!(
        matches!(result, Err(_) | Ok(Err(_))),
        "un certificat different de l'epingle doit faire echouer le handshake"
    );

    let _ = server.await;
}