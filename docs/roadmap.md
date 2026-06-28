# Roadmap — Toolé

## Phase 1 — Core réseau (core/)

| Module | Statut |
|---|---|
| `error.rs` — types d'erreurs ToolError | ✅ |
| `lib.rs` — trait UI, type Peer | ✅ |
| `utils.rs` — current_hostname, local_ip | ✅ |
| `discovery.rs` — UDP broadcast, port 58199 | ✅ |
| `transfer.rs` — transfert QUIC, streams, chunks | 🔜 |

---

## Phase 2 — App Tauri (desktop-app/)

| Module | Statut |
|---|---|
| `commands.rs` — TauriUI, commandes invoke | ✅ |
| `lib.rs` — Builder Tauri, on_window_event, plugins | ✅ |
| `tauri.conf.json` — config fenêtre + build | ✅ |
| `capabilities/default.json` — permissions Tauri | ✅ |
| `App.vue` — root component, titlebar, layout | ✅ |
| `style.css` — thème glassmorphism dark + Tailwind v4 | ✅ |
| `main.ts` — bootstrap Vue 3 + Pinia | ✅ |
| `components/*.vue` — WelcomeHeader, FileDropZone, PeerList, AboutModal | ✅ |
| `stores/*.ts` — peers (polling), files (sizes) | ✅ |
| `tauri.ts` — wrapper invoke | ✅ |
| `utils.ts` — formatSize | ✅ |
| `types.ts` — Peer, FileEntry | ✅ |
| `read_clipboard` — commande Ctrl+V | ✅ |
| `close_window` — commande fallback | ✅ |
| `get_file_sizes` — commande tailles fichiers | ✅ |

---

## Phase 3 — Transfert QUIC

| Tâche | Statut |
|---|---|
| Dépendance `quinn` dans core/Cargo.toml | 🔜 |
| Serveur QUIC (port 58200, accepte connexions entrantes) | 🔜 |
| Client QUIC (initie connexion vers un pair) | 🔜 |
| Metadata JSON par stream (rel_path, size, sha256, is_dir) | 🔜 |
| Envoi par chunks de 1 Mo avec Ack par chunk | 🔜 |
| SHA-256 progressif côté récepteur | 🔜 |
| Timeouts et renvoi (10s, 3 tentatives max) | 🔜 |
| Transfert parallèle de plusieurs fichiers (multiplexage QUIC) | 🔜 |
| Support des dossiers (parcours récursif, un stream par fichier) | 🔜 |
| Méthodes transfer_progress / transfer_done / transfer_error dans le trait UI | 🔜 |

---

## Phase 4 — Interface & polish

| Tâche | Statut |
|---|---|
| Barre de progression avec vitesse + temps restant | 🔜 |
| Bouton Annuler par fichier et global | 🔜 |
| Icône dans la barre des tâches avec progression | 🔮 |
| Notifications système (transfert terminé, erreur) | 🔮 |
| Reprise de transfert (via index du dernier Ack) | 🔮 |
| Historique local des transferts | 🔮 |
| Nom appareil personnalisable | 🔮 |
| Gestion des erreurs utilisateur (dialogue) | 🔮 |
| Tests unitaires et d'intégration | 🔮 |

---

> [Chiffrement et intégrité](crypto.md)
