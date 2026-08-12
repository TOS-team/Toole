# Comment fonctionne Toolé — guide développeur

## Vue d'ensemble

```
┌─────────────────────┐      UDP broadcast (58199)      ┌─────────────────────┐
│  Machine A (envoi)  │ ◄─────────────────────────────► │  Machine B (réception)│
│                     │      TOOLE_DISCOVERY / HERE     │                     │
│  send_files ────────┼──► QUIC (TLS 1.3) ────────────►│   récepteur (58200) │
│                     │      port destination :58200    │  → Téléchargements/ │
└─────────────────────┘                                  │     Toolé/          │
                                                          └─────────────────────┘
```

Deux mécanismes indépendants :

1. **Découverte** : UDP broadcast, port **58199**, pour trouver les voisins.
2. **Transfert** : QUIC (TLS 1.3 intégré), port **58200**, pour envoyer les
   fichiers de façon chiffrée et fiable.

## Architecture

Le code est découpé en deux crates et une app :

| Crate | Rôle |
|---|---|
| `toole_core` (`core/`) | Bibliothèque pure sans Tauri : logique réseau |
| `app` (`desktop-app/src-tauri/`) | App Tauri : ponts avec l'interface |
| Frontend (`desktop-app/ui/`) | Vue 3 + Pinia, rendu webview |

### `core/`

| Module | Responsabilité |
|---|---|
| `discovery.rs` | Écoute/broadcast UDP 58199, gestion du timeout des pairs |
| `transfer.rs` | Endpoints QUIC, protocole par stream, chunks pipelinés |
| `sender.rs` | Émetteur : collecte des chemins, envoi en parallèle (max 2) |
| `receiver.rs` | Récepteur QUIC 58200 → écrit les fichiers dans `Téléchargements/Toolé` |
| `file_certif.rs` | Certificat TLS auto-signé persistant + vérification désactivée |
| `utils.rs` | `device_id` stable (hostname + suffixe Crockford) |
| `error.rs` | Type `ToolError` |
| `lib.rs` | Trait `UI` (10 méthodes) + type `Peer` |

### `desktop-app/src-tauri/`

| Fichier | Responsabilité |
|---|---|
| `commands.rs` | `AppUI` (implémente `UI` en émettant des événements `tool://*`), commandes invoke |
| `lib.rs` | Builder Tauri, states (`DiscoveryState`, `TransferState`), récepteur au setup |

### Frontend

| Fichier | Responsabilité |
|---|---|
| `stores/peers.ts` | Liste des pairs (polling `get_peers` toutes les 2 s) |
| `stores/files.ts` | Fichiers ajoutés pour l'envoi |
| `stores/transfers.ts` | Historique + état des transferts (localStorage, 200 max) |
| `stores/settings.ts` | Thème clair/sombre/auto + halo (variables CSS) |
| `tauri.ts` | Wrapper `invoke` |

## Le pont UI (trait `UI`)

Le cœur Rust ne connaît pas Tauri : il parle au frontend via le **trait `UI`**
(`core/src/lib.rs`). L'app fournit une implémentation `AppUI` qui traduit
chaque méthode en **événement** `tool://*` émis vers la webview.

| Méthode du trait | Événement émis |
|---|---|
| `peer_found(peer)` | `tool://peer_found` |
| `peer_lost(id)` | `tool://peer_lost` |
| `show_progress_bar(id)` | `tool://transfer/start` |
| `update_progress_bar(id, sent, total)` | `tool://transfer/progress` |
| `file_progress_bar(id, name, sent, total)` | `tool://transfer/file_progress` |
| `transfert_cancel(id)` | `tool://transfer/cancel` |
| `transfert_completed(id)` | `tool://transfer/done` |
| `transfert_received(...)` | `tool://transfer/received` |
| `transfert_error(id, err)` | `tool://transfer/error` |
| `log(msg)` | `tool://log` |

Le frontend s'abonne à ces événements dans les stores (`transfers.ts`,
`peers.ts`).

## Découverte (UDP)

### Émission

Toutes les 3 secondes, chaque machine diffuse `TOOLE_DISCOVERY` sur :
- l'adresse broadcast de **chaque interface** (déduite si l'OS ne la fournit pas),
- le broadcast universel `255.255.255.255` en secours (souvent filtré en WiFi).

### Réponse

À la réception de `TOOLE_DISCOVERY`, la machine répond `TOOLE_HERE:<device_id>`
à l'adresse source (unicast).

### Timeout

Chaque pair vu est mémorisé avec un horodatage. Si plus de **9 secondes**
s'écoulent sans réponse, le pair est retiré (`peer_lost`).

