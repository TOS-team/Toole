// tests du protocole applicatif (transfer.rs du core)
//
// je couvre :
//   - la sérialisation / désérialisation de Metadata (le JSON envoyé en tête
//     de chaque fichier)
//   - read_json_line : la lecture d'une ligne JSON terminée par \n sur un flux
//     QUIC réel (roundtrip via un endpoint serveur + client en loopback)
//
// le framing des chunks (len 4 octets + données) est exercé indirectement par
// les tests e2e de transfert, qui envoient de vrais fichiers

use std::time::Duration;
use toole_core::transfer::{
    io_err, make_client_endpoint, make_server_endpoint, read_json_line, Metadata,
};

#[test]
fn should_serialiser_et_deserialiser_metadata() {
    // je vérifie que Metadata fait un aller-retour JSON sans perte, puisque
    // c'est le message qui porte transfer_id / rel_path / size / is_dir
    let original = Metadata {
        transfer_id: "abcd-1234".into(),
        rel_path: "dossier/photo.png".into(),
        size: 42_000,
        is_dir: false,
    };

    let json = serde_json::to_vec(&original).unwrap();
    let back: Metadata = serde_json::from_slice(&json).unwrap();

    assert_eq!(back.transfer_id, "abcd-1234");
    assert_eq!(back.rel_path, "dossier/photo.png");
    assert_eq!(back.size, 42_000);
    assert!(!back.is_dir);

    // je vérifie aussi la forme : un objet JSON, pas un tableau ni un scalaire
    assert!(json.starts_with(b"{"), "le metadata doit être un objet JSON");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_lire_une_ligne_json_sur_un_flux_quic() {
    // roundtrip réel : j'ouvre une connexion QUIC loopback, j'écris une ligne
    // JSON + \n sur un stream, puis je lis via read_json_line
    let _guard = toole_tests::common::PORT_LOCK.lock().unwrap();

    // endpoint serveur en arrière-plan : il lit la ligne JSON reçue avec
    // read_json_line (comme le vrai récepteur), vérifie son contenu, puis
    // répond un octet d'acquittement pour que le client sache que tout est lu
    let server = tokio::spawn(async move {
        let ep = make_server_endpoint().await.unwrap();
        let connecting = ep.accept().await.unwrap().await.unwrap();
        let (mut send, mut recv) = connecting.accept_bi().await.unwrap();
        let meta: Metadata = read_json_line(&mut recv).await.unwrap();
        assert_eq!(meta.transfer_id, "line-test");
        assert_eq!(meta.rel_path, "fichier.txt");
        assert_eq!(meta.size, 10);
        send.write_all(&[0x01]).await.unwrap();
        send.finish().unwrap();
        ep.wait_idle().await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let ep = make_client_endpoint().unwrap();
    let connecting = ep
        .connect("127.0.0.1:58200".parse().unwrap(), "localhost")
        .map_err(io_err)
        .unwrap();
    let connection = connecting.await.unwrap();
    let (mut send, mut recv) = connection.open_bi().await.unwrap();

    let meta = Metadata {
        transfer_id: "line-test".into(),
        rel_path: "fichier.txt".into(),
        size: 10,
        is_dir: false,
    };
    let mut line = serde_json::to_vec(&meta).unwrap();
    line.push(b'\n');
    send.write_all(&line).await.unwrap();
    send.finish().unwrap();

    // j'attends l'acquittement du serveur (preuve que la ligne a bien été lue)
    let mut ack = [0u8; 1];
    recv.read_exact(&mut ack).await.unwrap();
    assert_eq!(ack[0], 0x01, "le serveur doit accuser réception de la ligne");

    let _ = server.await;
    ep.wait_idle().await;
}
