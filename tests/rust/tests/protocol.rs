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
    assert!(
        json.starts_with(b"{"),
        "le metadata doit être un objet JSON"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_lire_une_ligne_json_sur_un_flux_quic() {
    // roundtrip réel : j'ouvre une connexion QUIC loopback, j'écris une ligne
    // JSON + \n sur un stream, puis je lis via read_json_line
    let _guard = toole_tests::common::PORT_LOCK.lock().await;

    // j'attends que l'endpoint serveur soit bindé avant de m'y connecter
    // (pas de délai fixe : sous charge, la liaison UDP peut prendre du temps)
    let server_ep = make_server_endpoint().await.unwrap();

    // endpoint serveur en arrière-plan : il lit la ligne JSON reçue avec
    // read_json_line (comme le vrai récepteur), vérifie son contenu, puis
    // répond un octet d'acquittement. Il attend ensuite la confirmation du
    // client : sans ce second jalon, fermer la connexion en plein vol ferait
    // perdre l'acquittement (ConnectionLost côté client).
    let server = tokio::spawn(async move {
        let connecting = server_ep.accept().await.unwrap().await.unwrap();
        let (mut send, mut recv) = connecting.accept_bi().await.unwrap();
        let meta: Metadata = read_json_line(&mut recv).await.unwrap();
        assert_eq!(meta.transfer_id, "line-test");
        assert_eq!(meta.rel_path, "fichier.txt");
        assert_eq!(meta.size, 10);
        send.write_all(&[0x01]).await.unwrap();
        send.finish().unwrap();
        let mut done = [0u8; 1];
        recv.read_exact(&mut done).await.unwrap();
        connecting.close(0u32.into(), b"test termine");
    });

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
    // je ne finis pas encore le send : le serveur attend une confirmation
    // après son acquittement, sur le même flux

    // j'attends l'acquittement du serveur (preuve que la ligne a bien été lue)
    let mut ack = [0u8; 1];
    recv.read_exact(&mut ack).await.unwrap();
    assert_eq!(
        ack[0], 0x01,
        "le serveur doit accuser réception de la ligne"
    );

    // je confirme au serveur que l'acquittement a bien été reçu avant qu'il
    // ne ferme la connexion (sinon l'octet peut être perdu à la fermeture)
    send.write_all(&[0x02]).await.unwrap();
    send.finish().unwrap();
    let _ = server.await;
    ep.wait_idle().await;
}
