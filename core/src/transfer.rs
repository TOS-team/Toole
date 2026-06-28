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
//   4. Envoi de dossiers recursifs = liste des fichiers + metadonnees
//
// Protocole applicatif (par stream) :
//   1. Sender → Receiver : Metadata JSON (rel_path, size, sha256, is_dir)
//   2. Receiver → Sender : Ack 0x01
//   3. Sender → Receiver : Chunk (chunk_index u32 big-endian + data)
//   4. Receiver → Sender : Ack (chunk_index u32 big-endian)
//   5. Repeter 3-4 jusqu'au dernier chunk
//   6. Sender → Receiver : Complete JSON { sha256 }
//   7. Receiver → Sender : FinalAck 0x01 (OK) ou 0x00 (REJET)
//
// Taille de chunk : 1 048 576 octets (1 Mo)
// Timeout renvoi : 10s, 3 tentatives max
// SHA-256 calcule progressivement cote Receiver, compare a la fin
//
// Pour un dossier :
//   - Le sender ouvre un stream dedie pour chaque fichier (recursif)
//   - Chaque stream commence par Metadata avec rel_path (ex: "photos/vacances/img1.jpg")
//   - Les streams sont ouverts en parallele (multiplexage QUIC)
//
// start_sender(ui, paths: Vec<PathBuf>, addr, stop)
// start_receiver(ui, dest_dir, stop)
