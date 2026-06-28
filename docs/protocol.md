# Protocole Réseau — Toolé

## UDP — Découverte (port 58199)

L'UDP est utilisé pour la découverte des appareils sur le réseau local.

| Direction | Paquet | Description |
|---|---|---|
| Broadcast | `TOOLE_DISCOVERY` | Diffusé toutes les 3s sur `255.255.255.255:58199` |
| Unicast | `TOOLE_HERE:<hostname>` | Réponse à l'expéditeur, ex: `TOOLE_HERE:PC-Gerard` |

### Détails

- **Port** : 58199
- **Intervalle de broadcast** : 3 secondes
- **Timeout d'expiration d'un pair** : 9 secondes sans réponse
- **Format du hostname** : nom de la machine (via `gethostname`)
- **Filtrage** : un appareil ne s'ajoute pas lui-même (même hostname + même IP)

---

## QUIC — Transfert (port 5200)

Le transfert utilise **QUIC via Quinn** pour le transport.

### Pourquoi QUIC ?

- **Multiplexage natif** : plusieurs fichiers en parallèle sur une seule connexion
- **Pas de Head-of-Line blocking** : un fichier lent ne bloque pas les autres
- **TLS 1.3 intégré** : chiffrement obligatoire, sans configuration manuelle
- **Contrôle de congestion et renvoi** : géré automatiquement par QUIC

### Établissement de connexion

1. Chaque pair démarre un **serveur QUIC** sur le port 5200
2. Pour envoyer des fichiers, le pair initiateur ouvre une **connexion QUIC** vers l'adresse du destinataire
3. Le handshake TLS 1.3 s'effectue automatiquement
4. La connexion est réutilisable pour plusieurs transferts

### Protocole applicatif (par stream)

Chaque fichier est transféré sur un **stream bidirectionnel QUIC** dédié :

| Étape | Direction | Contenu |
|---|---|---|
| 1. Metadata | Sender → Receiver | JSON : `{ rel_path, size, sha256, is_dir }` terminé par `\n` |
| 2. Ack | Receiver → Sender | `0x01` |
| 3. Chunks | Sender → Receiver | `chunk_index` (u32 big-endian) + data (1 Mo) |
| 4. Ack chunk | Receiver → Sender | `chunk_index` (u32 big-endian) |
| ... | ... | Répéter 3-4 jusqu'au dernier chunk |
| 5. Complete | Sender → Receiver | JSON : `{ sha256 }` terminé par `\n` |
| 6. FinalAck | Receiver → Sender | `0x01` (OK) ou `0x00` (REJET) |

### Metadata (JSON)

```json
{
  "rel_path": "photos/vacances/img1.jpg",
  "size": 104857600,
  "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "is_dir": false
}
```

### Transfert de dossiers

Pour un dossier, le sender :
1. Parcourt récursivement l'arborescence
2. Ouvre un stream QUIC dédié pour **chaque fichier**
3. Le `rel_path` dans Metadata conserve la structure relative (ex: `photos/vacances/img1.jpg`)
4. Les dossiers vides sont signalés par `{ "rel_path": "dossier_vide", "is_dir": true, "size": 0 }` sans chunks
5. Tous les streams sont ouverts en parallèle (multiplexage QUIC)

### Gestion des erreurs

- **Timeout** : 10s sans Ack, 3 tentatives de renvoi max
- **Chunk perdu** : renvoyé automatiquement par QUIC + mécanisme applicatif
- **SHA-256 mismatch** : FinalAck `0x00`, le fichier est supprimé côté récepteur
- **Annulation** : fermeture du stream QUIC côté sender, le récepteur ignore les chunks résiduels

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
  │=== Connexion QUIC (port 5200) ====> │
  │◄══ Handshake TLS 1.3 ══════════════►│
  │                                     │
  │─── Stream 1 : Metadata ────────────►│
  │◄── Ack 0x01 ───────────────────────│
  │─── Stream 1 : Chunk(0) ───────────►│
  │◄── Ack(0) ─────────────────────────│
  │─── Stream 1 : Chunk(1) ───────────►│
  │◄── Ack(1) ─────────────────────────│
  │─── ... en parallèle ───────────────►│
  │                                     │
  │─── Stream 2 : Metadata ────────────►│
  │◄── Ack 0x01 ───────────────────────│
  │─── Stream 2 : Chunk(0) ───────────►│
  │◄── Ack(0) ─────────────────────────│
  │─── ...                             │
  │                                     │
  │─── Stream 1 : Complete ───────────►│
  │◄── FinalAck 0x01 ──────────────────│
  │─── Stream 2 : Complete ───────────►│
  │◄── FinalAck 0x01 ──────────────────│
```

---

> [Architecture technique](architecture.md) | Lire ensuite : [Chiffrement et intégrité](crypto.md)
