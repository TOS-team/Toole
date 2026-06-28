# Roadmap — Toolé

## ✅ Phase 1 — Core réseau (core/)

- [x] error.rs — types d'erreurs ToolError
- [x] lib.rs — trait UI, type Peer
- [x] utils.rs — current_hostname, local_ip
- [x] discovery.rs — UDP broadcast (TOOLE_DISCOVERY / TOOLE_HERE), port 58199
- [ ] transfer.rs — TCP + TLS + chunks 1 Mo + Ack + SHA-256 (plan uniquement)

---

## ✅ Phase 2 — App Tauri (desktop-app/)

- [x] commands.rs — TauriUI + commandes : start_discovery, stop_discovery, get_hostname, get_peers
- [x] index.html — structure UI (liste appareils, modale à propos)
- [x] js/main.js — polling get_peers toutes les 2s, sélection, init auto
- [x] css/index.css — thème glassmorphism dark

---

## 🔜 Phase 3 — Transfert de fichiers (QUIC)

- [ ] **Dépendance** : ajouter `quinn` (QUIC) dans core/Cargo.toml
- [ ] Serveur QUIC (écoute port 5200, accepte connexions entrantes)
- [ ] Client QUIC (initie connexion vers un pair)
- [ ] Metadata JSON par stream (rel_path, size, sha256, is_dir)
- [ ] Envoi par chunks de 1 Mo avec Ack par chunk
- [ ] SHA-256 progressif côté récepteur
- [ ] Timeouts et renvoi (10s, 3 tentatives max)
- [ ] Transfert parallèle de plusieurs fichiers
- [ ] Support des dossiers (parcours récursif, un stream par fichier)
- [ ] Sélecteur de fichiers/dossiers natif
- [ ] Glisser-déposer de fichiers et dossiers
- [ ] Barre de progression avec vitesse + temps restant
- [ ] Bouton Annuler par fichier et global
- [ ] Rafraîchissement du trait UI (progress, transfer_done, error)

---

## 🔮 Phase 4 — Polish & futures versions

- [ ] Icône dans la barre des tâches avec progression
- [ ] Notifications système (transfert terminé)
- [ ] Reprise de transfert (via index du dernier Ack)
- [ ] Historique local des transferts
- [ ] Nom appareil personnalisable
- [ ] Gestion des erreurs utilisateur (dialogue)
- [ ] Tests unitaires et d'intégration

---

> [Chiffrement et intégrité](crypto.md)
