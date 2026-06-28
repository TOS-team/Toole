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
│  │  - transfer.rs (plan) │  │  ui/                │ │
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
}
```

Dans `desktop-app/src-tauri/src/commands.rs`, la structure `TauriUI` implémente ce trait en mettant à jour une liste partagée `Arc<Mutex<Vec<Peer>>>` :

```rust
impl UI for TauriUI {
    fn log(&self, _msg: &str) { }
    fn peer_found(&self, peer: &Peer) {
        self.peers.lock().unwrap().push(peer.clone());
    }
    fn peer_lost(&self, hostname: &str) {
        self.peers.lock().unwrap().retain(|p| p.hostname != hostname);
    }
}
```

Le frontend récupère la liste via la commande `get_peers` appelée toutes les 2s.

---

## Architecture réseau (actuelle)

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

**Note :** Le transfert de fichiers (TCP+TLS) n'est pas encore implémenté.

---

## Architecture async (Tokio)

```
Tokio Runtime
│
└── Tâche broadcast + écoute UDP    (envoi TOOLE_DISCOVERY + réponse TOOLE_HERE)
```

---

## Structure des modules

### core/src/

| Module | Responsabilité |
|---|---|
| `lib.rs` | Trait UI, type Peer |
| `error.rs` | ToolError (IoError, Canceled) |
| `utils.rs` | current_hostname, local_ip |
| `discovery.rs` | UDP broadcast (TOOLE_DISCOVERY / TOOLE_HERE), port 58199 |
| `transfer.rs` | Plan du protocole de transfert (non implémenté) |

### desktop-app/src-tauri/src/

| Fichier | Responsabilité |
|---|---|
| `main.rs` | Point d'entrée, appelle `app_lib::run()` |
| `lib.rs` | Builder Tauri : manage state, on_window_event, invoke_handler |
| `commands.rs` | Implémentation TauriUI + commandes : start_discovery, stop_discovery, get_hostname, get_peers |

### desktop-app/ui/

| Fichier | Responsabilité |
|---|---|
| `index.html` | Structure de l'interface : header, zone dépôt, liste appareils, modale |
| `css/index.css` | Thème glassmorphism dark |
| `js/main.js` | Polling get_peers, affichage liste, sélection, init auto |

---

> [SRS — exigences logicielles](srs.md) | Lire ensuite : [Protocole réseau](protocol.md)
