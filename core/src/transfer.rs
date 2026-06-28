// Transfert de fichiers par QUIC (via Quinn).
//
// Pourquoi QUIC plutot que TCP ?
//   - Multiplexage natif : plusieurs fichiers en parallele sur une seule connexion
//   - Pas de Head-of-Line blocking
//   - Chiffrement TLS 1.3 integré (aucune config manuelle)
//   - Controle de congestion, perte de paquets, renvoi automatique
//
// Principe :
//   1. Connexion QUIC etablie entre deux pairs (client/serveur)
//   2. Chaque fichier = un bi-directional stream QUIC dedie
//   3. Les streams circulent en parallele sur la meme connexion
//   4. Les dossiers sont transmis recursivement (un stream par fichier)
//
// Protocole applicatif (par stream) :
//   1. Sender → Receiver : Metadata JSON (rel_path, size, sha256, is_dir) + \n
//   2. Receiver → Sender : Ack 0x01
//   3. Sender → Receiver : Chunk (chunk_index u32 big-endian + data)
//   4. Receiver → Sender : Ack chunk_index u32 big-endian
//   5. Repeter 3-4 jusqu'au dernier chunk
//   6. Sender → Receiver : Complete JSON { sha256 } + \n
//   7. Receiver → Sender : FinalAck 0x01 (OK) ou 0x00 (REJET)
//
// Metadata JSON :
//   { "rel_path": "photos/vacances/img1.jpg", "size": 104857600,
//     "sha256": "e3b0c44...", "is_dir": false }
//
// Taille de chunk : 1 048 576 octets (1 Mo)
// Timeout renvoi : 10s, 3 tentatives max par chunk
// SHA-256 calcule progressivement cote Receiver, compare a la fin
//
// Connexion : QUIC sur le port 58200
// - Chaque pair demarre un serveur QUIC au lancement
// - L'envoi ouvre une connexion sortante vers l'IP du destinataire
// - Handshake TLS 1.3 automatique via Quinn
//
// Pour un dossier :
//   - Le sender parcourt recursivement l'arborescence
//   - Ouvre un stream QUIC dedie pour chaque fichier
//   - rel_path conserve la structure (ex: "photos/vacances/img1.jpg")
//   - Les streams sont ouverts en parallele (multiplexage QUIC)
//   - Les dossiers vides sont signales par is_dir=true sans chunks
//
// Gestion des erreurs :
//   - Si aucun Ack recu dans les 10s, renvoi du chunk (max 3 fois)
//   - Si SHA-256 mismatch, FinalAck 0x00 + fichier supprime cote receiver
//   - Annulation : fermeture du stream, le receiver ignore les residus
//
// start_sender(ui, paths: Vec<PathBuf>, peer_addr, stop)
// start_receiver(ui, dest_dir, stop)
