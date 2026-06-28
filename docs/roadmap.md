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
| `index.html` — structure UI | ✅ |
| `js/main.js` — polling, drag & drop, progression | ✅ |
| `css/index.css` — thème glassmorphism dark | ✅ |

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
| Sélecteur de fichiers/dossiers natif | 🔜 |
| Glisser-déposer de fichiers et dossiers | 🔜 |
| Barre de progression avec vitesse + temps restant | 🔜 |
| Bouton Annuler par fichier et global | 🔜 |
| Méthodes transfer_progress / transfer_done / transfer_error dans le trait UI | 🔜 |

---

## Phase 4 — Polish & futures versions

| Tâche | Statut |
|---|---|
| Icône dans la barre des tâches avec progression | 🔮 |
| Notifications système (transfert terminé, erreur) | 🔮 |
| Reprise de transfert (via index du dernier Ack) | 🔮 |
| Historique local des transferts | 🔮 |
| Nom appareil personnalisable | 🔮 |
| Gestion des erreurs utilisateur (dialogue) | 🔮 |
| Tests unitaires et d'intégration | 🔮 |

---

> [Chiffrement et intégrité](crypto.md)
