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
│  │  - transfer.rs        │  │  │  ├─ commands.rs     │ │
│  │                       │  │  │  ├─ lib.rs          │ │
│  │                       │  │  │  └─ build.rs        │ │
│  │                       │  │  ui/                   │ │
│  │                       │  │  ├─ index.html         │ │
│  │                       │  │  ├─ package.json       │ │
│  │                       │  │  ├─ vite.config.ts     │ │
│  │                       │  │  ├─ tsconfig.json      │ │
│  │                       │  │  └─ src/               │ │
│  │                       │  │      ├─ App.vue        │ │
│  │                       │  │      ├─ style.css      │ │
│  │                       │  │      ├─ main.ts        │ │
│  │                       │  │      ├─ types.ts       │ │
│  │                       │  │      ├─ tauri.ts       │ │
│  │                       │  │      ├─ utils.ts       │ │
│  │                       │  │      ├─ components/    │ │
│  │                       │  │      └─ stores/        │ │
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

`desktop-app/src-tauri/` est l'application Tauri qui implémente le trait `UI` en stockant les pairs dans un état partagé. Le frontend **Vue 3 + Pinia + TypeScript + Tailwind v4** interroge le backend par **polling** (toutes les 2s) via la commande `get_peers`.

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
    fn show_progress_bar(&self, transfer_id: &str);
    fn update_progress_bar(&self, transfer_id: &str, bytes_sent: u64, total_bytes: u64);
    fn file_progress_bar(&self, transfer_id: &str, file_name: &str, file_bytes_sent: u64, file_total_bytes: u64);
    fn transfert_cancel(&self, transfer_id: &str);
    fn transfert_completed(&self, transfer_id: &str);
    fn transfert_received(&self, transfer_id: &str, peer: &str, bytes: u64, files: Vec<String>);
    fn tranfert_error(&self, transfer_id: &str, error: &ToolError);
}
```

Dans `desktop-app/src-tauri/src/commands.rs`, la structure `AppUI` implémente ce trait et émet des événements Tauri vers le frontend :
- `peer_found`/`peer_lost` maintiennent une liste partagée `Arc<Mutex<Vec<Peer>>>` et émettent `tool://peer_found` / `tool://peer_lost`
- Les méthodes de progression émettent `tool://transfer/start`, `tool://transfer/progress`, `tool://transfer/file_progress`, `tool://transfer/done`, `tool://transfer/cancel`, `tool://transfer/received`, `tool://transfer/error`

Le frontend récupère la liste des pairs via la commande `get_peers` appelée toutes les 2s (polling), et s'abonne aux événements `tool://transfer/*` pour la progression.

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
│   ├── Metadata JSON (transfer_id, rel_path, size, is_dir)
│   ├── Chunks pipelinés (len u32 BE + data, 1 Mo max, pas d'ack)
│   └── Marqueur COMPLETE + FinalAck
│
├── Stream 2 : fichier "photos/vacances/img1.jpg"
│   ├── Metadata JSON
│   ├── Chunks pipelinés
│   └── Marqueur COMPLETE + FinalAck
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
├── Tâche récepteur QUIC (port 58200)   (démarrée au setup, écrit dans Downloads/Toolé)
│   └── 1 tâche par connexion entrante  (handle_incoming_connection)
│       └── 1 tâche par stream/fichier  (receive_one, en parallèle)
├── Tâches émetteur (1 par envoi)       (send_files → start_sender)
│   └── 1 tâche par fichier             (send_entry, sémaphore 2 en parallèle)
```

---

## Structure des modules

### core/src/

| Module | Responsabilité |
|---|---|
| `lib.rs` | Trait UI, type Peer, exports des modules |
| `error.rs` | ToolError (IoError, Canceled, TransferError) |
| `utils.rs` | current_hostname, local_ip, device_id (hostname + suffixe stable) |
| `discovery.rs` | UDP broadcast (TOOLE_DISCOVERY / TOOLE_HERE), port 58199 |
| `file_certif.rs` | Certificat TLS auto-signé + SkipServerVerification (session unique) |
| `transfer.rs` | Transfert QUIC : endpoints, streams, chunks pipelinés, UI throttle |
| `sender.rs` | Côté émetteur : parcours des chemins, ouverture des streams en parallèle |
| `recever.rs` | Côté récepteur : serveur QUIC (port 58200), écriture dans Downloads/Toolé |

### desktop-app/src-tauri/src/

| Fichier | Responsabilité |
|---|---|
| `main.rs` | Point d'entrée, appelle `app_lib::run()` |
| `lib.rs` | Builder Tauri : manage state, invoke_handler, récepteur au démarrage |
| `commands.rs` | AppUI + commandes : start_discovery, stop_discovery, get_hostname, get_device_id, get_peers, send_files, cancel_transfer, read_clipboard, close_window, get_file_infos |

### desktop-app/ui/

| Fichier | Responsabilité |
|---|---|
| `index.html` | Point d'entrée Vite |
| `src/main.ts` | Bootstrap Vue 3 + Pinia, thème système |
| `src/App.vue` | Root component : navigation, envoi, découverte |
| `src/style.css` | Thème glassmorphism dark + Tailwind v4 |
| `src/types.ts` | Interfaces partagées (Peer, FileEntry) |
| `src/tauri.ts` | Wrapper invoke Tauri |
| `src/utils.ts` | Utilitaires (formatSize, extOf, fileVisual) |
| `src/stores/peers.ts` | Store Pinia — liste des pairs + polling 2s |
| `src/stores/files.ts` | Store Pinia — fichiers sélectionnés + tailles |
| `src/stores/transfers.ts` | Store Pinia — historique + progression, persistance localStorage |
| `src/stores/settings.ts` | Store Pinia — thème (auto/sombre/clair) + halo couleur |
| `src/components/HomePage.vue` | Accueil + zone de dépôt |
| `src/components/FileDropZone.vue` | Dépôt fichiers, Ctrl+V, sélecteur natif, drag & drop Tauri |
| `src/components/PeerList.vue` | Liste des pairs, sélection individuelle/groupée |
| `src/components/TransferPage.vue` | Page des transferts en cours |
| `src/components/TransferList.vue` | Cartes de transfert (barre, débit, annuler) |
| `src/components/HistoryPage.vue` | Historique des transferts terminés |
| `src/components/SettingsPage.vue` | Paramètres (thème, halo) |
| `src/components/SidebarNav.vue` | Barre latérale de navigation |
| `src/components/TitleBar.vue` | Titlebar custom (drag, min, close, détection macOS) |
| `src/components/Icon.vue` | Icônes SVG chargées depuis assets/icons |
| `src/components/AboutModal.vue` | Modale À propos glassmorph |

### Fenêtre et permissions

La fenêtre Tauri est configurée sans décoration native (`decorations: false`) avec fond transparent (`transparent: true`) et titlebar personnalisée (zone de drag `data-tauri-drag-region`, boutons Réduire et Fermer). Les permissions Tauri v2 sont déclarées dans `capabilities/default.json` : `core:default`, `core:window:allow-start-dragging`, `core:window:allow-minimize`, `core:window:allow-close`, `dialog:default`.

### Commandes Tauri additionnelles

| Commande | Rôle |
|---|---|
| `read_clipboard` | Lit le presse-papier système via `arboard` (Ctrl+V) |
| `close_window` | Ferme la fenêtre (fallback) |
| `get_file_infos` | Retourne taille et type (fichier/dossier) de chaque chemin |
| `cancel_transfer` | Annule un transfert par son id (stop flag + abort) |

---

> [SRS — exigences logicielles](srs.md) | Lire ensuite : [Protocole réseau](protocol.md)
