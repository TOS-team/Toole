// Transfert de fichiers par TCP + TLS (port 5200).
//
// Protocole :
//   1. Sender → Receiver : Metadata JSON (filename, size, sha256, chunk_size)
//   2. Receiver → Sender : Ack 0x01
//   3. Sender → Receiver : Chunk (chunk_index u32 big-endian + data)
//   4. Receiver → Sender : Ack (chunk_index u32 big-endian)
//   5. Répéter 3-4 jusqu'au dernier chunk
//   6. Sender → Receiver : Complete JSON { sha256 }
//   7. Receiver → Sender : FinalAck 0x01 (OK) ou 0x00 (REJET)
//
// Taille de chunk : 1 048 576 octets (1 Mo)
// TLS via rustls : certificat auto-signé généré par rcgen, automatique sans validation
// Timeout renvoi : 10s, 3 tentatives max
// SHA-256 calculé progressivement côté Receiver, comparé à la fin
//
// start_sender(ui, file_path, stop)
// start_receiver(ui, dest_dir, stop)
