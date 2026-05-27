# Software Requirement Specifications (SRS)

## 1) Introduction

Toolé est un système P2P local qui permet de transférer des fichiers entre ordinateurs avec découverte automatique, transfert sécurisé TLS et vérification d'intégrité.

> Contexte produit : [PRD.md](PRD.md)
> Vision initiale : [brouillon.md](brouillon.md)

## 2) Description générale du système

### Contraintes de conception

- Backend en **Rust** avec Tokio async.
- Interface **Tauri** (React + TypeScript).
- Découverte en UDP broadcast, transfert en TCP + TLS.
- Architecture sender / receiver (pas de cluster).

## 3) Exigences fonctionnelles

### Découverte (F-001)

- **E-001** : Le sender diffuse périodiquement un packet discovery en UDP broadcast.
- **E-002** : Le packet contient `device_name`, `port`, `protocol_version`.
- **E-003** : Le receiver écoute les packets discovery et affiche la liste des senders.

### Transfert sécurisé (F-002)

- **E-004** : Le receiver initie une connexion TCP vers le sender.
- **E-005** : Le sender génère un certificat TLS temporaire.
- **E-006** : Handshake TLS obligatoire avant tout transfert.
- **E-007** : Le sender envoie les métadonnées (nom, taille, SHA-256, chunk_size).
- **E-008** : Le fichier est transféré par chunks de taille fixe.
- **E-009** : Chaque chunk est accompagné de son CRC32 pour validation rapide.
- **E-010** : Un packet `Complete` est envoyé après le dernier chunk.
- **E-011** : Le receiver vérifie le SHA-256 final et rejette le fichier en cas d'échec.

### Interface (F-003)

- **E-012** : L'utilisateur voit deux boutons : Send et Receive.
- **E-013** : En mode Receive, la liste des senders détectés s'affiche.
- **E-014** : Une barre de progression indique l'avancement du transfert.
- **E-015** : L'utilisateur valide manuellement la connexion après handshake TLS.

## 4) Exigences non fonctionnelles

- Découverte rapide et stable.
- Streaming mémoire : pas de chargement complet en RAM.
- Gestion d'erreur explicite (CRC32, SHA-256, timeouts).
- Architecture modulaire (crates Rust séparés).

---

> [ARCHITECTURE.md](ARCHITECTURE.md) — [PROTOCOL.md](PROTOCOL.md) — [SECURITY.md](SECURITY.md)
