# Architecture Technique de Toolé (etat actuel)

Hello le BOP, ici on pose l'architecture claire du projet pour que tout le monde code dans le meme sens.

## 1) Vue d'ensemble

Toolé est organise en 3 couches:

1. **Discovery UDP**: annonce de presence + ecoute des voisins.
2. **Controle TCP**: HELLO, HEARTBEAT, relais voisin, election, annonce du nouveau Master.
3. **Transfert fichier TCP**: envoi/reception fiable avec metadata (nom + taille) puis contenu.

## 2) Structure des dossiers

- `includes/`: contrats partages (`states.h`, `discovery.h`, `network.h`, `server_runtime.h`, `file_transfert.h`, `app.h`)
- `src/platform/linux/`: implementation Linux
  - `discovery.c`: presence beacon + ecoute + nettoyage de la liste
  - `network.c`: primitives TCP + messages de controle cluster
  - `file_transfert.c`: envoi/reception de fichiers
  - `server_runtime.c`: orchestration runtime (heartbeat/timeout/failover/election/reco)
- `src/core/`: boucle applicative
  - `app.c`: machine d'etat runtime (DISCOVERING/CLIENT/MASTER/ELECTION)
  - `main.c`: entrypoint CLI et boucle de tick orientee futur bridge UI Python
- `src/platform/windows/`: base Windows (incomplete pour l'instant)
- `src/ui/`: UI Python (customtkinter)

## 3) Donnees centrales partagees

Le coeur de l'etat reseau est dans `info` (`states.h`):

- `id`, `username`, `ip`, `tcp_port`
- `r` (ROLE_CLIENT / ROLE_MASTER)
- `cluster_id`
- `master_ip`, `master_port`

Ce format est utilise par discovery, controle et failover.

## 4) Discovery UDP (couche 1)

### Emission
`presence()` envoie periodiquement un beacon de forme:

`toole|id|username|ip|tcp_port|role|cluster_id|master_ip|master_port|message`

### Ecoute
`hear()` parse les beacons, dedoublonne par `id`, et met a jour la liste des appareils.

### Hygiene
`cleaner()` retire les noeuds muets depuis plus de 10 secondes.

## 5) Controle TCP (couche 2)

Le contrat est dans `includes/network.h`.

### Messages de controle supportes

- `NETWORK_MSG_HELLO`
- `NETWORK_MSG_HEARTBEAT`
- `NETWORK_MSG_MASTER_ANNOUNCE`
- `NETWORK_MSG_RELAY_REQUEST`
- `NETWORK_MSG_RELAY_RESPONSE`
- `NETWORK_MSG_ELECTION_VOTE`
- `NETWORK_MSG_ELECTION_RESULT`

### Encapsulation commune

Chaque message est un `network_ctrl_msg`:

- `magic` (signature protocole)
- `version`
- `type`
- `sender` (`info`)
- `payload` court

### Primitives reseau

- creation/connexion/acceptation sockets
- timeouts (`SO_RCVTIMEO`, `SO_SNDTIMEO`)
- envoi/reception de structure complete (`network_send_struct`, `network_recv_struct`)
- fermeture propre (`network_close`)

## 6) Runtime failover (couche 2 etat vivant)

Le runtime Linux est dans `server_runtime.c`.

### Ce que le runtime sait faire

- envoyer un heartbeat (`runtime_send_heartbeat_once`)
- attendre un message controle avec timeout (`runtime_wait_control`)
- executer un pas client heartbeat + detection perte master (`runtime_client_step`)
- elire un nouveau master depuis la liste de devices (`runtime_elect_master_from_devices`)
- diffuser une annonce master a des clients (`runtime_broadcast_master_to_clients`)
- tenter reconnexion directe ou via voisin (`runtime_try_reconnect_from_devices`)
- executer le failover client complet (`runtime_client_failover`)

### Regle d'election

Regle deterministe actuelle: **le plus petit `id` gagne**.

## 7) Transfert fichier TCP (couche 3)

Le module `file_transfert.c` fait:

1. envoi metadata (`name_len`, `file_size`, `filename`)
2. envoi flux fichier
3. reception metadata
4. ecriture du fichier de destination

L'header associe est `includes/file_transfert.h`.

## 8) Frontiere actuelle du projet

- Linux: socle reseau et failover de base operationnels
- Windows: non finalise
- `app.c` / `main.c`: branches sur le runtime reseau Linux (mode CLI de base)
- UI Python: lanceable avec dependances Python (`requirements.txt`)

---

> Pour les schemas textuels de flux: voir [diagram.md](diagram.md)
