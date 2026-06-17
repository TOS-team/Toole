# Roadmap — Toolé

## Phase 1 — Core réseau (core/)

- [ ] error.rs — types d'erreurs ToolError
- [ ] lib.rs — trait UI, types (Peer, Mode, TransferStatus)
- [ ] utils.rs — format_size, gen_ssid, hostname
- [ ] discovery.rs — UDP broadcast (TOOLE_DISCOVER / TOOLE_HERE)
- [ ] transfer.rs — TCP + TLS + chunks 1 Mo + Ack + SHA-256
- [ ] network.rs — nmcli hotspot (création, scan, connexion, destruction, scan_loop)

---

## Phase 2 — App Tauri (app/)

- [ ] commands.rs — TauriUI + invoke handlers
- [ ] index.html — structure UI (2 boutons, liste hotspots, progression)
- [ ] main.js — events Tauri + invoke + logique frontend
- [ ] style.css — minimaliste, responsive

---

## Phase 3 — Polish

- [ ] Timeouts et reconnexion
- [ ] Barre progression avec vitesse + temps restant
- [ ] Gestion des erreurs (hotspot perdu, chunk corrompu, annulation)
- [ ] Tests

---

## Phase 4 — Futures versions

- [ ] Plusieurs receivers simultanés
- [ ] Reprise de transfert (via index du dernier Ack)
- [ ] Historique local des transferts
- [ ] Nom appareil personnalisable
- [ ] Support Windows
