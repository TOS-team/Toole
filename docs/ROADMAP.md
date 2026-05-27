# Roadmap — Toolé

## Phase 1 — Foundation

- Setup projet
- Architecture modules
- Tauri
- Tokio
- Logging

---

## Phase 2 — Discovery

- UDP broadcast
- Découverte appareils
- Affichage UI

---

## Phase 3 — TCP

- Connexion sender / receiver
- Sessions multiples

---

## Phase 4 — Security

- TLS
- Certificats
- Validation

---

## Phase 5 — File Transfer

- Chunking
- Progression
- CRC32
- SHA-256

---

## Phase 6 — UI

- Interface moderne
- Animations
- Expérience utilisateur

---

## Phase 7 — Optimisation

- Buffers
- Streaming
- Mémoire
- Performances

---

## Phase 8 — Packaging

- Builds Windows
- Builds Linux
- Builds macOS

---

## Difficultés techniques

- **Concurrence async** : gestion des tâches Tokio, channels, état partagé
- **Architecture protocole** : le protocole est le cœur du projet, une mauvaise conception rendra le projet difficile à maintenir
- **Sécurité** : TLS ajoute complexité, gestion des certificats, validation
- **Cross-platform** : Linux/Windows/macOS ont des firewalls et comportements réseau différents
