# Architecture Technique — Toolé

## Vue globale

```
┌──────────────────────────────────────────────────┐
│                  Toolé (workspace)                │
│                                                    │
│  ┌──────────────────────┐  ┌────────────────────┐ │
│  │       core/           │  │  desktop-app/      │ │
│  │  (bibliothèque pure)  │  │  (application Tauri)│ │
│  │                       │  │                     │ │
│  │  - lib.rs (trait UI)  │  │  src-tauri/        │ │
│  │  - error.rs           │  │  ├─ commands.rs     │ │
│  │  - utils.rs           │  │  ├─ lib.rs          │ │
│  │  - discovery.rs       │  │  └─ build.rs        │ │
│  │  - transfer.rs        │  │  ui/                │ │
│  └──────────┬────────────┘  │  ├─ index.html      │ │
│             │               │  ├─ css/index.css   │ │
│             │               │  └─ js/main.js      │ │
│             │               └────────────────────┘ │
│             │                                       │
│             └───────────┬───────────────┐           │
│                         │               │           │
│                  Trait UI (IPC)    Polling get_peers│
└─────────────────────────────────────────────────────┘
```

---

## Pourquoi workspace core/ + desktop-app/ ?

`core/` est une bibliothèque Rust pure, **sans aucune dépendance Tauri**. Elle expose un **trait `UI`** générique que n'importe quel frontend peut implémenter.

`desktop-app/src-tauri/` est l'application Tauri qui implémente le trait `UI` en stockant les pairs dans un état partagé. Le frontend HTML/JS interroge le backend par **polling** (toutes les 2s) via la commande `get_peers`.

Cette séparation permet :
- De tester `core/` sans Tauri (`cargo run -p toole_core`)
- De réutiliser `core/` pour d'autres interfaces (CLI, Android...)
- De découpler la logique métier de l'interface

---

## Trait UI — Le pont entre core et le frontend

```rust
pub trait UI: Send + Sync {
    fn log(&self, msg: &str);
    fn peer_found(&self, peer: &Peer);
    fn peer_lost(&self, hostname: &str);
    fn transfer_progress(&self, peer_addr: &str, rel_path: &str, percent: u8, speed: &str);
    fn transfer_done(&self, peer_addr: &str, rel_path: &str, success: bool);
    fn transfer_error(&self, peer_addr: &str, rel_path: &str, err: &str);
}
```

Dans `desktop-app/src-tauri/src/commands.rs`, la structure `TauriUI` implémente ce trait :
- `log`, `peer_found`, `peer_lost` mettent à jour une liste partagée `Arc<Mutex<Vec<Peer>>>`
- `transfer_progress`, `transfer_done`, `transfer_error` sont relayés au frontend par polling ou events

Le frontend récupère la liste des pairs via la commande `get_peers` appelée toutes les 2s.

---

## Architecture réseau

```
                   ┌──────────────┐
                   │  Appareil    │
                   │              │
                   │  Broadcast   │
                   │  UDP toutes  │
                   │  les 3s      │
                   └──────┬───────┘
                          │ Réseau local (LAN)
                          │
             ┌────────────┴────────────┐
             │                         │
             ▼                         ▼
      ┌──────────────┐        ┌──────────────┐
      │  Pair 1      │        │  Pair 2      │
      │  (Toolé)     │        │  (Toolé)     │
      └──────────────┘        └──────────────┘

      1. Chaque app broadcast TOOLE_DISCOVERY en UDP
      2. Les autres répondent TOOLE_HERE:<hostname>
      3. La liste des pairs s'affiche dans l'interface
```

---

## Architecture transfert QUIC

```
Connexion QUIC (TLS 1.3 intégré)
│
├── Stream 1 : fichier "rapport.pdf"
│   ├── Metadata JSON (rel_path, size, sha256, is_dir)
│   ├── Chunks 1 Mo + Ack (chunk_index u32 BE)
│   └── Complete JSON + FinalAck
│
├── Stream 2 : fichier "photos/vacances/img1.jpg"
│   ├── Metadata JSON
│   ├── Chunks 1 Mo + Ack
│   └── Complete + FinalAck
│
├── Stream 3 : fichier "photos/vacances/img2.jpg" (en parallèle)
│   └── ...
│
└── Tous les streams circulent simultanément
    Avantage : pas de Head-of-Line blocking
```

---

## Architecture async (Tokio)

```
Tokio Runtime
│
├── Tâche broadcast + écoute UDP        (envoi TOOLE_DISCOVERY + réponse TOOLE_HERE)
├── Tâche serveur QUIC (port 5200)      (connexions entrantes)
├── Tâche client QUIC                   (connexions sortantes)
└── Tâches streams QUIC (1 par fichier) (transfert en parallèle)
```

---

## Structure des modules

### core/src/

| Module | Responsabilité |
|---|---|
| `lib.rs` | Trait UI, type Peer, types TransferStatus |
| `error.rs` | ToolError (IoError, Canceled, TransferError) |
| `utils.rs` | current_hostname, local_ip |
| `discovery.rs` | UDP broadcast (TOOLE_DISCOVERY / TOOLE_HERE), port 58199 |
| `transfer.rs` | Transfert QUIC : serveur, client, streams, chunks, SHA-256 |

### desktop-app/src-tauri/src/

| Fichier | Responsabilité |
|---|---|
| `main.rs` | Point d'entrée, appelle `app_lib::run()` |
| `lib.rs` | Builder Tauri : manage state, on_window_event, invoke_handler |
| `commands.rs` | TauriUI + commandes : start_discovery, stop_discovery, get_hostname, get_peers, start_transfer, cancel_transfer |

### desktop-app/ui/

| Fichier | Responsabilité |
|---|---|
| `index.html` | Structure : header, zone dépôt, liste appareils, progression, modale |
| `css/index.css` | Thème glassmorphism dark |
| `js/main.js` | Polling get_peers, drag & drop, sélection, upload, barres progression |

---

> [SRS — exigences logicielles](srs.md) | Lire ensuite : [Protocole réseau](protocol.md)
