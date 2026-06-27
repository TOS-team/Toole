# Software Requirement Specifications (SRS)

## 1) Introduction

Toolé est un système P2P qui permet de transférer des fichiers entre ordinateurs sur un réseau local, avec découverte automatique par UDP broadcast, transfert sécurisé TLS et vérification d'intégrité SHA-256.

> Besoin produit : [prd.md](prd.md)

## 2) Description générale du système

### Contraintes de conception

- Backend en **Rust** avec Tokio async, séparé en workspace `core/` (biblio pure) + `app/` (Tauri).
- Frontend **HTML / CSS / JS vanilla**.
- Découverte en UDP broadcast sur le réseau local, transfert en TCP + TLS.
- Architecture sender / receiver (un seul receiver en V1).

## 3) Exigences fonctionnelles

### Découverte (F-001)

- **E-001** : Le sender diffuse périodiquement un paquet `TOOLE_DISCOVER` en UDP broadcast toutes les 2s sur le port 5199.
- **E-002** : Le receiver écoute sur le port 5199 et répond `TOOLE_HERE:<hostname>` à l'expéditeur.
- **E-003** : Le sender affiche la liste des receivers qui ont répondu. Timeout de 10s si aucun receiver.

### Transfert sécurisé (F-002)

- **E-004** : Le receiver initie une connexion TCP vers le sender (port 5200).
- **E-005** : Le sender génère un certificat TLS temporaire. Handshake TLS 1.3 automatique.
- **E-006** : Le sender envoie les métadonnées (nom fichier, taille, SHA-256, chunk_size) en JSON.
- **E-007** : Le receiver accuse réception des métadonnées (Ack `1`).
- **E-008** : Le fichier est transféré par chunks de taille fixe (1 Mo). Chaque chunk est précédé de son index (u32).
- **E-009** : Le receiver accuse réception de chaque chunk (Ack avec l'index).
- **E-010** : En cas de timeout (10s), le sender renvoie le même chunk (3 tentatives max).
- **E-011** : Un paquet `Complete` avec le SHA-256 final est envoyé après le dernier chunk.
- **E-012** : Le receiver vérifie le SHA-256 et renvoie `1` (OK) ou `0` (rejet).

### Interface (F-003)

- **E-013** : L'utilisateur voit deux boutons : Send et Receive.
- **E-014** : En mode Send, un sélecteur de fichier natif s'ouvre.
- **E-015** : En mode Receive, la liste des appareils découverts s'affiche.
- **E-016** : Une barre de progression avec vitesse indique l'avancement du transfert.
- **E-017** : Un bouton Annuler est toujours visible.

## 4) Exigences non fonctionnelles

- Découverte rapide (broadcast toutes les 2s, timeout 10s).
- Streaming mémoire : pas de chargement complet en RAM.
- Gestion d'erreur explicite (SHA-256, timeouts, chunks perdus).
- Architecture modulaire : `core/` (bibliothèque pure) + `app/` (Tauri).

---

> [PRD — vision produit](prd.md) | Lire ensuite : [Architecture technique](architecture.md)
