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

Chaque fichier est transféré sur un **stream bidirectionnel QUIC** dédié :

| Étape | Direction | Contenu |
|---|---|---|
| 1. Metadata | Sender → Receiver | JSON : `{ transfer_id, rel_path, size, is_dir }` terminé par `\n` |
| 2. Ack | Receiver → Sender | `0x01` |
| 3. Chunks | Sender → Receiver | `len` (u32 big-endian) + data (1 Mo max) |
| ... | ... | Répéter jusqu'au dernier chunk (pas d'ack par chunk) |
| 4. Complete | Sender → Receiver | `0x02` (marqueur de fin de fichier) |
| 5. FinalAck | Receiver → Sender | `0x01` |

### Metadata (JSON)

```json
{
  "transfer_id": "550e8400-e29b-41d4-a716-446655440000",
  "rel_path": "photos/vacances/img1.jpg",
  "size": 104857600,
  "is_dir": false
}
```

> Le `transfer_id` est généré par l'émetteur et partagé au récepteur pour que les deux affichent la même progression. Il n'y a **pas** de champ `sha256` : l'intégrité est assurée par QUIC/TLS 1.3 (voir [crypto.md](crypto.md)).

### Fiabilité

- **Pas d'ack par chunk** : QUIC gère la fiabilité, la congestion et le renvoi automatiquement — on pipeline les chunks sans attendre de confirmation applicative.
- **Chunk** : `len` (u32 big-endian) suivi des données, chunks de 1 Mo max.
- **Fin de fichier** : un marqueur `0x02` (`COMPLETE`) après le dernier chunk.
- **Annulation** : le sender reset son stream (`stop`), le récepteur détecte l'échec et signale une erreur — jamais une réception tronquée.
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
- **Annulation** : reset du stream QUIC côté sender, le récepteur marque un flux en échec et la connexion est signalée en erreur (pas comme un transfert reçu)

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
```

---

> [Architecture technique](architecture.md) | Lire ensuite : [Chiffrement et intégrité](crypto.md)
