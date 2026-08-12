# Roadmap — Toolé

## Phase 1 — Core réseau (core/)

| Module | Statut |
|---|---|
| `error.rs` — types d'erreurs ToolError | ✅ |
| `lib.rs` — trait UI, type Peer | ✅ |
| `utils.rs` — current_hostname, local_ip, device_id | ✅ |
| `discovery.rs` — UDP broadcast, port 58199 | ✅ |
| `transfer.rs` — transfert QUIC, streams, chunks | ✅ |
| `file_certif.rs` — certificat TLS auto-signé | ✅ |
| `sender.rs` — émetteur (chemins, streams parallèles) | ✅ |
| `recever.rs` — récepteur QUIC (port 58200) | ✅ |

---

## Phase 2 — App Tauri (desktop-app/)

| Module | Statut |
|---|---|
| `commands.rs` — AppUI, commandes invoke | ✅ |
| `lib.rs` — Builder Tauri, states, récepteur au setup | ✅ |
| `tauri.conf.json` — config fenêtre + build | ✅ |
| `capabilities/default.json` — permissions Tauri | ✅ |
| `App.vue` — root component, navigation, envoi | ✅ |
| `style.css` — thème glassmorphism dark + Tailwind v4 | ✅ |
| `main.ts` — bootstrap Vue 3 + Pinia | ✅ |
| `components/*.vue` — HomePage, FileDropZone, PeerList, TitleBar, SidebarNav, TransferPage/List, HistoryPage, SettingsPage, AboutModal, Icon | ✅ |
| `stores/*.ts` — peers (polling), files (sizes), transfers (historique), settings (thème/halo) | ✅ |
| `tauri.ts` — wrapper invoke | ✅ |
| `utils.ts` — formatSize, extOf, fileVisual | ✅ |
| `types.ts` — Peer, FileEntry | ✅ |
| `read_clipboard` — commande Ctrl+V | ✅ |
| `close_window` — commande fallback | ✅ |
| `get_file_infos` — commande tailles/types fichiers | ✅ |

---

## Phase 3 — Transfert QUIC ✅

| Tâche | Statut |
|---|---|
| Dépendance `quinn` dans core/Cargo.toml | ✅ |
| Serveur QUIC (port 58200, accepte connexions entrantes) | ✅ |
| Client QUIC (initie connexion vers un pair) | ✅ |
| Metadata JSON par stream (transfer_id, rel_path, size, is_dir) | ✅ |
| Envoi par chunks de 1 Mo pipelinés (sans ack par chunk) | ✅ |
| Transfert parallèle de plusieurs fichiers (multiplexage QUIC) | ✅ |
| Support des dossiers (parcours récursif, un stream par fichier) | ✅ |
| Progression UI (globale + par fichier, throttle ~20/s) | ✅ |
| Annulation (stop flag, reset stream, erreur côté récepteur) | ✅ |

---

## Phase 4 — Interface & polish

| Tâche | Statut |
|---|---|
| Barre de progression avec débit (globale + mini-barres par fichier) | ✅ |
| Bouton Annuler (par transfert) | ✅ |
| Historique local des transferts (localStorage, 200 entrées) | ✅ |
| Thème clair/sombre/auto + couleur de halo | ✅ |
| Miniatures et icônes par type de fichier | ✅ |
| Tests unitaires et d'intégration (Rust + frontend) | ✅ |
| Icône dans la barre des tâches avec progression | 🔮 |
| Notifications système (transfert terminé, erreur) | 🔮 |
| Reprise de transfert | 🔮 |
| Nom appareil personnalisable | 🔮 |
| Gestion des erreurs utilisateur (dialogue) | 🔮 |

---

> [Chiffrement et intégrité](crypto.md)
