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

## QUIC — Transfert (port 5200) — NON IMPLÉMENTÉ

Le transfert utilise **QUIC via Quinn** pour le transport.

### Pourquoi QUIC ?

- **Multiplexage natif** : plusieurs fichiers en parallèle sur une seule connexion
- **Pas de Head-of-Line blocking** : un fichier lent ne bloque pas les autres
- **TLS 1.3 intégré** : chiffrement obligatoire, sans configuration manuelle
- **Contrôle de congestion et renvoi** : géré automatiquement par QUIC

### Principe

1. Connexion QUIC établie entre les deux pairs (client bootstrap / serveur)
2. Chaque fichier = un **stream bidirectionnel QUIC** dédié
3. Les streams circulent en **parallèle** sur la même connexion
4. Les dossiers sont transférés récursivement (un stream par fichier)

### Protocole applicatif (par stream)

| Étape | Direction | Contenu |
|---|---|---|
| 1. Metadata | Sender → Receiver | JSON : `rel_path`, `size`, `sha256`, `is_dir` |
| 2. Ack | Receiver → Sender | `0x01` |
| 3. Chunks | Sender → Receiver | `chunk_index` (u32 BE) + data (1 Mo) |
| 4. Ack chunk | Receiver → Sender | `chunk_index` (u32 BE) |
| 5. Complete | Sender → Receiver | JSON : `{ sha256 }` |
| 6. FinalAck | Receiver → Sender | `0x01` (OK) ou `0x00` (REJET) |

### Transfert de dossiers

Pour un dossier, le sender :
- Parcourt récursivement l'arborescence
- Ouvre un stream QUIC dédié pour **chaque fichier**
- Le `rel_path` dans Metadata conserve la structure (ex: `photos/vacances/img1.jpg`)
- Tous les streams sont ouverts en parallèle (multiplexage QUIC)

### Détails

- **Port** : 5200
- **Taille de chunk** : 1 048 576 octets (1 Mo)
- **Timeout renvoi** : 10s, 3 tentatives max
- **SHA-256** : calculé progressivement côté récepteur, comparé à la fin

---

## Séquence actuelle

```
Appareil A                           Appareil B
  │                                     │
  │─── TOOLE_DISCOVERY (UDP) ──────────►│
  │◄── TOOLE_HERE:PC-B (UDP) ──────────│
  │                                     │
  │─── TOOLE_DISCOVERY (UDP) ──────────►│
  │◄── TOOLE_HERE:PC-B (UDP) ──────────│
  │                                     │
  │  (toutes les 3s)                    │
  │  (si pas de réponse pendant 9s,     │
  │   le pair est retiré de la liste)   │
```

---

> [Architecture technique](architecture.md) | Lire ensuite : [Chiffrement et intégrité](crypto.md)
