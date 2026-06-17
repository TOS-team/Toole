// Point d'entrée de la bibliothèque core.
//
// Réexporte tous les modules publics : error, utils, discovery, transfer, network.
//
// Définit aussi :
//   - Trait UI          : pont entre core et le frontend (log, set_progress, on_peer_found, on_transfer_done, on_error)
//   - struct Peer       : nom + adresse d'un appareil détecté
//   - enum Mode         : Send / Receive
//   - struct TransferStatus : filename, total_bytes, received_bytes, speed
