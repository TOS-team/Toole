# Protocole Réseau — Toolé

## UDP — Découverte (port 58199)

L'UDP est utilisé pour la découverte des appareils sur le réseau local.

| Direction | Paquet | Description |
|---|---|---|
| Broadcast | `TOOLE_DISCOVERY` | Diffusé toutes les 3s sur `255.255.255.255:58199` |
| Unicast | `TOOLE_HERE:<device_id>` | Réponse à l'expéditeur, ex: `TOOLE_HERE:PC-Gerard-3k9f2` |

### Détails

- **Port** : 58199
- **Intervalle de broadcast** : 3 secondes
- **Timeout d'expiration d'un pair** : 9 secondes sans réponse
- **Format du device_id** : `hostname` + `-` + suffixe de 5 caractères (base32 Crockford), stable via un fichier `device_id` (voir `utils::device_id`)
- **Filtrage** : un appareil ne s'ajoute pas lui-même (même device_id + même IP)

---

## QUIC — Transfert (port 58200)

Le transfert utilise **QUIC via Quinn** pour le transport.

### Pourquoi QUIC ?

- **Multiplexage natif** : plusieurs fichiers en parallèle sur une seule connexion
- **Pas de Head-of-Line blocking** : un fichier lent ne bloque pas les autres
- **TLS 1.3 intégré** : chiffrement obligatoire, sans configuration manuelle
- **Contrôle de congestion et renvoi** : géré automatiquement par QUIC

### Établissement de connexion

1. Chaque pair démarre un **serveur QUIC** sur le port 58200
2. Pour envoyer des fichiers, le pair initiateur ouvre une **connexion QUIC** vers l'adresse du destinataire
3. Le handshake TLS 1.3 s'effectue automatiquement
4. La connexion est réutilisable pour plusieurs transferts

### Protocole applicatif (par stream)

Dès la connexion établie, l'émetteur ouvre un **premier stream dédié** pour
l'en-tête de lot, puis un stream par fichier :

| Étape | Direction | Contenu |
|---|---|---|
| 0. En-tête de lot | Sender → Receiver | JSON : `{ transfer_id, total_bytes, sender, files }` sur le 1er stream |
| 1. Décision | Receiver → Sender | `0x01` (**ACK**, accepter) ou `0x03` (**REFUSE**, refuser) |
| 2. Metadata | Sender → Receiver | JSON : `{ transfer_id, rel_path, size, is_dir }` terminé par `\n` |
| 3. Ack | Receiver → Sender | `0x01` |
| 4. Chunks | Sender → Receiver | `len` (u32 big-endian) + data (1 Mo max) |
| ... | ... | Répéter jusqu'au dernier chunk (pas d'ack par chunk) |
| 5. Complete | Sender → Receiver | `0x02` (marqueur de fin de fichier) |
| 6. FinalAck | Receiver → Sender | `0x01` |

### Demande d'acceptation (validation utilisateur)