### `device_id`

Stable et unique : `hostname` + suffixe de 5 caractères en base32 Crockford
(pas de chiffres ambigus I, L, O, U). Généré une fois, stocké sur disque.

## Transfert (QUIC)

### Certificat

Chaque machine génère un **certificat auto-signé** persistant (clé ECDSA P-256,
valide 2026–2036), stocké dans le dossier de données de l'application. Le
client **ne vérifie pas** le certificat (`SkipServerVerification`) : le réseau
local est considéré de confiance, et QUIC fournit le chiffrement et l'intégrité
des paquets.

### Établissement

1. L'émetteur ouvre un endpoint client et **se connecte** à `peer:58200`.
2. Le récepteur accepte la connexion (endpoint serveur sur `0.0.0.0:58200`).
3. L'émetteur collecte les fichiers (`collect_entries`, récursif pour les
   dossiers) et calcule le **volume total**.

### Protocole par stream

Chaque fichier (ou dossier vide) est envoyé sur son **propre stream bidirectionnel**
(`open_bi`), ce qui permet le multiplexage QUIC :

```
Émetteur                                 Récepteur
──────────────────────────  ──────────────────────────
Metadata (JSON + \n)
 ─────────────────────────►  (crée les dossiers)
Ack (0x01) ◄───────────────
Chunk 1: [len u32 BE][data]
Chunk 2: [len u32 BE][data]   pas d'ack par chunk
...
 ─────────────────────────►  (écrit sur disque)
COMPLETE (0x02) ───────────►
Ack final (0x01) ◄──────────
finish()                              finish()
```

### Metadata

```json
{ "transfer_id": "...", "rel_path": "photos/vacances.jpg", "size": 123456, "is_dir": false }
```

- JSON sur une ligne, terminé par `\n`.
- `rel_path` préserve l'arborescence (les dossiers sont recréés côté récepteur).
- Pas de champ `sha256` : l'intégrité repose entièrement sur QUIC/TLS 1.3.
- Les **dossiers vides** sont transmis comme des streams `is_dir: true`.

### Pipelining (pas d'ack par chunk)

- Les chunks font au maximum **1 Mo** (`len` u32 BE + données).
- Aucun ack par chunk : QUIC garantit la livraison et l'ordre. Le récepteur
  écrit en continu ce qui lui arrive.
- C'est ce qui permet ~190 Mo/s en loopback (vs ~90 avec ack applicatif).
- Un seul `ACK` en début (validation des métadonnées) et un à la fin
  (confirmation complète).

### Fenêtres QUIC

Les fenêtres par défaut de quinn plafonnent le débit sur le réseau local. Toolé
les élargit : **32 Mo** de réception/émission, **8 Mo** par stream.

### Progression

- L'émetteur et le récepteur partagent le **même `transfer_id`** (celui de
  l'émetteur, lu dans la metadata), pour afficher la même progression.
- Les événements UI sont **throttlés à ~20/s** (`UiThrottle`, 50 ms min) pour ne
  pas saturer le pont frontend en boucle serrée.

## Parallélisme et annulation

### Sémaphore (2 fichiers)

Le `sender` lance une tâche par fichier mais limite à **2 en parallèle**
(`Semaphore::new(2)`) pour ne pas saturer la liaison.

### Annulation

- Chaque commande `send_files` enregistre un `stop_flag` (AtomicBool) et un
  `AbortHandle` dans `TransferState`.
- `cancel_transfer` pose le flag puis abort la tâche.
- L'émetteur, en vérifiant le flag, **resets son stream** (`send.reset`).
- Côté récepteur, le stream échoue → `had_error` est posé → la connexion est
  signalée en **erreur**, jamais comme un transfert reçu ou tronqué.

## Réception

Le récepteur démarre au **setup de l'app** (`lib.rs`) et écrit dans
`dirs::download_dir()/Toolé/`. Chaque connexion entrante est traitée dans une
tâche Tokio dédiée ; chaque stream dans une sous-tâche.

## Cycle d'une commande d'envoi (frontend)

1. L'utilisateur coche des appareils et clique **Transférer** (`App.vue`).
2. Pour chaque appareil coché : `invoke("send_files", { paths, peerAddr })`
   avec `peerAddr = peer.addr + ":58200"`.
3. Le backend crée un `transfer_id` UUID et lance `start_sender`.
4. Les événements `tool://transfer/*` alimentent le store `transfers`.
5. L'app bascule sur la page **Transferts**.

---

> [Sommaire](index.md) · Références techniques : [docs/](../docs/)
