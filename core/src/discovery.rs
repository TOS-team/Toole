// Découverte des appareils sur le réseau ad hoc via UDP broadcast (port 5199).
//
// start_discovery_broadcast(ui, stop)
//   - Sender envoie "TOOLE_DISCOVER" en broadcast toutes les 2s
//   - Les Receivers répondent "TOOLE_HERE:<hostname>"
//   - Chaque réponse → ui.on_peer_found()
//   - Timeout 10s si aucun Receiver
//
// listen_for_discovery(ui, stop)
//   - Receiver écoute les paquets "TOOLE_DISCOVER"
//   - Répond avec "TOOLE_HERE:<hostname>" à l'expéditeur