Le récepteur **présente la demande à l'utilisateur** avant de recevoir le
moindre fichier (boutons « Accepter » / « Refuser »). L'émetteur attend la
décision sur le premier stream, avec un délai maximal de **30 s**
(`DECISION_TIMEOUT`) : à l'échéance, le transfert échoue côté émetteur. La
connexion reste vivante pendant l'attente grâce aux pings de garde QUIC
(voir [Fiabilité](#fiabilite)).

- **Accepter** : le récepteur répond `ACK`, l'émetteur envoie les fichiers.
- **Refuser** : le récepteur répond `REFUSE`, puis ferme la connexion avec le
  code d'annulation. Les deux côtés notifient `transfert_refused`.

### En-tête de lot (BatchHeader)

```json
{
  "transfer_id": "550e8400-e29b-41d4-a716-446655440000",
  "total_bytes": 209715200,
  "sender": "PC-Gerard-3k9f2",
  "files": ["photos/vacances/img1.jpg", "video.mkv"]
}
```

Le `total_bytes` est la **somme des tailles de tous les fichiers du lot**,
calculée par l'émetteur avant l'envoi. Le récepteur s'en sert immédiatement
comme dénominateur de la progression globale : les deux appareils affichent
ainsi la même barre dès le premier fichier (sans elle, le récepteur ne
connaissait le total qu'au fil des métadonnées et sa barre dérivait puis
reculait à chaque nouveau fichier).

> `sender` et `files` sont tolérants à la désérialisation (`#[serde(default)]`) :
> un ancien émetteur qui ne les envoie pas reste compatible. Le `transfer_id`
> est généré par l'émetteur et partagé au récepteur pour que les deux affichent
> la même progression. Il n'y a **pas** de champ `sha256` : l'intégrité est
> assurée par QUIC/TLS 1.3 (voir [crypto.md](crypto.md)).

### Fiabilité

- **Pas d'ack par chunk** : QUIC gère la fiabilité, la congestion et le renvoi automatiquement — on pipeline les chunks sans attendre de confirmation applicative.
- **Chunk** : `len` (u32 big-endian) suivi des données, chunks de 1 Mo max.
- **Fin de fichier** : un marqueur `0x02` (`COMPLETE`) après le dernier chunk.
- **Codes de fermeture QUIC** : `CLOSE_OK = 0` (fin normale), `CLOSE_CANCEL = 1` (annulation explicite, refus ou arrêt utilisateur). Le récepteur distingue une **annulation** (reset du stream ou fermeture `CLOSE_CANCEL`) d'une simple **erreur réseau** (fermeture avec un autre code, perte de connexion).
- **Annulation croisée** : les **deux côtés** peuvent annuler en cliquant sur la croix de la carte. Le sender reset ses streams et ferme avec `CLOSE_CANCEL` ; le récepteur qui reçoit un reset/`CLOSE_CANCEL` supprime les fichiers partiels et notifie `transfert_cancel`, jamais une réception tronquée.
- **Déconnexion soudaine** : `max_idle_timeout = 15 s` + `keep_alive_interval = 3 s`. Une app fermée brutalement (sans fermeture QUIC propre) est détectée en ~15 s via le timeout d'idle ; le destinataire signale alors une **erreur** et supprime le fichier partiel (contrôle de complétude `done < expected` en garde-fou).
- **Progression UI** : limitée à ~20 événements/s (`UiThrottle`) pour ne pas saturer le pont frontend.

### Transfert de dossiers

Pour un dossier, le sender :
1. Parcourt récursivement l'arborescence
2. Ouvre un stream QUIC dédié pour **chaque fichier**
3. Le `rel_path` dans Metadata conserve la structure relative (ex: `photos/vacances/img1.jpg`)
4. Les dossiers vides sont signalés par `{ "rel_path": "dossier_vide", "is_dir": true, "size": 0 }` sans chunks
5. Tous les streams sont ouverts en parallèle (multiplexage QUIC)

### Gestion des erreurs

- **Chunk perdu** : renvoyé automatiquement par QUIC (contrôle de congestion + renvoi)
- **Fin de fichier inattendue** : si le marqueur `COMPLETE` manque, le récepteur considère le protocole désynchronisé et échoue
- **Refus** : le récepteur répond `REFUSE` puis ferme la connexion ; les deux côtés notifient `transfert_refused`, aucun fichier n'est écrit
- **Annulation** : reset du stream QUIC ou fermeture `CLOSE_CANCEL` d'un côté, l'autre côté notifie `transfert_cancel` et nettoie les fichiers partiels
- **Perte de connexion** : une app qui disparaît sans fermeture propre (crash, arrêt brutal) est détectée par le timeout d'idle (~15 s) ; le flux est marqué en échec et la connexion est signalée en erreur, jamais comme un transfert reçu

---

## Séquence complète

```
Appareil A                           Appareil B
  │                                     │
  │─── TOOLE_DISCOVERY (UDP) ──────────►│
  │◄── TOOLE_HERE:PC-B (UDP) ──────────│
  │                                     │
  │ (découverte toutes les 3s)          │
  │                                     │
  │=== Connexion QUIC (port 58200) ===> │
  │◄══ Handshake TLS 1.3 ══════════════►│
  │                                     │
  │─── Stream 0 : BatchHeader ─────────►│
  │              (transfer_id, total,   │
  │               sender, files)        │
  │   [B affiche « Accepter / Refuser »]│
  │◄── Décision ACK 0x01 / REFUSE 0x03 ─│
  │                                     │
  │─── Stream 1 : Metadata ────────────►│
  │◄── Ack 0x01 ───────────────────────│
  │─── Stream 1 : len + Chunk(0) ─────►│
  │─── Stream 1 : len + Chunk(1) ─────►│
  │─── ... (pipeline, pas d'ack) ──────►│
  │                                     │
  │─── Stream 2 : Metadata ────────────►│
  │◄── Ack 0x01 ───────────────────────│
  │─── Stream 2 : len + Chunk(0) ─────►│
  │─── ...                             │
  │                                     │
  │─── Stream 1 : Complete 0x02 ───────►│
  │◄── FinalAck 0x01 ──────────────────│
  │─── Stream 2 : Complete 0x02 ───────►│
  │◄── FinalAck 0x01 ──────────────────│
  │                                     │
  │◄══ Fermeture CLOSE_OK (code 0) ═════│
```

**Refus** : au lieu du `ACK`, B répond `REFUSE 0x03` et ferme la connexion
avec `CLOSE_CANCEL` — aucun fichier n'est écrit. **Annulation** : l'un des deux
appareils ferme avec `CLOSE_CANCEL` (ou reset un stream) en cours de transfert ;
l'autre côté notifie l'annulation et supprime les fichiers partiels.
**Déconnexion soudaine** : sans fermeture propre, la perte est détectée par le
timeout d'idle QUIC (~15 s) et signalée comme erreur.

---

> [Architecture technique](architecture.md) | Lire ensuite : [Chiffrement et intégrité](crypto.md)
