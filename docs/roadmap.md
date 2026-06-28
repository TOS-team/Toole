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

## 🔜 Phase 3 — Transfert de fichiers

- [ ] Protocole TCP + TLS (rustls)
- [ ] Envoi par chunks de 1 Mo
- [ ] Accusé de réception (Ack) par chunk
- [ ] SHA-256 progressif
- [ ] Timeouts et renvoi (3 tentatives max)
- [ ] Sélecteur de fichier natif
- [ ] Barre de progression avec vitesse + temps restant
- [ ] Bouton Annuler

---

## 🔮 Phase 4 — Polish & futures versions

- [ ] Plusieurs receivers simultanés
- [ ] Reprise de transfert (via index du dernier Ack)
- [ ] Historique local des transferts
- [ ] Nom appareil personnalisable
- [ ] Gestion des erreurs utilisateur
- [ ] Tests unitaires et d'intégration

---

> [Chiffrement et intégrité](crypto.md)
